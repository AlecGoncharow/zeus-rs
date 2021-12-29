use super::mode::PolygonMode;
use crate::{graphics::texture::Texture, Mat4};

pub struct Mesh {
    pub mode: PolygonMode,
    pub model_matrix: Mat4,
    pub texture: Option<Texture>,
    pub vert_count: u32,
}

impl Mesh {
    pub fn new(
        mode: PolygonMode,
        model_matrix: Mat4,
        texture: Option<Texture>,
        vert_count: u32,
    ) -> Self {
        Self {
            mode,
            model_matrix,
            vert_count,
            texture,
        }
    }
}
