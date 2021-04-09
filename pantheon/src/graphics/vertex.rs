use crate::math::Vec2;
use crate::math::Vec3;
use crate::Color;

pub enum VertexKind {
    Vertex(Vertex),
    ShadedVertex(ShadedVertex),
}

impl From<(Vec3, Color, Vec3)> for VertexKind {
    fn from(vecs: (Vec3, Color, Vec3)) -> Self {
        Self::ShadedVertex(ShadedVertex::from(vecs))
    }
}

impl From<(Vec3, Color)> for VertexKind {
    fn from(vecs: (Vec3, Color)) -> Self {
        Self::Vertex(Vertex::from(vecs))
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
            array_stride: std::mem::size_of::<ShadedVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float3,
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

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: Vec3,
    pub color: Color,
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float4,
                },
            ],
        }
    }
}

impl From<(Vec3, Color)> for Vertex {
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
    _padding1: u32,
    pub color: Color,
    _padding2: u32,
    pub uv_coords: Vec2,
}

impl TexturedVertex {
    pub fn new(position: Vec3, color: Color, uv_coords: Vec2) -> Self {
        Self {
            position,
            _padding1: 0,
            color,
            _padding2: 0,
            uv_coords,
        }
    }

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float2,
                },
            ],
        }
    }
}

impl From<(Vec3, Color, Vec2)> for TexturedVertex {
    fn from(vecs: (Vec3, Color, Vec2)) -> Self {
        Self {
            position: vecs.0,
            _padding1: 0,
            color: vecs.1,
            _padding2: 0,
            uv_coords: vecs.2,
        }
    }
}
