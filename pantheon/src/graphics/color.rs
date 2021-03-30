#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,

    a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            r, g, b, a: 255
        }
    }

    pub fn floats(r: f32, g: f32, b: f32) -> Self {
        Self::new(
                (255.0 * r) as u8,
                (255.0 * g) as u8,
                (255.0 * b) as u8,
            )
    }
}

impl From<(f32, f32, f32)> for Color {
    fn from(tuple: (f32, f32, f32)) -> Self {
        Self::floats(
                 tuple.0,
                tuple.1,
                 tuple.2
            )
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(tuple: (u8, u8, u8)) -> Self {
       Self::new(tuple.0, tuple.1, tuple.2) 
    }
}