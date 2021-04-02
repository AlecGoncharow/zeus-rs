#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,

    pub a: f32,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self::floats(r as f32 / 255., g as f32 / 255., b as f32 / 255.)
    }

    pub fn floats(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
}

impl From<(f32, f32, f32)> for Color {
    fn from(tuple: (f32, f32, f32)) -> Self {
        Self::floats(tuple.0, tuple.1, tuple.2)
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(tuple: (u8, u8, u8)) -> Self {
        Self::new(tuple.0, tuple.1, tuple.2)
    }
}
