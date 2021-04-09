use super::mode::DrawMode;

pub struct Mesh {
    pub mode: DrawMode,
    pub vertex: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub index: Option<wgpu::Buffer>,
    pub count: u32,
}

impl Mesh {
    pub fn new(
        mode: DrawMode,
        bind_group: wgpu::BindGroup,
        vertex: wgpu::Buffer,
        index: Option<wgpu::Buffer>,
        count: u32,
    ) -> Self {
        Self {
            mode,
            bind_group,
            vertex,
            index,
            count,
        }
    }
}
