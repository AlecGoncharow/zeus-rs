use super::*;
use pantheon::graphics::prelude::*;
use pantheon::prelude::*;

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
