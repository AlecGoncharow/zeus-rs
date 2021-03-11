use crate::context::Context;
use std::collections::HashSet;
use winit::event::VirtualKeyCode;

/// Taken from / Inspired by (ggez)[https://github.com/ggez/ggez/blob/master/src/input/keyboard.rs]
/// with some changes so I can use it with my types
/// Set the current position of the mouse cursor, in pixels.
/// Uses strictly window-only coordinates.

/// Tracks held down keyboard keys, active keyboard modifiers,
/// and figures out if the system is sending repeat keystrokes.
#[derive(Clone, Debug)]
pub struct KeyboardContext {
    /// A simple mapping of which key code has been pressed.
    /// We COULD use a `Vec<bool>` but turning Rust enums to and from
    /// integers is unsafe and a set really is what we want anyway.
    pressed_keys_set: HashSet<VirtualKeyCode>,

    // These two are necessary for tracking key-repeat.
    last_pressed: Option<VirtualKeyCode>,
    current_pressed: Option<VirtualKeyCode>,
}

impl KeyboardContext {
    pub(crate) fn new() -> Self {
        Self {
            // We just use 256 as a number Big Enough For Keyboard Keys to try to avoid resizing.
            pressed_keys_set: HashSet::with_capacity(256),
            last_pressed: None,
            current_pressed: None,
        }
    }

    pub(crate) fn set_key(&mut self, key: VirtualKeyCode, pressed: bool) {
        if pressed {
            let _ = self.pressed_keys_set.insert(key);
            self.last_pressed = self.current_pressed;
            self.current_pressed = Some(key);
        } else {
            let _ = self.pressed_keys_set.remove(&key);
            self.current_pressed = None;
        }
    }

    pub(crate) fn is_key_pressed(&self, key: VirtualKeyCode) -> bool {
        self.pressed_keys_set.contains(&key)
    }

    pub(crate) fn is_key_repeated(&self) -> bool {
        if self.last_pressed.is_some() {
            self.last_pressed == self.current_pressed
        } else {
            false
        }
    }

    pub(crate) fn pressed_keys(&self) -> &HashSet<VirtualKeyCode> {
        &self.pressed_keys_set
    }
}

impl Default for KeyboardContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Checks if a key is currently pressed down.
pub fn is_key_pressed(ctx: &Context, key: VirtualKeyCode) -> bool {
    ctx.keyboard_context.is_key_pressed(key)
}

/// Checks if the last keystroke sent by the system is repeated,
/// like when a key is held down for a period of time.
pub fn is_key_repeated(ctx: &Context) -> bool {
    ctx.keyboard_context.is_key_repeated()
}

/// Returns a reference to the set of currently pressed keys.
pub fn pressed_keys(ctx: &Context) -> &HashSet<VirtualKeyCode> {
    ctx.keyboard_context.pressed_keys()
}
