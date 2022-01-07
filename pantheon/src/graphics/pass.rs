use super::prelude::*;

#[derive(Debug)]
pub struct Pass<'a> {
    pub label: &'a str,

    /// Pipeline stuff
    pub pipeline_ctx: Option<PipelineContext<'a>>,
    pub pipelines: Vec<wgpu::RenderPipeline>,

    /// wgpu render pass stuff
    pub color_attachment_ops: Option<wgpu::Operations<wgpu::Color>>,
    pub color_attachment_view_handle: Option<TextureHandle<'a>>,
    pub depth_ops: Option<wgpu::Operations<f32>>,
    pub stencil_ops: Option<wgpu::Operations<u32>>,
    pub depth_stencil_view_handle: Option<TextureHandle<'a>>,

    /// Drawing stuff
    pub draw_call_handles: Vec<DrawCallHandle<'a>>,
    /// per https://github.com/gfx-rs/wgpu/wiki/Do%27s-and-Dont%27s#do-group-resource-bindings-by-the-change-frequency-start-from-the-lowest
    pub pass_bind_group_handle: Option<BindGroupHandle<'a>>,
    //pub bind_group_handle: Vec<BindGroupHandle<'a>>,
    pub vertex_buffer_handle: BufferHandle<'a>,
    pub index_buffer_handle: BufferHandle<'a>,
}
