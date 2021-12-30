use super::prelude::*;

pub struct Pass<'a> {
    pub label: &'a str,

    /// Pipeline stuff
    pub pipeline_ctx: PipelineContext<'a>,
    pub pipelines: Vec<wgpu::RenderPipeline>,

    /// wgpu render pass stuff
    pub color_attachment_ops: Option<wgpu::Operations<wgpu::Color>>,
    pub color_attachment_view_handle: Option<TextureHandle<'a>>,
    pub depth_ops: Option<wgpu::Operations<f32>>,
    pub stencil_ops: Option<wgpu::Operations<u32>>,
    pub depth_stencil_view_handle: Option<TextureHandle<'a>>,

    /// Drawing stuff
    pub draw_call_handles: Vec<DrawCallHandle<'a>>,
    pub bind_group_handles: Option<Vec<BindGroupHandle<'a>>>,
    pub vertex_buffer_handle: BufferHandle<'a>,
    pub index_buffer_handle: BufferHandle<'a>,
}
