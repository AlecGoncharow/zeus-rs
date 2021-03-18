use crate::graphics;
use crate::graphics::Topology;
use crate::input::{keyboard, mouse};
use crate::math::Vec2;
use crate::math::Vec3;
use crate::timer;
use futures::executor::block_on;
use std::sync::Arc;
use winit::{event::*, event_loop::EventLoop, window::WindowBuilder};

/// A custom event type for the winit app.
pub enum EngineEvent {
    RequestRedraw,
}

pub struct Context {
    pub continuing: bool,
    pub keyboard_context: keyboard::KeyboardContext,
    pub mouse_context: mouse::MouseContext,
    pub gfx_context: graphics::renderer::GraphicsContext,
    pub timer_context: timer::TimeContext,
    pub ui_context: graphics::ui::UiContext,
    pub frame: Option<wgpu::SwapChainFrame>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub swap_chain: wgpu::SwapChain,
    pub sc_desc: wgpu::SwapChainDescriptor,
    pub window: winit::window::Window,
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub event_loop_proxy: std::sync::Mutex<winit::event_loop::EventLoopProxy<EngineEvent>>,
}

impl<'a> Context {
    pub fn new(clear_color: crate::math::Vec4) -> (Self, EventLoop<EngineEvent>) {
        let event_loop: EventLoop<EngineEvent> = EventLoop::with_user_event();
        let event_loop_proxy = std::sync::Mutex::new(event_loop.create_proxy());

        let window = WindowBuilder::new().build(&event_loop).unwrap();
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(&window) };
        let size = window.inner_size();

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
        }))
        .unwrap();

        let mut features = wgpu::Features::empty();
        // @TODO need to wrap this so that non Vulkan/DX12 don't offer multiple pipelines
        features.set(wgpu::Features::NON_FILL_POLYGON_MODE, true);
        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Request Device"),
                features,
                limits: wgpu::Limits::default(),
            },
            None, // Trace path
        ))
        .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let gfx_context = block_on(graphics::renderer::GraphicsContext::new(
            &window,
            &device,
            &sc_desc,
            clear_color,
        ));

        let frame = loop {
            match swap_chain.get_current_frame() {
                Ok(frame) => break frame,
                Err(e) => {
                    eprintln!("dropped frame: {:?}", e);
                    continue;
                }
            }
        };

        let ui_context = graphics::ui::UiContext::new(&device, &window, event_loop_proxy);
        let event_loop_proxy = std::sync::Mutex::new(event_loop.create_proxy());
        let ctx = Self {
            continuing: true,
            keyboard_context: keyboard::KeyboardContext::new(),
            mouse_context: mouse::MouseContext::new(),
            gfx_context,
            timer_context: timer::TimeContext::new(),
            ui_context,
            frame: Some(frame),
            device,
            queue,
            swap_chain,
            window,
            surface,
            adapter,
            sc_desc,
            event_loop_proxy,
        };

        (ctx, event_loop)
    }

    pub fn process_event(&mut self, event: &Event<'a, EngineEvent>) {
        self.ui_context.platform.handle_event(event);
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(_logical_size) => {}
                WindowEvent::CursorMoved { position, .. } => {
                    self.mouse_context
                        .set_last_position(Vec2::new(position.x as f32, position.y as f32));
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    let pressed = match state {
                        ElementState::Pressed => true,
                        ElementState::Released => false,
                    };
                    self.mouse_context.set_button(*button, pressed);
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state,
                            virtual_keycode: Some(keycode),
                            ..
                        },
                    ..
                } => {
                    let pressed = match state {
                        ElementState::Pressed => true,
                        ElementState::Released => false,
                    };
                    self.keyboard_context.set_key(*keycode, pressed);
                }
                WindowEvent::ScaleFactorChanged { .. } => {
                    // Nope.
                }
                _ => (),
            },
            Event::DeviceEvent { event, .. } => {
                if let DeviceEvent::MouseMotion { delta: (x, y) } = event {
                    self.mouse_context
                        .set_last_delta(Vec2::new(*x as f32, *y as f32));
                }
            }

            _ => (),
        };
    }

    pub fn set_camera(&mut self, camera: Arc<impl crate::graphics::CameraProjection + 'static>) {
        self.gfx_context.projection_transform = camera.projection_matrix();
        self.gfx_context.view_transform = camera.view_matrix();
    }

    pub fn start_drawing(&mut self) {
        if self.frame.is_none() {
            self.frame = loop {
                match self.swap_chain.get_current_frame() {
                    Ok(frame) => break Some(frame),
                    Err(e) => {
                        eprintln!("dropped frame: {:?}", e);
                        if e == wgpu::SwapChainError::Outdated {
                            let size = self.window.inner_size();
                            self.sc_desc = wgpu::SwapChainDescriptor {
                                usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
                                format: self.adapter.get_swap_chain_preferred_format(&self.surface),
                                width: size.width,
                                height: size.height,
                                present_mode: wgpu::PresentMode::Immediate,
                            };

                            self.swap_chain =
                                self.device.create_swap_chain(&self.surface, &self.sc_desc);

                            self.gfx_context.resize(
                                size,
                                &self.device,
                                &self.sc_desc,
                                &self.window,
                            );
                        }

                        continue;
                    }
                }
            };
        }

        loop {
            self.gfx_context.start(
                &self.device,
                &self.frame.as_ref().unwrap().output.view,
                &self.queue,
            );
            if self.gfx_context.command_encoder.is_some() {
                break;
            } else {
                println!("resizing");
            }
        }
    }

    pub fn draw(&mut self, mode: Topology, verts: &[(Vec3, Vec3)]) {
        self.gfx_context.draw(
            &self.frame.as_ref().unwrap().output.view,
            &self.device,
            mode,
            verts,
        );
    }

    pub fn draw_indexed(&mut self, mode: Topology, verts: &[(Vec3, Vec3)], indices: &[u16]) {
        self.gfx_context.draw_indexed(
            &self.frame.as_ref().unwrap().output.view,
            &self.device,
            mode,
            verts,
            indices,
        );
    }

    pub fn render(&mut self) {
        let command_encoder = self.ui_context.draw(
            &self.window,
            &self.device,
            &self.sc_desc,
            &self.frame.as_ref().unwrap(),
            &mut self.queue,
        );
        self.gfx_context.render(&self.queue, command_encoder);
        self.frame = None;
    }
}
