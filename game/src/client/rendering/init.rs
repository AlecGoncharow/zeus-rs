use pantheon::graphics::prelude::*;
use pantheon::math::prelude::*;
use pantheon::prelude::*;
use wgpu::util::DeviceExt;

const CAMERA_UNIFORM_BUFFER_SIZE: wgpu::BufferAddress = 2 * 16 * 4 + 4 * 3 + 4 * 2 + 12;

// @TODO FIXME this is arbitrary
const VERTEX_BUFFER_SIZE: wgpu::BufferAddress = ((3 + 4 + 3) * 4 * 3) * 200_000;
// @TODO FIXME this is arbitrary
const INDEX_BUFFER_SIZE: wgpu::BufferAddress = 4 * 2_000_000;

pub fn init_camera_resources(ctx: &mut Context) {
    let camera_bind_group_layout =
        ctx.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },

                    count: None,
                }],
                label: Some("camera uniform bind group layout"),
            });

    let camera_uniform_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Camera Uniform Buffer"),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
        size: CAMERA_UNIFORM_BUFFER_SIZE,
    });

    let camera_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &camera_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: camera_uniform_buffer.as_entire_binding(),
        }],
        label: Some("Camera Bind Group"),
    });

    let reflect_camera_uniform_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Reflect Camera Uniform Buffer"),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
        size: CAMERA_UNIFORM_BUFFER_SIZE,
    });

    let reflect_camera_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &camera_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: reflect_camera_uniform_buffer.as_entire_binding(),
        }],
        label: Some("Reflect Camera Bind Group"),
    });

    let _camera_bind_group_layout_handle = ctx
        .wrangler
        .add_bind_group_layout(camera_bind_group_layout, "camera");

    let _camera_bind_group_handle = ctx.wrangler.add_bind_group(camera_bind_group, "camera");

    let _camera_uniform_buffer = ctx
        .wrangler
        .add_uniform_buffer(camera_uniform_buffer, "camera");

    let _reflect_camera_bind_group_handle = ctx
        .wrangler
        .add_bind_group(reflect_camera_bind_group, "camera_reflect");

    let _reflect_camera_uniform_buffer = ctx
        .wrangler
        .add_uniform_buffer(reflect_camera_uniform_buffer, "camera_reflect");
}

pub fn init_entity_resources(ctx: &mut Context) {
    let depth_texture = Texture::create_depth_texture(&ctx.device, &ctx.surface_config, "depth");

    let _depth_texture_handle = ctx.wrangler.add_texture(depth_texture, "depth");
}

pub fn init_shaded_resources<'a>(
    ctx: &mut Context<'a>,
    label: &'a str,
    water_height: f32,
    refraction_offset: f32,
) {
    init_vert_index_buffers(ctx, label);

    let no_clip = Vec4::new_from_one(0);
    // @TODO FIXME thin matrix might have different coordinate system
    let reflection_clip = Vec4::new(0., 1, 0., -water_height);
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

    let clip_plane_bind_group_layout =
        ctx.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },

                    count: None,
                }],
                label: Some("clip planebind group layout"),
            });

    let shaded_clip_plane_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &clip_plane_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: shaded_clip_plane_uniform_buffer.as_entire_binding(),
        }],
        label: Some("shaded clip plane Bind Group"),
    });

    let reflection_clip_plane_bind_group =
        ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &clip_plane_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: reflection_clip_plane_uniform_buffer.as_entire_binding(),
            }],
            label: Some("Reflect clip plane Bind Group"),
        });

    let refraction_clip_plane_bind_group =
        ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &clip_plane_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: refraction_clip_plane_uniform_buffer.as_entire_binding(),
            }],
            label: Some("Refraction clip plane Bind Group"),
        });

    let _handle = ctx
        .wrangler
        .add_uniform_buffer(shaded_clip_plane_uniform_buffer, "shaded_clip_plane");
    let _handle = ctx.wrangler.add_uniform_buffer(
        reflection_clip_plane_uniform_buffer,
        "reflection_clip_plane",
    );
    let _handle = ctx.wrangler.add_uniform_buffer(
        refraction_clip_plane_uniform_buffer,
        "refraction_clip_plane",
    );
    let _handle = ctx
        .wrangler
        .add_bind_group_layout(clip_plane_bind_group_layout, "clip_plane");
    let _handle = ctx
        .wrangler
        .add_bind_group(shaded_clip_plane_bind_group, "shaded_clip_plane");
    let _handle = ctx
        .wrangler
        .add_bind_group(reflection_clip_plane_bind_group, "reflection_clip_plane");
    let _handle = ctx
        .wrangler
        .add_bind_group(refraction_clip_plane_bind_group, "refraction_clip_plane");
}

/// :^)
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

    let camera_bind_group_layout_handle = match ctx.wrangler.handle_to_bind_group_layout("camera") {
        Some(handle) => handle,
        None => {
            init_camera_resources(ctx);
            ctx.wrangler.handle_to_bind_group_layout("camera").unwrap()
        }
    };
    let camera_bind_group_handle = ctx.wrangler.handle_to_bind_group("camera").unwrap();

    let clip_plane_bind_group_layout = ctx
        .wrangler
        .handle_to_bind_group_layout("clip_plane")
        .unwrap();
    let clip_plane_bind_group = ctx
        .wrangler
        .handle_to_bind_group("shaded_clip_plane")
        .unwrap();

    let bind_group_handles = Some(vec![camera_bind_group_handle, clip_plane_bind_group]);

    let color_attachment_ops = Some(wgpu::Operations {
        load: wgpu::LoadOp::Clear(ctx.gfx_context.clear_color),
        store: true,
    });

    let depth_ops = Some(wgpu::Operations {
        load: wgpu::LoadOp::Clear(1.0),
        store: true,
    });
    let depth_stencil_view_handle = Some(depth_texture_handle);

    let pipeline_ctx = Some(PipelineContext {
        uniform_bind_group_layout_handles: vec![
            camera_bind_group_layout_handle,
            clip_plane_bind_group_layout,
        ],
        vs_path: Some("shaded.vert.spv"),
        fs_path: Some("shaded.frag.spv"),
        vert_desc: crate::base::vertex::ShadedVertex::desc,
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
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },

        multiview: None,
    });

    let pipelines = Vec::new();

    let pass = Pass {
        label: pass_label,
        pipeline_ctx,
        pipelines,
        color_attachment_ops,
        color_attachment_view_handle: None,
        depth_ops,
        stencil_ops: None,
        depth_stencil_view_handle,
        draw_call_handles: Vec::new(),
        bind_group_handles,
        vertex_buffer_handle,
        index_buffer_handle,
    };

    let handle = ctx.wrangler.add_pass(pass, pass_label);
    ctx.wrangler
        .reload_shaders(&ctx.device, &ctx.shader_context, &ctx.surface_config);
    handle
}

pub fn init_refraction_resources<'a>(ctx: &mut Context<'a>, label: &'a str) {
    let texture = Texture::create_surface_texture(&ctx.device, &ctx.surface_config, label);
    let depth_texture =
        Texture::create_depth_texture(&ctx.device, &ctx.surface_config, "refraction_depth");

    let _handle = ctx.wrangler.add_texture(texture, label);
    let _handle = ctx.wrangler.add_texture(depth_texture, "refraction_depth");
}

pub fn init_refraction_pass<'a>(ctx: &'a mut Context) {
    let pass_label = "refraction";
    let shaded_label = "shaded";
    init_refraction_resources(ctx, pass_label);

    let depth_texture_handle = ctx.wrangler.handle_to_texture("refraction_depth").unwrap();
    let vertex_buffer_handle = ctx.wrangler.handle_to_vertex_buffer(shaded_label).unwrap();
    let index_buffer_handle = ctx.wrangler.handle_to_index_buffer(shaded_label).unwrap();

    let camera_bind_group_layout_handle = match ctx.wrangler.handle_to_bind_group_layout("camera") {
        Some(handle) => handle,
        None => {
            init_camera_resources(ctx);
            ctx.wrangler.handle_to_bind_group_layout("camera").unwrap()
        }
    };
    let camera_bind_group_handle = ctx.wrangler.handle_to_bind_group("camera").unwrap();

    let clip_plane_bind_group_layout = ctx
        .wrangler
        .handle_to_bind_group_layout("clip_plane")
        .unwrap();
    let clip_plane_bind_group = ctx
        .wrangler
        .handle_to_bind_group("refraction_clip_plane")
        .unwrap();

    let bind_group_handles = Some(vec![camera_bind_group_handle, clip_plane_bind_group]);

    let color_attachment_ops = Some(wgpu::Operations {
        load: wgpu::LoadOp::Clear(ctx.gfx_context.clear_color),
        store: true,
    });

    let depth_ops = Some(wgpu::Operations {
        load: wgpu::LoadOp::Clear(1.0),
        store: true,
    });
    let depth_stencil_view_handle = Some(depth_texture_handle);

    let pipeline_ctx = Some(PipelineContext {
        uniform_bind_group_layout_handles: vec![
            camera_bind_group_layout_handle,
            clip_plane_bind_group_layout,
        ],
        vs_path: Some("shaded.vert.spv"),
        fs_path: Some("shaded.frag.spv"),
        vert_desc: crate::base::vertex::ShadedVertex::desc,
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
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },

        multiview: None,
    });

    let pipelines = Vec::new();

    let color_attachment_view_handle = Some(ctx.wrangler.handle_to_texture(pass_label).unwrap());

    let pass = Pass {
        label: pass_label,
        pipeline_ctx,
        pipelines,
        color_attachment_ops,
        color_attachment_view_handle,
        depth_ops,
        stencil_ops: None,
        depth_stencil_view_handle,
        draw_call_handles: Vec::new(),
        bind_group_handles,
        vertex_buffer_handle,
        index_buffer_handle,
    };

    let _handle = ctx.wrangler.add_pass(pass, pass_label);
    ctx.wrangler
        .reload_shaders(&ctx.device, &ctx.shader_context, &ctx.surface_config);
}

pub fn init_reflection_resources<'a>(ctx: &mut Context<'a>, label: &'a str) {
    let texture = Texture::create_surface_texture(&ctx.device, &ctx.surface_config, label);

    let _handle = ctx.wrangler.add_texture(texture, label);
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

    init_reflection_resources(ctx, pass_label);
    let vertex_buffer_handle = ctx.wrangler.handle_to_vertex_buffer(shaded_label).unwrap();
    let index_buffer_handle = ctx.wrangler.handle_to_index_buffer(shaded_label).unwrap();

    let camera_bind_group_layout_handle = match ctx.wrangler.handle_to_bind_group_layout("camera") {
        Some(handle) => handle,
        None => {
            init_camera_resources(ctx);
            ctx.wrangler.handle_to_bind_group_layout("camera").unwrap()
        }
    };
    let camera_bind_group_handle = ctx.wrangler.handle_to_bind_group("camera_reflect").unwrap();

    let clip_plane_bind_group_layout = ctx
        .wrangler
        .handle_to_bind_group_layout("clip_plane")
        .unwrap();
    let clip_plane_bind_group = ctx
        .wrangler
        .handle_to_bind_group("reflection_clip_plane")
        .unwrap();

    let bind_group_handles = Some(vec![camera_bind_group_handle, clip_plane_bind_group]);

    let color_attachment_ops = Some(wgpu::Operations {
        load: wgpu::LoadOp::Clear(ctx.gfx_context.clear_color),
        store: true,
    });

    let depth_ops = Some(wgpu::Operations {
        load: wgpu::LoadOp::Clear(1.0),
        store: true,
    });
    let depth_stencil_view_handle = Some(depth_texture_handle);

    let pipeline_ctx = Some(PipelineContext {
        uniform_bind_group_layout_handles: vec![
            camera_bind_group_layout_handle,
            clip_plane_bind_group_layout,
        ],
        vs_path: Some("shaded.vert.spv"),
        fs_path: Some("shaded.frag.spv"),
        vert_desc: crate::base::vertex::ShadedVertex::desc,
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
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },

        multiview: None,
    });

    let pipelines = Vec::new();

    let color_attachment_view_handle = Some(ctx.wrangler.handle_to_texture(pass_label).unwrap());

    let pass = Pass {
        label: pass_label,
        pipeline_ctx,
        pipelines,
        color_attachment_ops,
        color_attachment_view_handle,
        depth_ops,
        stencil_ops: None,
        depth_stencil_view_handle,
        draw_call_handles: Vec::new(),
        bind_group_handles,
        vertex_buffer_handle,
        index_buffer_handle,
    };

    let _handle = ctx.wrangler.add_pass(pass, pass_label);
    ctx.wrangler
        .reload_shaders(&ctx.device, &ctx.shader_context, &ctx.surface_config);
}

pub fn init_water_resources<'a>(ctx: &mut Context<'a>, label: &'a str) {
    init_vert_index_buffers(ctx, label);
}

pub fn init_water_pass<'a>(ctx: &'a mut Context) -> PassHandle<'a> {
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

    let camera_bind_group_layout_handle = match ctx.wrangler.handle_to_bind_group_layout("camera") {
        Some(handle) => handle,
        None => {
            init_camera_resources(ctx);
            ctx.wrangler.handle_to_bind_group_layout("camera").unwrap()
        }
    };
    let reflection_bind_group_layout_handle =
        match ctx.wrangler.handle_to_bind_group_layout("basic_textured") {
            Some(handle) => handle,
            None => {
                init_textured_resources(ctx, "basic_textured");
                ctx.wrangler
                    .handle_to_bind_group_layout("basic_textured")
                    .unwrap()
            }
        };
    let refraction_bind_group_layout_handle = ctx
        .wrangler
        .handle_to_bind_group_layout("basic_textured")
        .unwrap();
    let depth_bind_group_layout_handle = ctx
        .wrangler
        .handle_to_bind_group_layout("basic_textured")
        .unwrap();

    let camera_bind_group_handle = ctx.wrangler.handle_to_bind_group("camera").unwrap();
    let reflection_bind_group_handle = ctx.wrangler.handle_to_bind_group("reflection").unwrap();
    let refraction_bind_group_handle = ctx.wrangler.handle_to_bind_group("refraction").unwrap();
    let depth_bind_group_handle = ctx
        .wrangler
        .handle_to_bind_group("refraction_depth")
        .unwrap();

    let bind_group_handles = Some(vec![
        camera_bind_group_handle,
        reflection_bind_group_handle,
        refraction_bind_group_handle,
        depth_bind_group_handle,
    ]);

    let color_attachment_ops = Some(wgpu::Operations {
        load: wgpu::LoadOp::Load,
        store: true,
    });

    let depth_ops = Some(wgpu::Operations {
        load: wgpu::LoadOp::Load,
        store: true,
    });
    let depth_stencil_view_handle = Some(depth_texture_handle);

    let pipeline_ctx = Some(PipelineContext {
        uniform_bind_group_layout_handles: vec![
            camera_bind_group_layout_handle,
            reflection_bind_group_layout_handle,
            refraction_bind_group_layout_handle,
            depth_bind_group_layout_handle,
        ],
        vs_path: Some("water.vert.spv"),
        fs_path: Some("water.frag.spv"),
        vert_desc: crate::base::vertex::WaterVertex::desc,
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
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },

        multiview: None,
    });

    let pipelines = Vec::new();

    let pass = Pass {
        label: pass_label,
        pipeline_ctx,
        pipelines,
        color_attachment_ops,
        color_attachment_view_handle: None,
        depth_ops,
        stencil_ops: None,
        depth_stencil_view_handle,
        draw_call_handles: Vec::new(),
        bind_group_handles,
        vertex_buffer_handle,
        index_buffer_handle,
    };

    let handle = ctx.wrangler.add_pass(pass, pass_label);
    ctx.wrangler
        .reload_shaders(&ctx.device, &ctx.shader_context, &ctx.surface_config);
    handle
}

/// not useful
pub fn init_clone_resources<'a>(ctx: &mut Context<'a>, label: &'a str) {
    let size = wgpu::Extent3d {
        width: ctx.surface_config.width,
        height: ctx.surface_config.height,
        depth_or_array_layers: 1,
    };
    let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: Texture::IMAGE_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST,
    });

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let mirror_texture = Texture {
        texture,
        view,
        sampler,
        format: Texture::IMAGE_FORMAT,
    };

    let _handle = ctx.wrangler.add_texture(mirror_texture, label);
}

/// not useful
pub fn init_clone_pass<'a>(ctx: &mut Context<'a>) {
    let pass_label = "clone";
    init_clone_resources(ctx, pass_label);

    let color_attachment_ops = Some(wgpu::Operations {
        load: wgpu::LoadOp::Clear(ctx.gfx_context.clear_color),
        store: true,
    });

    let color_attachment_view_handle = Some(ctx.wrangler.handle_to_texture(pass_label).unwrap());

    let pass = Pass {
        label: pass_label,
        pipeline_ctx: None,
        pipelines: Vec::new(),
        color_attachment_ops,
        color_attachment_view_handle,
        depth_ops: None,
        stencil_ops: None,
        depth_stencil_view_handle: None,
        draw_call_handles: Vec::new(),
        bind_group_handles: None,
        vertex_buffer_handle: ctx.wrangler.handle_to_vertex_buffer("shaded").unwrap(),
        index_buffer_handle: ctx.wrangler.handle_to_index_buffer("shaded").unwrap(),
    };

    let _handle = ctx.wrangler.add_pass(pass, pass_label);
}

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

    init_bind_group_for_textured_pass(ctx, &basic_textured_bind_group_layout, "reflection", None);
    init_bind_group_for_textured_pass(ctx, &basic_textured_bind_group_layout, "refraction", None);
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

    let pipeline_ctx = Some(PipelineContext {
        uniform_bind_group_layout_handles: vec![bglh_basic_textured],
        vs_path: Some("basic_textured.vert.spv"),
        fs_path: Some("basic_textured.frag.spv"),
        vert_desc: crate::base::vertex::BasicTexturedVertex::desc,
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
    });

    let pipelines = Vec::new();

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
        draw_call_handles: Vec::new(),
        bind_group_handles: None,
        vertex_buffer_handle,
        index_buffer_handle,
    };

    let _handle = ctx.wrangler.add_pass(pass, pass_label);
    ctx.wrangler
        .reload_shaders(&ctx.device, &ctx.shader_context, &ctx.surface_config);
}

fn init_vert_index_buffers<'a>(ctx: &mut Context<'a>, label: &'a str) {
    let shaded_vertex_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(&format!("{} vertex buffer", label)),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
        size: VERTEX_BUFFER_SIZE,
    });

    let shaded_index_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(&format!("{} index buffer", label)),
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
        size: INDEX_BUFFER_SIZE,
    });

    let _vertex_buffer_handle = ctx.wrangler.add_vertex_buffer(shaded_vertex_buffer, label);
    let _index_buffer_handle = ctx.wrangler.add_index_buffer(shaded_index_buffer, label);
}

#[allow(dead_code)]
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

    let _handle = ctx.wrangler.add_bind_group(sampler_bind_group, label);
}
