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
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub normal: [f32; 3],
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
            position: [vecs.0.x, vecs.0.y, vecs.0.z],
            color: [vecs.1.r, vecs.1.g, vecs.1.b, vecs.1.a],
            normal: [vecs.2.x, vecs.2.y, vecs.2.z],
        }
    }
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
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
            position: [vecs.0.x, vecs.0.y, vecs.0.z],
            color: [vecs.1.r, vecs.1.g, vecs.1.b, vecs.1.a],
        }
    }
}
