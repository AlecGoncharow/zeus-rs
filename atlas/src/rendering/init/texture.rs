use super::*;
use pantheon::graphics::prelude::*;
use pantheon::prelude::*;

/// @NOTE @DEBUG are you trying to reverse engineer how the textures are sampled because the
/// texture you are trying to sample is outputting garbage? Have you remembered to recreate the
/// bind group for the texture and sampler after the texture has been recreated on resize?
pub fn init_textured_resources<'a>(ctx: &mut Context<'a>, label: &'a str) {
    init_vert_index_buffers(ctx, label);

    let basic_textured_bind_group_layout =
        ctx.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some(label),
            });

    let depth_texture_bind_group_layout =
        ctx.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                        count: None,
                    },
                ],
                label: Some(label),
            });

    init_bind_group_for_textured_pass(
        ctx,
        &basic_textured_bind_group_layout,
        REFLECTION_TEXTURE,
        None,
    );
    init_bind_group_for_textured_pass(
        ctx,
        &basic_textured_bind_group_layout,
        REFRACTION_TEXTURE,
        None,
    );
    init_bind_group_for_textured_pass(
        ctx,
        &basic_textured_bind_group_layout,
        "refraction_depth",
        Some(&Texture::surface_texture_sampler(&ctx.device)),
    );

    let _basic_texured_bgl_handle = ctx
        .wrangler
        .add_bind_group_layout(basic_textured_bind_group_layout, label);
    let _handle = ctx
        .wrangler
        .add_bind_group_layout(depth_texture_bind_group_layout, "depth_sampler");
}

pub fn init_basic_textured_pass<'a>(ctx: &'a mut Context) {
    let pass_label = "basic_textured";
    init_textured_resources(ctx, pass_label);
    let bglh_basic_textured = ctx
        .wrangler
        .handle_to_bind_group_layout(pass_label)
        .unwrap();
    let vertex_buffer_handle = ctx.wrangler.handle_to_vertex_buffer(pass_label).unwrap();
    let index_buffer_handle = ctx.wrangler.handle_to_index_buffer(pass_label).unwrap();
    let push_constant_ranges = &[];

    let pipeline_ctx = PipelineContext {
        pass_bind_group_layout_handle: None,
        draw_call_bind_group_layout_handle: Some(bglh_basic_textured),
        push_constant_ranges,
        vs_path: Some("basic_textured.vert.spv"),
        fs_path: Some("basic_textured.frag.spv"),
        vert_desc: crate::vertex::BasicTexturedVertex::desc,
        label: Some(pass_label),
        fragment_targets: Some(vec![ColorTarget {
            format_handle: None,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        }]),
        primitive: wgpu::PrimitiveState {
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            conservative: false,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },

        multiview: None,
    };

    let pipelines = ctx.wrangler.create_pipelines(
        &ctx.device,
        &ctx.shader_context,
        &ctx.surface_config,
        &pipeline_ctx,
    );

    let color_attachment_ops = Some(wgpu::Operations {
        load: wgpu::LoadOp::Load,
        store: true,
    });

    let pass = Pass {
        label: pass_label,
        pipeline_ctx,
        pipelines,
        color_attachment_ops,
        color_attachment_view_handle: None,
        depth_ops: None,
        stencil_ops: None,
        depth_stencil_view_handle: None,
        pass_bind_group_handle: None,
        vertex_buffer_handle,
        index_buffer_handle,
    };

    let _handle = ctx.wrangler.add_pass(pass, pass_label);
}

fn init_bind_group_for_textured_pass<'a>(
    ctx: &mut Context<'a>,
    layout: &wgpu::BindGroupLayout,
    label: &'a str,
    sample_override: Option<&wgpu::Sampler>,
) {
    let texture = ctx.wrangler.find_texture(label);
    let sampler_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(
                    sample_override.unwrap_or(&texture.sampler),
                ),
            },
        ],
        label: Some(label),
    });

    let _handle = ctx
        .wrangler
        .add_surface_bound_bind_group(sampler_bind_group, label);
}
