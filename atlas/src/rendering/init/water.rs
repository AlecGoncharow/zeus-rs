use super::*;
use pantheon::graphics::prelude::*;
use pantheon::prelude::*;

pub const WATER: &'static str = "water";
pub const REFRACTION_DEPTH: &'static str = "refraction_depth";
pub const REFRACTION_CAMERA_CLIP: &'static str = "refraction_camera_clip";
pub const REFRACTION_TEXTURE: &'static str = "refraction_texture";
pub const REFLECTION_CAMERA_CLIP: &'static str = "reflection_camera_clip";
pub const REFLECTION_TEXTURE: &'static str = "reflection_texture";

pub fn init_refraction_resources<'a>(ctx: &mut Context<'a>) {
    let texture =
        Texture::create_surface_texture(&ctx.device, &ctx.surface_config, REFRACTION_TEXTURE);
    let depth_texture =
        Texture::create_depth_texture(&ctx.device, &ctx.surface_config, REFRACTION_DEPTH);

    let _handle = ctx.wrangler.add_texture(texture, REFRACTION_TEXTURE);
    let _handle = ctx.wrangler.add_texture(depth_texture, REFRACTION_DEPTH);
}

pub fn init_refraction_pass<'a>(ctx: &'a mut Context) {
    let pass_label = "refraction";
    let shaded_label = "shaded";
    init_refraction_resources(ctx);

    let depth_texture_handle = ctx.wrangler.handle_to_texture(REFRACTION_DEPTH).unwrap();
    let vertex_buffer_handle = ctx.wrangler.handle_to_vertex_buffer(shaded_label).unwrap();
    let index_buffer_handle = ctx.wrangler.handle_to_index_buffer(shaded_label).unwrap();

    let clip_plane_bind_group_layout = ctx.wrangler.handle_to_bind_group_layout(SHADED).unwrap();
    let clip_plane_bind_group = ctx
        .wrangler
        .handle_to_bind_group(REFRACTION_CAMERA_CLIP)
        .unwrap();

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
        vert_desc: crate::vertex::ShadedVertex::desc,
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

    let color_attachment_view_handle =
        Some(ctx.wrangler.handle_to_texture(REFRACTION_TEXTURE).unwrap());

    let pass = Pass {
        label: pass_label,
        pipeline_ctx,
        pipelines,
        color_attachment_ops,
        color_attachment_view_handle,
        depth_ops,
        stencil_ops: None,
        depth_stencil_view,
        pass_bind_group_handle,
        vertex_buffer_handle,
        index_buffer_handle,
    };

    let _handle = ctx.wrangler.add_pass(pass, pass_label);
}

pub fn init_reflection_resources<'a>(ctx: &mut Context<'a>) {
    let texture =
        Texture::create_surface_texture(&ctx.device, &ctx.surface_config, REFLECTION_TEXTURE);

    let _handle = ctx.wrangler.add_texture(texture, REFLECTION_TEXTURE);
}

pub fn init_reflection_pass<'a>(ctx: &'a mut Context) {
    let pass_label = "reflection";
    let shaded_label = "shaded";
    let depth_texture_handle = match ctx.wrangler.handle_to_texture("depth") {
        Some(handle) => handle,
        None => {
            init_entity_resources(ctx);
            ctx.wrangler.handle_to_texture("depth").unwrap()
        }
    };

    init_reflection_resources(ctx);
    let vertex_buffer_handle = ctx.wrangler.handle_to_vertex_buffer(shaded_label).unwrap();
    let index_buffer_handle = ctx.wrangler.handle_to_index_buffer(shaded_label).unwrap();

    let clip_plane_bind_group_layout = ctx.wrangler.handle_to_bind_group_layout(SHADED).unwrap();
    let clip_plane_bind_group = ctx
        .wrangler
        .handle_to_bind_group(REFLECTION_CAMERA_CLIP)
        .unwrap();

    let pass_bind_group_handle = Some(clip_plane_bind_group);

    let color_attachment_ops = Some(wgpu::Operations {
        load: wgpu::LoadOp::Clear(ctx.gfx_context.clear_color),
        store: true,
    });
    let color_attachment_view_handle =
        Some(ctx.wrangler.handle_to_texture(REFLECTION_TEXTURE).unwrap());

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
        color_attachment_view_handle,
        depth_ops,
        stencil_ops: None,
        depth_stencil_view,
        pass_bind_group_handle,
        vertex_buffer_handle,
        index_buffer_handle,
    };

    let _handle = ctx.wrangler.add_pass(pass, pass_label);
}

pub fn init_water_resources<'a>(ctx: &mut Context<'a>, label: &'a str) {
    init_vert_index_buffers(ctx, label);

    let texture_sampler_bind_group_layout =
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
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },

                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 7,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some(WATER),
            });

    let reflection = ctx.wrangler.find_texture(REFLECTION_TEXTURE);
    let refraction = ctx.wrangler.find_texture(REFRACTION_TEXTURE);
    let refraction_depth = ctx.wrangler.find_texture(REFRACTION_DEPTH);
    let camera_buffer = ctx.wrangler.find_uniform_buffer(CAMERA);
    let texture = ctx.wrangler.find_texture(GLOBAL_LIGHT_SHADOW);
    let sampler = Texture::shadow_texture_sampler(&ctx.device);
    let global_shadow_buffer = ctx.wrangler.find_uniform_buffer(GLOBAL_LIGHT_SHADOW);

    let texture_sampler_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &texture_sampler_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&reflection.view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&refraction.view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&refraction_depth.view),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::Sampler(&Texture::surface_texture_sampler(
                    &ctx.device,
                )),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: global_shadow_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            },
            wgpu::BindGroupEntry {
                binding: 7,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
        label: Some(WATER),
    });

    let _handle = ctx
        .wrangler
        .add_bind_group_layout(texture_sampler_bind_group_layout, WATER);
    let _handle = ctx
        .wrangler
        .add_surface_bound_bind_group(texture_sampler_bind_group, WATER);
}

pub fn init_water_pass<'a>(ctx: &mut Context<'a>) -> PassHandle<'a> {
    let pass_label = "water";
    let depth_texture_handle = match ctx.wrangler.handle_to_texture("depth") {
        Some(handle) => handle,
        None => {
            init_entity_resources(ctx);
            ctx.wrangler.handle_to_texture("depth").unwrap()
        }
    };

    init_water_resources(ctx, pass_label);
    let vertex_buffer_handle = ctx.wrangler.handle_to_vertex_buffer(pass_label).unwrap();
    let index_buffer_handle = ctx.wrangler.handle_to_index_buffer(pass_label).unwrap();

    let texture_sampler_bind_group_layout_handle =
        ctx.wrangler.handle_to_bind_group_layout(WATER).unwrap();
    let static_entity_bind_group_layout_handle = ctx
        .wrangler
        .handle_to_bind_group_layout(UNIFORM_BUFFER_VERTEX)
        .unwrap();
    let texture_sampler_bind_group_handle = ctx.wrangler.handle_to_bind_group(WATER).unwrap();

    let pass_bind_group_handle = Some(texture_sampler_bind_group_handle);

    let color_attachment_ops = Some(wgpu::Operations {
        load: wgpu::LoadOp::Load,
        store: true,
    });

    let depth_ops = Some(wgpu::Operations {
        load: wgpu::LoadOp::Load,
        store: true,
    });
    let depth_stencil_view = Some(ViewKind::Handle(depth_texture_handle));
    let push_constant_ranges = &[wgpu::PushConstantRange {
        stages: wgpu::ShaderStages::VERTEX,
        range: 0..16,
    }];

    let pipeline_ctx = PipelineContext {
        pass_bind_group_layout_handle: Some(texture_sampler_bind_group_layout_handle),
        draw_call_bind_group_layout_handle: Some(static_entity_bind_group_layout_handle),
        push_constant_ranges,
        vs_path: Some("water.vert.spv"),
        fs_path: Some("water.frag.spv"),
        vert_desc: crate::vertex::WaterVertex::desc,
        label: Some(pass_label),
        fragment_targets: Some(vec![ColorTarget {
            format_handle: None,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        }]),
        primitive: wgpu::PrimitiveState {
            strip_index_format: None,
            front_face: wgpu::FrontFace::Cw,
            cull_mode: None,
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
