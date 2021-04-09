use super::mode::DrawMode;
use crate::graphics::texture::Texture;

pub struct Mesh {
    pub mode: DrawMode,
    pub vertex: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub index: Option<wgpu::Buffer>,
    pub texture: Option<Texture>,
    pub count: u32,
}

impl Mesh {
    pub fn new(
        mode: DrawMode,
        bind_group: wgpu::BindGroup,
        vertex: wgpu::Buffer,
        count: u32,
        index: Option<wgpu::Buffer>,
     texture: Option<Texture>,
    ) -> Self {
        Self {
            mode,
            bind_group,
            vertex,
            index,
            count,
            texture,
        }
    }
}
