use crate::{context::Context, shader};
use anyhow::*;
use winit::dpi;
use winit::event::ElementState;
use winit::event::Event;
use winit::event::KeyboardInput;
use winit::event::ModifiersState;
use winit::event::MouseButton;
use winit::event::MouseScrollDelta;
use winit::event::VirtualKeyCode;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::input::{keyboard, mouse};

pub trait EventHandler {
    // Called upon each logic update to the game.
    /// This should be where the game's logic takes place.
    fn update(&mut self, _ctx: &mut Context) -> Result<()>;

    /// Called to do the drawing of your game.
    fn draw(&mut self, _ctx: &mut Context) -> Result<()>;

    /// A mouse button was pressed
    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
    }

    /// A mouse button was released
    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
    }

    /// The mouse was moved; it provides both absolute x and y coordinates in the window,
    /// and relative x and y coordinates compared to its last position.
    fn mouse_motion_event(&mut self, _ctx: &mut Context, _x: f32, _y: f32, _dx: f32, _dy: f32) {}

    /// The mousewheel was scrolled, vertically (y, positive away from and negative toward the user)
    /// or horizontally (x, positive to the right and negative to the left).
    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, _y: f32) {}

    /// A keyboard button was pressed.
    fn key_down_event(&mut self, ctx: &mut Context, keycode: VirtualKeyCode, _repeat: bool) {
        if keycode == VirtualKeyCode::Escape {
            quit(ctx);
        }
    }

    /// A keyboard button was released.
    fn key_up_event(&mut self, _ctx: &mut Context, _keycode: VirtualKeyCode) {}

    /// A unicode character was received, usually from keyboard input.
    /// This is the intended way of facilitating text input.
    fn text_input_event(&mut self, _ctx: &mut Context, _character: char) {}

    /// Called when the window is shown or hidden.
    fn focus_event(&mut self, _ctx: &mut Context, _gained: bool) {}

    /// Called upon a quit event.  If it returns false,
    /// the game does not exit (the quit event is cancelled).
    fn quit_event(&mut self, _ctx: &mut Context) -> bool {
        println!("quit_event() callback called, quitting...");
        true
    }

    /// Called when the user resizes the window
    fn resize_event(&mut self, _ctx: &mut Context, _width: f32, _height: f32) {}

    fn key_mods_changed(&mut self, _ctx: &mut Context, _modifiers_state: ModifiersState) {}
}

pub fn quit(ctx: &mut Context) {
    ctx.continuing = false;
}

pub fn run<S: 'static>(
    events_loop: EventLoop<crate::context::EngineEvent>,
    mut ctx: Context,
    mut state: S,
) -> !
where
    S: EventHandler,
{
    let shader_reload_handle = Arc::new(AtomicBool::new(false));
    shader::start_hotloader(Arc::clone(&shader_reload_handle));
    #[cfg(target_os = "macos")]
    let mut wait = false;

    events_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        ctx.process_event(&event);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(logical_size) => {
                    // let actual_size = logical_size;

                    state.resize_event(
                        &mut ctx,
                        logical_size.width as f32,
                        logical_size.height as f32,
                    );

                    ctx.resize();
                }
                WindowEvent::CloseRequested => {
                    if state.quit_event(&mut ctx) {
                        quit(&mut ctx);
                    }
                }
                WindowEvent::Focused(gained) => {
                    state.focus_event(&mut ctx, gained);

                    // @HACK @FIXME https://github.com/gfx-rs/wgpu/issues/1783 Just Mac things :)
                    #[cfg(target_os = "macos")]
                    if !gained {
                        *control_flow = ControlFlow::Wait;
                        wait = true;
                    } else {
                        wait = false;
                    }
                }
                WindowEvent::ReceivedCharacter(ch) => {
                    state.text_input_event(&mut ctx, ch);
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(keycode),
                            ..
                        },
                    ..
                } => {
                    let repeat = keyboard::is_key_repeated(&ctx);

                    state.key_down_event(&mut ctx, keycode, repeat);
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Released,
                            virtual_keycode: Some(keycode),
                            ..
                        },
                    ..
                } => {
                    state.key_up_event(&mut ctx, keycode);
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    let (x, y) = match delta {
                        MouseScrollDelta::LineDelta(x, y) => (x as f32, y as f32),
                        MouseScrollDelta::PixelDelta(dpi::PhysicalPosition { x, y }) => {
                            (x as f32, y as f32)
                        }
                    };
                    state.mouse_wheel_event(&mut ctx, x, y);
                }

                WindowEvent::MouseInput {
                    state: element_state,
                    button,
                    ..
                } => {
                    println!("input detected");
                    let position = mouse::position(&ctx);
                    match element_state {
                        ElementState::Pressed => {
                            state.mouse_button_down_event(&mut ctx, button, position.x, position.y)
                        }
                        ElementState::Released => {
                            state.mouse_button_up_event(&mut ctx, button, position.x, position.y)
                        }
                    }
                }
                WindowEvent::CursorMoved { .. } => {
                    let position = mouse::position(&ctx);
                    let delta = mouse::delta(&ctx);
                    state.mouse_motion_event(&mut ctx, position.x, position.y, delta.x, delta.y);
                }
                WindowEvent::ModifiersChanged(modifiers_state) => {
                    state.key_mods_changed(&mut ctx, modifiers_state);
                }

                WindowEvent::AxisMotion { .. } => {}
                WindowEvent::CursorLeft { .. } => {}
                WindowEvent::TouchpadPressure { .. } => {}

                x => {
                    eprintln!("ignoring window event {:?}", x);
                }
            },
            Event::DeviceEvent { .. } => (),
            Event::MainEventsCleared => {
                #[cfg(target_os = "macos")]
                if wait {
                    return
                }

                if shader_reload_handle.load(Ordering::Relaxed) {
                    ctx.reload_shaders();
                    shader_reload_handle.store(false, Ordering::Relaxed);
                }

                if !ctx.continuing {
                    *control_flow = ControlFlow::Exit;
                } else {
                    ctx.timer_context.tick();
                    let _ = state.update(&mut ctx);
                    let _ = state.draw(&mut ctx);

                    // CLEAR VALUES
                    ctx.mouse_context.set_last_delta((0.0, 0.0).into());
                }
            }

            _ => (),
        }
    });
}
