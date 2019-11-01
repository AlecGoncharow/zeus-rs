// @TODO generalize

#[derive(Debug, Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn origin() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}
