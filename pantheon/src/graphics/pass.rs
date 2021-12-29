use super::prelude::*;
use crate::shader::ShaderContext;

pub struct PipelineContext<'a> {
    pub uniform_bind_group_layout_handles: Vec<BindGroupLayoutHandle<'a>>,
    pub vs_path: Option<&'a str>,
    pub fs_path: Option<&'a str>,
    pub vert_desc: fn() -> wgpu::VertexBufferLayout<'a>,
    pub label: Option<&'a str>,
}

// @TODO think about this a bit more, so much context is already being provided, we could just
// store all the necessary context to generically rebuild pipelines against some function
pub type RecreatePipelines<'a> = fn(
    &mut Vec<wgpu::RenderPipeline>,
    &PipelineContext,
    &ShaderContext,
    &[&wgpu::BindGroupLayout],
    &wgpu::Device,
    &wgpu::SurfaceConfiguration,
);

pub struct Pass<'a> {
    pub label: &'a str,

    /// Pipeline stuff
    pub pipeline_ctx: PipelineContext<'a>,
    pub recreate_pipelines: RecreatePipelines<'a>,
    pub pipelines: Vec<wgpu::RenderPipeline>,

    /// wgpu render pass stuff
    pub color_attachment_ops: Option<wgpu::Operations<wgpu::Color>>,
    pub color_attachment_view_handle: Option<TextureHandle<'a>>,
    pub depth_ops: Option<wgpu::Operations<f32>>,
    pub stencil_ops: Option<wgpu::Operations<u32>>,
    pub depth_stencil_view_handle: Option<TextureHandle<'a>>,

    /// Drawing stuff
    pub draw_call_handles: Vec<DrawCallHandle<'a>>,
    pub bind_group_handles: Vec<BindGroupHandle<'a>>,
    pub vertex_buffer_handle: BufferHandle<'a>,
    pub index_buffer_handle: BufferHandle<'a>,
}
