use crate::Vec4;

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

    pub const fn floats(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub const fn floats_a(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn interpolate(color_1: Self, color_2: Self, blend: f32) -> Self {
        let color_1_weight = 1. - blend;
        Self {
            r: (color_1_weight * color_1.r) + (blend * color_2.r),
            g: (color_1_weight * color_1.g) + (blend * color_2.g),
            b: (color_1_weight * color_1.b) + (blend * color_2.b),
            a: (color_1_weight * color_1.a) + (blend * color_2.a),
        }
    }
}

impl From<Color> for Vec4 {
    fn from(color: Color) -> Self {
        Self {
            x: color.r,
            y: color.g,
            z: color.b,
            w: color.a,
        }
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

impl From<Color> for wgpu::Color {
    fn from(color: Color) -> Self {
        Self {
            r: color.r as f64,
            g: color.g as f64,
            b: color.b as f64,
            a: color.a as f64,
        }
    }
}
