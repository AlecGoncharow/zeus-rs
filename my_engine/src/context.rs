use crate::graphics;
use crate::input::{keyboard, mouse};
use crate::timer;
use winit::DeviceEvent;
use winit::ElementState;
use winit::Event;
use winit::EventsLoop;
use winit::KeyboardInput;
use winit::WindowEvent;

use crate::graphics::Topology;
use crate::math::Vec2;
use crate::math::Vec3;
use std::sync::Arc;

pub struct Context {
    pub continuing: bool,
    pub keyboard_context: keyboard::KeyboardContext,
    pub mouse_context: mouse::MouseContext,
    pub gfx_context: graphics::context::GraphicsContext,
    pub timer_context: timer::TimeContext,
}

impl Context {
    pub fn new() -> (Self, EventsLoop) {
        let event_loop = EventsLoop::new();

        let gfx_context = graphics::context::GraphicsContext::new_default(&event_loop);

        let ctx = Self {
            continuing: true,
            keyboard_context: keyboard::KeyboardContext::new(),
            mouse_context: mouse::MouseContext::new(),
            gfx_context,
            timer_context: timer::TimeContext::new(),
        };

        (ctx, event_loop)
    }

    pub fn process_event(&mut self, event: &Event) {
        match event.clone() {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(_logical_size) => {
                    //let hidpi_factor = self.gfx_context.window.get_hidpi_factor();
                    //let physical_size = logical_size.to_physical(hidpi_factor as f64);
                    //self.gfx_context.window.resize(physical_size);
                    //self.gfx_context.resize_viewport();
                    self.gfx_context.recreate_swapchain = true;
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
                    self.mouse_context.set_button(button, pressed);
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
                    self.keyboard_context.set_key(keycode, pressed);
                }
                WindowEvent::HiDpiFactorChanged(_) => {
                    // Nope.
                }
                _ => (),
            },
            Event::DeviceEvent { event, .. } => {
                if let DeviceEvent::MouseMotion { delta: (x, y) } = event {
                    self.mouse_context
                        .set_last_delta(Vec2::new(x as f32, y as f32));
                }
            }

            _ => (),
        };
    }

    pub fn set_camera(&mut self, camera: Arc<impl crate::graphics::CameraProjection + 'static>) {
        self.gfx_context.projection_transform = camera.projection_matrix();
        self.gfx_context.view_transform = camera.view_matrix();
    }

    pub fn start_drawing(&mut self, clear_color: crate::math::Vec4) {
        loop {
            self.gfx_context.start(clear_color);
            if self.gfx_context.graphics_command_buffer.is_some() {
                break;
            } else {
                println!("resizing");
            }
        }
    }

    pub fn draw(&mut self, mode: Topology, verts: &[(Vec3, Vec3)]) {
        self.gfx_context.draw(mode, verts);
    }

    pub fn draw_indexed(&mut self, mode: Topology, verts: &[(Vec3, Vec3)], indices: &[u16]) {
        self.gfx_context.draw_indexed(mode, verts, indices);
    }

    pub fn render(&mut self) {
        self.gfx_context.render();
    }
}
