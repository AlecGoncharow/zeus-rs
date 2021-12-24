use pantheon::math::Vec2;
use pantheon::math::Vec3;
use pantheon::Color;

pub enum VertexKind {
    Basic,
    Shaded,
    Textured,
}

pub enum Vertex {
    Basic(BasicVertex),
    Shaded(ShadedVertex),
}

impl From<(Vec3, Color, Vec3)> for Vertex {
    fn from(vecs: (Vec3, Color, Vec3)) -> Self {
        Self::Shaded(ShadedVertex::from(vecs))
    }
}

impl From<(Vec3, Color)> for Vertex {
    fn from(vecs: (Vec3, Color)) -> Self {
        Self::Basic(BasicVertex::from(vecs))
    }
}

unsafe impl bytemuck::Pod for ShadedVertex {}
unsafe impl bytemuck::Zeroable for ShadedVertex {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ShadedVertex {
    pub position: Vec3,
    pub color: Color,
    pub normal: Vec3,
}

impl ShadedVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

impl From<(Vec3, Color, Vec3)> for ShadedVertex {
    fn from(vecs: (Vec3, Color, Vec3)) -> Self {
        Self {
            position: vecs.0,
            color: vecs.1,
            normal: vecs.2,
        }
    }
}

unsafe impl bytemuck::Pod for BasicVertex {}
unsafe impl bytemuck::Zeroable for BasicVertex {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct BasicVertex {
    pub position: Vec3,
    pub color: Color,
}

impl BasicVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

impl From<(Vec3, Color)> for BasicVertex {
    fn from(vecs: (Vec3, Color)) -> Self {
        Self {
            position: vecs.0,
            color: vecs.1,
        }
    }
}

unsafe impl bytemuck::Pod for TexturedVertex {}
unsafe impl bytemuck::Zeroable for TexturedVertex {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TexturedVertex {
    pub position: Vec3,
    pub color: Color,
    pub uv_coords: Vec2,
}

impl TexturedVertex {
    pub fn new(position: Vec3, color: Color, uv_coords: Vec2) -> Self {
        Self {
            position,
            color,
            uv_coords,
        }
    }

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

impl From<(Vec3, Color, Vec2)> for TexturedVertex {
    fn from(vecs: (Vec3, Color, Vec2)) -> Self {
        Self {
            position: vecs.0,
            color: vecs.1,
            uv_coords: vecs.2,
        }
    }
}
