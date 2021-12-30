use crate::graphics::prelude::*;
use crate::shader::ShaderContext;

pub struct ColorTarget<'a> {
    pub format_handle: Option<TextureHandle<'a>>,
    pub blend: Option<wgpu::BlendState>,
    pub write_mask: wgpu::ColorWrites,
}

pub struct PipelineContext<'a> {
    pub uniform_bind_group_layout_handles: Vec<BindGroupLayoutHandle<'a>>,
    pub vs_path: Option<&'a str>,
    pub fs_path: Option<&'a str>,
    pub vert_desc: fn() -> wgpu::VertexBufferLayout<'a>,
    pub label: Option<&'a str>,

    pub fragment_targets: Option<Vec<ColorTarget<'a>>>,
    pub primitive: wgpu::PrimitiveState,
    pub depth_stencil: Option<wgpu::DepthStencilState>,
    pub multisample: wgpu::MultisampleState,
    pub multiview: Option<core::num::NonZeroU32>,
}

impl<'a> PipelineContext<'a> {
    pub fn recreate_pipelines(
        &self,
        pipelines: &mut Vec<wgpu::RenderPipeline>,
        shader_ctx: &ShaderContext,
        layouts: &[&wgpu::BindGroupLayout],
        device: &wgpu::Device,
        targets: Option<&Vec<wgpu::ColorTargetState>>,
    ) {
        let non_fill_polygon_modes = device
            .features()
            .contains(wgpu::Features::POLYGON_MODE_LINE & wgpu::Features::POLYGON_MODE_POINT);
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: self.label,
                bind_group_layouts: layouts,
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX,
                    range: 0..(4 * 16),
                }],
            });

        Topology::iterator(non_fill_polygon_modes).for_each(|mode| {
            let fs_module;
            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: self.label,
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_ctx.make_module(device, &self.vs_path.unwrap()),
                    entry_point: "main",
                    buffers: &[(self.vert_desc)()],
                },
                fragment: if let Some(fs_path) = self.fs_path {
                    fs_module = shader_ctx.make_module(device, &fs_path);

                    Some(wgpu::FragmentState {
                        // 2.
                        module: &fs_module,
                        entry_point: "main",
                        targets: &targets.expect("nice one"),
                    })
                } else {
                    None
                },

                primitive: self.primitive,

                depth_stencil: self.depth_stencil.clone(),

                multisample: self.multisample,
                multiview: self.multiview,
            });

            if usize::from(*mode) != pipelines.len()
                && device.features().contains(
                    wgpu::Features::POLYGON_MODE_LINE & wgpu::Features::POLYGON_MODE_POINT,
                )
            {
                panic!("Render pipeline construction broke");
            }
            pipelines.push(pipeline);
        });
    }
}
