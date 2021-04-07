use super::mode::DrawMode;

pub struct Mesh {
    pub mode: DrawMode,
    pub vertex: wgpu::Buffer,
    //@TODO FIXME use offsets into single bindgroup's instead
    pub shadow: wgpu::BindGroup,
    pub forward: wgpu::BindGroup,
    pub index: Option<wgpu::Buffer>,
    pub count: u32,
}

impl Mesh {
    pub fn new(
        mode: DrawMode,
        shadow: wgpu::BindGroup,
        forward: wgpu::BindGroup,
        vertex: wgpu::Buffer,
        index: Option<wgpu::Buffer>,
        count: u32,
    ) -> Self {
        Self {
            mode,
            shadow,
            forward,
            vertex,
            index,
            count,
        }
    }
}
