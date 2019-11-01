use crate::context::Context;
use crate::math::vec2::Vec2;
use std::collections::HashMap;
use winit::MouseButton;
use winit::MouseCursor;

/// Taken from (ggez)[https://github.com/ggez/ggez/blob/master/src/input/mouse.rs]
/// with some changes so I can use it with my types
/// Stores state information for the mouse.
#[derive(Clone, Debug)]
pub struct MouseContext {
    last_position: Vec2,
    last_delta: Vec2,
    buttons_pressed: HashMap<MouseButton, bool>,
    cursor_type: MouseCursor,
    cursor_grabbed: bool,
    cursor_hidden: bool,
}

impl MouseContext {
    pub(crate) fn new() -> Self {
        Self {
            last_position: Vec2::origin(),
            last_delta: Vec2::origin(),
            cursor_type: MouseCursor::Default,
            buttons_pressed: HashMap::new(),
            cursor_grabbed: false,
            cursor_hidden: false,
        }
    }

    pub(crate) fn set_last_position(&mut self, p: Vec2) {
        self.last_position = p;
    }

    pub(crate) fn set_last_delta(&mut self, p: Vec2) {
        self.last_delta = p;
    }

    pub(crate) fn set_button(&mut self, button: MouseButton, pressed: bool) {
        let _ = self.buttons_pressed.insert(button, pressed);
    }

    fn button_pressed(&self, button: MouseButton) -> bool {
        *(self.buttons_pressed.get(&button).unwrap_or(&false))
    }
}

impl Default for MouseContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the current mouse cursor type of the window.
pub fn cursor_type(ctx: &Context) -> MouseCursor {
    ctx.mouse_context.cursor_type
}

/// Get whether or not the mouse is grabbed (confined to the window)
pub fn cursor_grabbed(ctx: &Context) -> bool {
    ctx.mouse_context.cursor_grabbed
}

/// Set whether or not the mouse is hidden (invisible)
pub fn cursor_hidden(ctx: &Context) -> bool {
    ctx.mouse_context.cursor_hidden
}

/// Set whether or not the mouse is hidden (invisible).
/// Get the current position of the mouse cursor, in pixels.
/// Complement to [`set_position()`](fn.set_position.html).
/// Uses strictly window-only coordinates.
pub fn position(ctx: &Context) -> Vec2 {
    ctx.mouse_context.last_position.into()
}

/// Get the distance the cursor was moved during last frame, in pixels.
pub fn delta(ctx: &Context) -> Vec2 {
    ctx.mouse_context.last_delta.into()
}

/// Returns whether or not the given mouse button is pressed.
pub fn button_pressed(ctx: &Context, button: MouseButton) -> bool {
    ctx.mouse_context.button_pressed(button)
}
