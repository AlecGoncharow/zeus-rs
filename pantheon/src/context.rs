use crate::graphics;
use crate::graphics::mode::{DrawMode, PolygonMode};
use crate::graphics::texture::TextureKind;
use crate::input::{keyboard, mouse};
use crate::math::Vec2;
use crate::timer;
use crate::Mat4;
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
    pub frame: Option<wgpu::SurfaceFrame>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub window: winit::window::Window,
    pub surface: wgpu::Surface,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub adapter: wgpu::Adapter,
    pub event_loop_proxy: std::sync::Mutex<winit::event_loop::EventLoopProxy<EngineEvent>>,
    pub forced_draw_mode: Option<PolygonMode>,
}

impl<'a> Context {
    pub fn new(clear_color: crate::math::Vec4) -> (Self, EventLoop<EngineEvent>) {
        let event_loop: EventLoop<EngineEvent> = EventLoop::with_user_event();

        let window = WindowBuilder::new().build(&event_loop).unwrap();
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
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

        let (device, queue) = match block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Request Device"),
                features,
                limits: wgpu::Limits::default(),
            },
            None, // Trace path
        )) {
            Ok(stuff) => stuff,
            Err(_) => block_on(adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Fallback Device"),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            ))
            .unwrap(),
        };

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate,
        };
        surface.configure(&device, &surface_config);

        let gfx_context = block_on(graphics::renderer::GraphicsContext::new(
            &window,
            &device,
            &surface_config,
            clear_color,
        ));

        let frame = loop {
            match surface.get_current_frame() {
                Ok(frame) => break frame,
                Err(e) => {
                    eprintln!("dropped frame: {:?}", e);
                    continue;
                }
            }
        };

        let event_loop_proxy = std::sync::Mutex::new(event_loop.create_proxy());
        let ctx = Self {
            continuing: true,
            keyboard_context: keyboard::KeyboardContext::new(),
            mouse_context: mouse::MouseContext::new(),
            gfx_context,
            timer_context: timer::TimeContext::new(),
            frame: Some(frame),
            device,
            queue,
            window,
            surface,
            surface_config,
            adapter,
            event_loop_proxy,
            forced_draw_mode: None,
        };

        (ctx, event_loop)
    }

    pub fn process_event(&mut self, event: &Event<'a, EngineEvent>) {
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
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta: (x, y) },
                ..
            } => {
                self.mouse_context
                    .set_last_delta(Vec2::new(*x as f32, *y as f32));
            }

            _ => (),
        };
    }

    pub fn set_camera(&mut self, camera: Arc<impl crate::graphics::CameraProjection + 'static>) {
        self.gfx_context.entity_uniforms.projection = camera.projection_matrix();
        self.gfx_context.entity_uniforms.view = camera.view_matrix();
    }

    pub fn set_projection(&mut self, mat: Mat4) {
        self.gfx_context.entity_uniforms.projection = mat;
    }

    pub fn set_view(&mut self, mat: Mat4) {
        self.gfx_context.entity_uniforms.view = mat;
    }

    pub fn set_model(&mut self, mat: Mat4) {
        self.gfx_context.entity_uniforms.model = mat;
    }

    pub fn set_cursor_icon(&mut self, icon: winit::window::CursorIcon) {
        self.window.set_cursor_icon(icon);
    }

    pub fn reload_shaders(&mut self) {
        self.gfx_context
            .reload_shaders(&self.device, &self.surface_config);
    }

    pub fn start_drawing(&mut self) {
        if self.frame.is_none() {
            self.frame = loop {
                match self.surface.get_current_frame() {
                    Ok(frame) => break Some(frame),
                    Err(e) => {
                        eprintln!("dropped frame: {:?}", e);
                        if e == wgpu::SurfaceError::Outdated {
                            let size = self.window.inner_size();
                            self.surface_config = wgpu::SurfaceConfiguration {
                                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                                format: self.surface.get_preferred_format(&self.adapter).unwrap(),
                                width: size.width,
                                height: size.height,
                                present_mode: wgpu::PresentMode::Immediate,
                            };

                            self.surface.configure(&self.device, &self.surface_config);

                            self.gfx_context.resize(
                                size,
                                &self.device,
                                &self.surface_config,
                                &self.window,
                            );
                        }

                        continue;
                    }
                }
            };
        }

        self.gfx_context.start();
    }

    pub fn resize(&mut self) {
        let size = self.window.inner_size();
        self.surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface.get_preferred_format(&self.adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate,
        };

        self.surface.configure(&self.device, &self.surface_config);

        self.gfx_context
            .resize(size, &self.device, &self.surface_config, &self.window);
    }

    pub fn draw<F>(&mut self, mut mode: DrawMode, verts: &[F])
    where
        F: bytemuck::Pod,
    {
        if let Some(polygon_mode) = self.forced_draw_mode {
            mode.inner_mut().set_inner(polygon_mode);
        }

        self.gfx_context.draw::<F>(&self.device, mode, verts);
    }

    pub fn draw_indexed<F>(&mut self, mut mode: DrawMode, verts: &[F], indices: &[u32])
    where
        F: bytemuck::Pod,
    {
        if let Some(polygon_mode) = self.forced_draw_mode {
            mode.inner_mut().set_inner(polygon_mode);
        }

        self.gfx_context
            .draw_indexed::<F>(&self.device, mode, verts, indices);
    }

    pub fn draw_textured<F>(&mut self, mode: DrawMode, verts: &[F], texture: TextureKind)
    where
        F: bytemuck::Pod,
    {
        self.gfx_context
            .draw_textured::<F>(&self.device, mode, verts, texture);
    }

    pub fn render(&mut self) {
        self.gfx_context.render(
            &self.device,
            &self
                .frame
                .as_ref()
                .unwrap()
                .output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
            &self.queue,
        );
        self.frame = None;
    }
}
