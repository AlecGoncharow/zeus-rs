pub mod color;
pub mod mesh;
pub mod mode;
pub mod pass;
pub mod pipeline;
pub mod renderer;
pub mod texture;
pub mod vertex;
pub mod wrangler;

mod common {
    use super::handles::{BindGroupHandle, PassHandle};
    use super::{PolygonMode, Topology};
    use core::ops::Range;
    use std::marker::PhantomData;
    #[derive(Clone, Copy, Debug)]
    pub struct LabeledEntryHandle<'a, T> {
        pub label: &'a str,
        pub idx: usize,
        pub(crate) marker: PhantomData<T>,
    }

    pub struct LabeledEntry<'a, T> {
        pub label: &'a str,
        pub entry: T,
    }

    #[derive(Debug)]
    pub struct PushConstant {
        pub stages: wgpu::ShaderStages,
        pub offset: u32,
        pub data: Vec<u8>,
    }

    pub type PassBundle<'a> = Vec<PassHandle<'a>>;

    impl PushConstant {
        pub fn vertex_data<T>(offset: u32, data: &[T]) -> Self
        where
            T: bytemuck::Pod,
        {
            Self {
                stages: wgpu::ShaderStages::VERTEX,
                offset,
                data: Vec::from(bytemuck::cast_slice(data)),
            }
        }

        pub fn replace_data<T>(&mut self, data: &[T])
        where
            T: bytemuck::Pod,
        {
            self.data.clear();
            self.data.extend_from_slice(bytemuck::cast_slice(data));
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub enum DrawCallKind {
        Vertex,
        Indexed(i32),
    }
    // @TODO Range doesn't impl Copy, need to think about how best to approach this, do we clone the
    // Range on draw or do we keep the params necessary to construct on the fly, is one faster than
    // another?
    // See: https://github.com/rust-lang/rust/pull/27186
    #[derive(Debug)]
    pub struct DrawCall<'a> {
        pub kind: DrawCallKind,
        /// this should really be typed against the generated bitflags struct but I'm lazy
        pub pass_flags: usize,
        pub index_range: Range<u32>,
        pub instances: Range<u32>,
        pub push_constant: Option<PushConstant>,
        pub topology: Topology,
        pub bind_group_handle: Option<BindGroupHandle<'a>>,
    }

    impl<'a> DrawCall<'a> {
        pub fn default() -> Self {
            Self {
                kind: DrawCallKind::Vertex,
                pass_flags: 0,
                index_range: 0..0,
                instances: 0..1,
                push_constant: None,
                topology: Topology::TriangleList(PolygonMode::Fill),
                bind_group_handle: None,
            }
        }

        pub fn set_topology(&mut self, new_topology: Topology) {
            self.topology = new_topology;
        }

        pub fn set_push_constant_data<T>(&mut self, data: &[T])
        where
            T: bytemuck::Pod,
        {
            self.push_constant.as_mut().unwrap().replace_data(data);
        }
    }
}

pub mod handles {
    use super::common::*;
    pub use super::pass::Pass;
    pub use super::texture::Texture;
    use crate::prelude::*;
    pub type BufferHandle<'a> = LabeledEntryHandle<'a, &'a wgpu::Buffer>;
    pub type BufferAddressHandle<'a> = LabeledEntryHandle<'a, &'a wgpu::BufferAddress>;
    pub type BindGroupHandle<'a> = LabeledEntryHandle<'a, &'a wgpu::BindGroup>;
    pub type BindGroupLayoutHandle<'a> = LabeledEntryHandle<'a, &'a wgpu::BindGroupLayout>;
    pub type TextureHandle<'a> = LabeledEntryHandle<'a, &'a Texture>;
    pub type PassHandle<'a> = LabeledEntryHandle<'a, &'a Pass<'a>>;
    pub type PassBundleHandle<'a> = LabeledEntryHandle<'a, PassBundle<'a>>;

    pub type DrawCallHandle<'a> = LabeledEntryHandle<'a, &'a DrawCall<'a>>;

    impl<'a> DrawCallHandle<'a> {
        pub fn set_push_constant_data<T>(&self, ctx: &mut Context<'a>, data: &[T])
        where
            T: bytemuck::Pod,
        {
            let draw_call = ctx.wrangler.get_draw_call_mut(self);

            draw_call.set_push_constant_data(data);
        }

        pub fn set_topology(&self, ctx: &mut Context<'a>, topology: super::Topology) {
            let draw_call = ctx.wrangler.get_draw_call_mut(self);

            draw_call.set_topology(topology);
        }
    }
}

pub mod prelude {

    pub use super::common::*;
    pub use super::handles::*;

    pub use super::color::Color;
    pub use super::mesh::Mesh;
    pub use super::mode::{PolygonMode, Topology};
    pub use super::pass::Pass;
    pub use super::pipeline::{ColorTarget, PipelineContext};
    pub use super::texture::Texture;
    pub use super::wrangler::RenderWrangler;
}

pub use color::Color;
pub use mode::DrawMode;
pub use mode::PolygonMode;
pub use mode::Topology;

use crate::math::Mat4;
use crate::math::Vec3;

pub trait CameraProjection {
    fn projection_view_matrix(&self) -> Mat4;
    fn projection_matrix(&self) -> Mat4;
    fn view_matrix(&self) -> Mat4;
}

pub struct DefaultCamera {}

impl CameraProjection for DefaultCamera {
    fn projection_view_matrix(&self) -> Mat4 {
        Mat4::identity()
    }
    fn projection_matrix(&self) -> Mat4 {
        Mat4::identity()
    }
    fn view_matrix(&self) -> Mat4 {
        Mat4::identity()
    }
}

pub trait Drawable {
    /// R*T Matrix to translate model from model space to world space
    fn model_matrix(&self) -> Mat4 {
        Mat4::identity()
    }

    fn draw_mode(&self) -> DrawMode {
        DrawMode::Normal(Topology::TriangleList(PolygonMode::Fill))
    }

    fn rotate(&mut self, _theta: f32, _axis: Vec3) {}
    fn translate(&mut self, (_x_tr, _y_tr, _z_tr): (f32, f32, f32)) {}
}
