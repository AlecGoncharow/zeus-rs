use super::handles::TextureHandle;
use super::mode::MAX_PIPELINES;
use super::prelude::*;

#[derive(Debug)]
pub enum ViewKind<'a> {
    Handle(TextureHandle<'a>),
    View(wgpu::TextureView),
}

/// corresponding to https://docs.rs/wgpu/0.12.0/wgpu/struct.RenderPass.html#method.set_viewport
#[derive(Debug)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}

impl Viewport {
    pub fn new(x: f32, y: f32, w: f32, h: f32, min_depth: f32, max_depth: f32) -> Self {
        Self {
            x,
            y,
            w,
            h,
            min_depth,
            max_depth,
        }
    }
}

#[derive(Debug)]
pub struct Pass<'a> {
    pub label: &'a str,

    /// Pipeline stuff
    pub pipeline_ctx: PipelineContext<'a>,
    pub pipelines: [wgpu::RenderPipeline; MAX_PIPELINES],

    /// wgpu render pass stuff
    pub color_attachment_ops: Option<wgpu::Operations<wgpu::Color>>,
    pub color_attachment_view_handle: Option<TextureHandle<'a>>,
    pub depth_ops: Option<wgpu::Operations<f32>>,
    pub stencil_ops: Option<wgpu::Operations<u32>>,
    pub depth_stencil_view: Option<ViewKind<'a>>,
    pub viewport: Option<Viewport>,

    /// per https://github.com/gfx-rs/wgpu/wiki/Do%27s-and-Dont%27s#do-group-resource-bindings-by-the-change-frequency-start-from-the-lowest
    pub pass_bind_group_handle: Option<BindGroupHandle<'a>>,
    /// This is required for switching to allowing post shadow bake passes to use same bind group 0
    /// for global light and shadow stuff
    pub frame_bind_group_handle_override: Option<BindGroupHandle<'a>>,
    //pub bind_group_handle: Vec<BindGroupHandle<'a>>,
    pub vertex_buffer_handle: BufferHandle<'a>,
    pub index_buffer_handle: BufferHandle<'a>,
}

/*
impl<'a> Default for Pass<'a> {
    fn default() -> Self {
        Self {
            label: Default::default(),
            pipeline_ctx: Default::default(),
            pipelines: [MaybeUninit::uninit(); 15],
            color_attachment_ops: Default::default(),
            color_attachment_view_handle: Default::default(),
            depth_ops: Default::default(),
            stencil_ops: Default::default(),
            depth_stencil_view_handle: Default::default(),
            pass_bind_group_handle: Default::default(),
            vertex_buffer_handle: BufferHandle {
                label: UNINIT,
                idx: usize::MAX,
                marker: PhantomData,
            },
            index_buffer_handle: BufferHandle {
                label: UNINIT,
                idx: usize::MAX,
                marker: PhantomData,
            },
        }
    }
}
*/
