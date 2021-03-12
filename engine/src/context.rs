use crate::graphics;
use crate::graphics::Topology;
use crate::input::{keyboard, mouse};
use crate::math::Vec2;
use crate::math::Vec3;
use crate::timer;
use futures::executor::block_on;
use std::sync::Arc;
use winit::{event::*, event_loop::EventLoop, window::WindowBuilder};

pub struct Context {
    pub continuing: bool,
    pub keyboard_context: keyboard::KeyboardContext,
    pub mouse_context: mouse::MouseContext,
    pub gfx_context: graphics::renderer::GraphicsContext,
    pub timer_context: timer::TimeContext,
    pub frame: wgpu::SwapChainFrame,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub swap_chain: wgpu::SwapChain,
}

impl<'a> Context {
    pub fn new(clear_color: crate::math::Vec4) -> (Self, EventLoop<()>) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(&window) };
        let size = window.inner_size();

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
        }))
        .unwrap();

        println!("{:#?}", adapter.features());
        let mut features = wgpu::Features::empty();
        features.set(wgpu::Features::SHADER_FLOAT64, true);
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
            present_mode: wgpu::PresentMode::Fifo,
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

        let ctx = Self {
            continuing: true,
            keyboard_context: keyboard::KeyboardContext::new(),
            mouse_context: mouse::MouseContext::new(),
            gfx_context,
            timer_context: timer::TimeContext::new(),
            frame,
            device,
            queue,
            swap_chain,
        };

        (ctx, event_loop)
    }

    pub fn process_event(&mut self, event: &Event<'a, ()>) {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(_logical_size) => {
                    //let hidpi_factor = self.gfx_context.window.get_hidpi_factor();
                    //let physical_size = logical_size.to_physical(hidpi_factor as f64);
                    //self.gfx_context.window.resize(physical_size);
                    //self.gfx_context.resize_viewport();
                    //self.gfx_context.recreate_swapchain = true;
                }
                WindowEvent::CursorMoved {
                    position: logical_position,
                    ..
                } => {
                    self.mouse_context.set_last_position(Vec2::new(
                        logical_position.x as f32,
                        logical_position.y as f32,
                    ));
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
        self.frame = loop {
            match self.swap_chain.get_current_frame() {
                Ok(frame) => break frame,
                Err(e) => {
                    eprintln!("dropped frame: {:?}", e);
                    continue;
                }
            }
        };

        loop {
            self.gfx_context
                .start(&self.device, &self.frame.output.view, &self.queue);
            if self.gfx_context.command_encoder.is_some() {
                break;
            } else {
                println!("resizing");
            }
        }
    }

    pub fn draw(&mut self, mode: Topology, verts: &[(Vec3, Vec3)]) {
        self.gfx_context
            .draw(&self.frame.output.view, &self.device, mode, verts);
    }

    pub fn draw_indexed(&mut self, mode: Topology, verts: &[(Vec3, Vec3)], indices: &[u16]) {
        self.gfx_context
            .draw_indexed(&self.frame.output.view, &self.device, mode, verts, indices);
    }

    pub fn render(&mut self) {
        self.gfx_context.render(&self.queue);
    }
}
