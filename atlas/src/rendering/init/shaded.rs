use super::*;
use pantheon::graphics::prelude::*;
use pantheon::math::prelude::*;
use pantheon::prelude::*;
use wgpu::util::DeviceExt;

pub const SHADED: &'static str = "shaded";

pub fn init_shaded_resources<'a>(
    ctx: &mut Context<'a>,
    label: &'a str,
    water_height: f32,
    refraction_offset: f32,
) {
    init_vert_index_buffers(ctx, label);

    let no_clip = Vec4::new_from_one(0);
    // @NOTE this really ought to just be `-water_height` but the small offset allows for the
    // reflection to not have/not have as bad as a gap where it meets the terrain, this offset is
    // small enough that there should be minimal artifacting but still a bit of wiggle room for
    // lower waves
    let reflection_clip = Vec4::new(0., 1, 0., -water_height + 0.1);
    let refraction_clip = Vec4::new(0., -1, 0., water_height + refraction_offset);

    let shaded_clip_plane_uniform_buffer =
        ctx.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("shaded clip plane Uniform Buffer"),
                usage: wgpu::BufferUsages::UNIFORM,
                contents: bytemuck::cast_slice(&[no_clip]),
            });
    let reflection_clip_plane_uniform_buffer =
        ctx.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("shaded clip plane Uniform Buffer"),
                usage: wgpu::BufferUsages::UNIFORM,
                contents: bytemuck::cast_slice(&[reflection_clip]),
            });
    let refraction_clip_plane_uniform_buffer =
        ctx.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("shaded clip plane Uniform Buffer"),
                usage: wgpu::BufferUsages::UNIFORM,
                contents: bytemuck::cast_slice(&[refraction_clip]),
            });

    let camera_buffer = ctx.wrangler.find_uniform_buffer(CAMERA);
    let camera_reflect_buffer = ctx.wrangler.find_uniform_buffer(CAMERA_REFLECT);

    let shaded_bind_group_layout =
        ctx.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },

                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },

                        count: None,
                    },
                ],
                label: Some(SHADED),
            });

    let shaded_clip_plane_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &shaded_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: shaded_clip_plane_uniform_buffer.as_entire_binding(),
            },
        ],
        label: Some("shaded Bind Group"),
    });

    let reflection_clip_plane_bind_group =
        ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &shaded_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_reflect_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: reflection_clip_plane_uniform_buffer.as_entire_binding(),
                },
            ],
            label: Some("reflect Bind Group"),
        });

    let refraction_clip_plane_bind_group =
        ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &shaded_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: refraction_clip_plane_uniform_buffer.as_entire_binding(),
                },
            ],
            label: Some("refraction Bind Group"),
        });

    let _handle = ctx
        .wrangler
        .add_uniform_buffer(shaded_clip_plane_uniform_buffer, SHADED);
    let _handle = ctx
        .wrangler
        .add_uniform_buffer(reflection_clip_plane_uniform_buffer, REFLECTION_CAMERA_CLIP);
    let _handle = ctx
        .wrangler
        .add_uniform_buffer(refraction_clip_plane_uniform_buffer, REFRACTION_CAMERA_CLIP);

    let _handle = ctx
        .wrangler
        .add_bind_group(shaded_clip_plane_bind_group, SHADED);
    let _handle = ctx
        .wrangler
        .add_bind_group(reflection_clip_plane_bind_group, REFLECTION_CAMERA_CLIP);
    let _handle = ctx
        .wrangler
        .add_bind_group(refraction_clip_plane_bind_group, REFRACTION_CAMERA_CLIP);

    let _handle = ctx
        .wrangler
        .add_bind_group_layout(shaded_bind_group_layout, SHADED);
}

pub fn init_shaded_pass<'a>(ctx: &'a mut Context) -> PassHandle<'a> {
    let pass_label = "shaded";
    let depth_texture_handle = match ctx.wrangler.handle_to_texture("depth") {
        Some(handle) => handle,
        None => {
            init_entity_resources(ctx);
            ctx.wrangler.handle_to_texture("depth").unwrap()
        }
    };

    let vertex_buffer_handle = ctx.wrangler.handle_to_vertex_buffer(pass_label).unwrap();
    let index_buffer_handle = ctx.wrangler.handle_to_index_buffer(pass_label).unwrap();

    let clip_plane_bind_group_layout = ctx.wrangler.handle_to_bind_group_layout(SHADED).unwrap();
    let clip_plane_bind_group = ctx.wrangler.handle_to_bind_group(SHADED).unwrap();

    let pass_bind_group_handle = Some(clip_plane_bind_group);

    let color_attachment_ops = Some(wgpu::Operations {
        load: wgpu::LoadOp::Clear(ctx.gfx_context.clear_color),
        store: true,
    });

    let depth_ops = Some(wgpu::Operations {
        load: wgpu::LoadOp::Clear(DEPTH_CLEAR),
        store: true,
    });
    let depth_stencil_view = Some(ViewKind::Handle(depth_texture_handle));

    let push_constant_ranges = &[wgpu::PushConstantRange {
        stages: wgpu::ShaderStages::VERTEX,
        range: 0..16 * 4,
    }];

    let pipeline_ctx = PipelineContext {
        pass_bind_group_layout_handle: Some(clip_plane_bind_group_layout),
        draw_call_bind_group_layout_handle: None,
        push_constant_ranges,
        vs_path: Some("shaded.vert.spv"),
        fs_path: Some("shaded.frag.spv"),
        vert_desc: vertex::ShadedVertex::desc,
        label: Some(pass_label),
        fragment_targets: Some(vec![ColorTarget {
            format_handle: None,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        }]),
        primitive: wgpu::PrimitiveState {
            strip_index_format: None,
            front_face: wgpu::FrontFace::Cw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            conservative: false,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: pantheon::graphics::texture::Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Greater,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
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

    let pass = Pass {
        label: pass_label,
        pipeline_ctx,
        pipelines,
        color_attachment_ops,
        color_attachment_view_handle: None,
        depth_ops,
        stencil_ops: None,
        depth_stencil_view,
        pass_bind_group_handle,
        vertex_buffer_handle,
        index_buffer_handle,
    };

    let handle = ctx.wrangler.add_pass(pass, pass_label);
    handle
}
