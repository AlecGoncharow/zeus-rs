use std::path::PathBuf;

use crate::graphics;
use crate::graphics::mode::PolygonMode;
use crate::input::{keyboard, mouse};
use crate::math::Vec2;
use crate::shader;
use crate::timer;
use futures::executor::block_on;
use graphics::prelude::*;
use winit::{event::*, event_loop::EventLoop, window::WindowBuilder};

/// A custom event type for the winit app.
pub enum EngineEvent {
    RequestRedraw,
}

pub struct Context<'a> {
    pub continuing: bool,
    pub keyboard_context: keyboard::KeyboardContext,
    pub mouse_context: mouse::MouseContext,
    pub gfx_context: graphics::renderer::GraphicsContext,
    pub wrangler: RenderWrangler<'a>,
    pub timer_context: timer::TimeContext,
    pub shader_context: shader::ShaderContext,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub window: winit::window::Window,
    pub surface: wgpu::Surface,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub adapter: wgpu::Adapter,
    pub event_loop_proxy: std::sync::Mutex<winit::event_loop::EventLoopProxy<EngineEvent>>,
    pub forced_draw_mode: Option<PolygonMode>,
}

impl<'a, 'winit> Context<'a> {
    pub fn new(
        present_mode: wgpu::PresentMode,
        clear_color: crate::math::Vec4,
        shader_path: PathBuf,
    ) -> (Self, EventLoop<EngineEvent>) {
        let event_loop: EventLoop<EngineEvent> = EventLoop::with_user_event();

        let window = WindowBuilder::new().build(&event_loop).unwrap();
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(&window) };
        let size = window.inner_size();

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        println!("[adapter.backend] {:#?}", adapter.get_info().backend);
        println!("[adapter.features] {:#?}", adapter.features());

        let mut features = wgpu::Features::empty();
        // @TODO need to wrap this so that non Vulkan/DX12 don't offer multiple pipelines
        #[cfg(not(target_os = "macos"))]
        features.set(wgpu::Features::POLYGON_MODE_LINE, true);
        #[cfg(not(target_os = "macos"))]
        features.set(wgpu::Features::POLYGON_MODE_POINT, true);
        features.set(wgpu::Features::PUSH_CONSTANTS, true);
        features.set(wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER, true);
        features.set(wgpu::Features::DEPTH_CLIP_CONTROL, true);

        let (device, queue) = match block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Request Device"),
                features,
                limits: wgpu::Limits {
                    /// AMD pls https://www.khronos.org/registry/vulkan/specs/1.1/html/vkspec.html#limits-minmax
                    max_push_constant_size: 256,
                    ..wgpu::Limits::default()
                },
            },
            None, // Trace path
        )) {
            Ok(stuff) => stuff,
            Err(e) => {
                eprintln!("[request_device] {:#?}", e);
                block_on(adapter.request_device(
                    &wgpu::DeviceDescriptor {
                        label: Some("Fallback Device"),
                        features: wgpu::Features::empty(),
                        limits: wgpu::Limits::default(),
                    },
                    None,
                ))
                .unwrap()
            }
        };
        println!("[device.features] {:#?}", device.features());

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode,
        };
        surface.configure(&device, &surface_config);

        let gfx_context = graphics::renderer::GraphicsContext::new(&window, clear_color);

        let wrangler = RenderWrangler::new();

        let shader_src_path = shader_path.clone();
        let shader_spirv_path = shader_path.join("build");
        let shader_context = shader::ShaderContext {
            shader_src_path,
            shader_spirv_path,
        };

        let event_loop_proxy = std::sync::Mutex::new(event_loop.create_proxy());
        let ctx = Self {
            continuing: true,
            keyboard_context: keyboard::KeyboardContext::new(),
            mouse_context: mouse::MouseContext::new(),
            gfx_context,
            timer_context: timer::TimeContext::new(),
            device,
            queue,
            window,
            surface,
            surface_config,
            adapter,
            event_loop_proxy,
            forced_draw_mode: None,
            wrangler,
            shader_context,
        };

        (ctx, event_loop)
    }

    pub fn process_event(&mut self, event: &Event<'winit, EngineEvent>) {
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

    pub fn set_cursor_icon(&mut self, icon: winit::window::CursorIcon) {
        self.window.set_cursor_icon(icon);
    }

    pub fn reload_shaders(&mut self) {
        self.wrangler
            .reload_shaders(&self.device, &self.shader_context, &self.surface_config);
    }

    pub fn start_drawing(&mut self) {}

    pub fn resize(&mut self) {
        let size = self.window.inner_size();
        self.surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface.get_preferred_format(&self.adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: self.surface_config.present_mode,
        };

        self.surface.configure(&self.device, &self.surface_config);

        self.gfx_context.resize(size, &self.window);
    }

    pub fn render(&mut self) {
        let current_texture = {
            if let Ok(texture) = self.surface.get_current_texture() {
                texture
            } else {
                // :)
                return;
            }
        };

        self.gfx_context
            .render(&self.wrangler, &self.device, current_texture, &self.queue);
    }
}
