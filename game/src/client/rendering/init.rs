use pantheon::graphics::prelude::*;
use pantheon::pass::{PipelineContext, RecreatePipelines};
use pantheon::prelude::*;
use pantheon::shader::ShaderContext;

const CAMERA_UNIFORM_BUFFER_SIZE: wgpu::BufferAddress = 2 * 16 * 4;
const LIGHT_UNIFORM_BUFFER_SIZE: wgpu::BufferAddress = (16 + 3 + 1 + 4) * 4;

// @TODO FIXME this is arbitrary
const VERTEX_BUFFER_SIZE: wgpu::BufferAddress = ((3 + 4 + 3) * 4 * 3) * 200_000;
// @TODO FIXME this is arbitrary
const INDEX_BUFFER_SIZE: wgpu::BufferAddress = 4 * 2_000_000;

const MAX_LIGHTS: usize = 1;
const SHADOW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
const SHADOW_SIZE: wgpu::Extent3d = wgpu::Extent3d {
    width: 4096,
    height: 4096,
    depth_or_array_layers: MAX_LIGHTS as u32,
};

/// entity -> camera
/// forward -> shadow
/// shadow -> lights

pub fn init_light_resources(ctx: &mut Context) {
    let shadow_sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("shadow"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        compare: None,
        ..Default::default()
    });

    let shadow_texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        size: SHADOW_SIZE,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: SHADOW_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        label: None,
    });
    let shadow_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let shadow_texture = Texture {
        texture: shadow_texture,
        view: shadow_view,
        sampler: shadow_sampler,
    };

    let lights_bind_group_layout =
        ctx.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },

                    count: None,
                }],
                label: Some("lights bind group layout"),
            });

    let lights_uniform_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Light Uniform Buffer"),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
        size: LIGHT_UNIFORM_BUFFER_SIZE,
    });

    let lights_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &lights_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: lights_uniform_buffer.as_entire_binding(),
        }],
        label: None,
    });

    let shadow_bind_group_layout =
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
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("uniform_bind_group_layout"),
            });

    let shadow_sampler_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &shadow_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&shadow_texture.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&shadow_texture.sampler),
            },
        ],
        label: Some("Shadow Sampler Bind Group"),
    });

    let _shadow_bind_group_layout_handle = ctx
        .wrangler
        .add_bind_group_layout(shadow_bind_group_layout, "shadow");
    let _shadow_bind_group_handle = ctx
        .wrangler
        .add_bind_group(shadow_sampler_bind_group, "shadow");
    let _shadow_texture_handle = ctx.wrangler.add_texture(shadow_texture, "shadow");

    let _lights_bind_group_layout_handle = ctx
        .wrangler
        .add_bind_group_layout(lights_bind_group_layout, "lights");
    let _lights_bind_group_handle = ctx.wrangler.add_bind_group(lights_bind_group, "lights");
    let _lights_uniform_buffer = ctx
        .wrangler
        .add_uniform_buffer(lights_uniform_buffer, "lights");
}

pub fn init_camera_resources(ctx: &mut Context) {
    let camera_bind_group_layout =
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

    let _camera_bind_group_layout_handle = ctx
        .wrangler
        .add_bind_group_layout(camera_bind_group_layout, "camera");

    let _camera_bind_group_handle = ctx.wrangler.add_bind_group(camera_bind_group, "camera");

    let _camera_uniform_buffer = ctx
        .wrangler
        .add_uniform_buffer(camera_uniform_buffer, "camera");
}

pub fn init_entity_resources(ctx: &mut Context) {
    let depth_texture = Texture::create_depth_texture(&ctx.device, &ctx.surface_config, "depth");

    let _depth_texture_handle = ctx.wrangler.add_texture(depth_texture, "depth");
}

pub fn init_shaded_resources(ctx: &mut Context) {
    let shaded_vertex_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Shaded Vertex Buffer"),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
        size: VERTEX_BUFFER_SIZE,
    });

    let shaded_index_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Shaded Index Buffer"),
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
        size: INDEX_BUFFER_SIZE,
    });

    let _shaded_vertex_buffer_handle = ctx
        .wrangler
        .add_vertex_buffer(shaded_vertex_buffer, "shaded");
    let _shaded_index_buffer_handle = ctx.wrangler.add_index_buffer(shaded_index_buffer, "shaded");
}

/// :^)
pub fn init_shaded_pass<'a>(ctx: &'a mut Context) -> PassHandle<'a> {
    let depth_texture_handle = match ctx.wrangler.handle_to_texture("depth") {
        Some(handle) => handle,
        None => {
            init_entity_resources(ctx);
            ctx.wrangler.handle_to_texture("depth").unwrap()
        }
    };

    init_shaded_resources(ctx);
    let vertex_buffer_handle = ctx.wrangler.handle_to_vertex_buffer("shaded").unwrap();
    let index_buffer_handle = ctx.wrangler.handle_to_index_buffer("shaded").unwrap();

    let lights_bind_group_layout_handle = match ctx.wrangler.handle_to_bind_group_layout("lights") {
        Some(handle) => handle,
        None => {
            init_light_resources(ctx);
            ctx.wrangler.handle_to_bind_group_layout("lights").unwrap()
        }
    };
    let lights_bind_group_handle = ctx.wrangler.handle_to_bind_group("lights").unwrap();
    let shadow_bind_group_layout_handle =
        ctx.wrangler.handle_to_bind_group_layout("shadow").unwrap();
    let shadow_bind_group_handle = ctx.wrangler.handle_to_bind_group("shadow").unwrap();

    let camera_bind_group_layout_handle = match ctx.wrangler.handle_to_bind_group_layout("camera") {
        Some(handle) => handle,
        None => {
            init_camera_resources(ctx);
            ctx.wrangler.handle_to_bind_group_layout("camera").unwrap()
        }
    };
    let camera_bind_group_handle = ctx.wrangler.handle_to_bind_group("camera").unwrap();

    let bind_group_handles = vec![
        camera_bind_group_handle,
        shadow_bind_group_handle,
        lights_bind_group_handle,
    ];

    let color_attachment_ops = Some(wgpu::Operations {
        load: wgpu::LoadOp::Clear(ctx.gfx_context.clear_color),
        store: true,
    });

    let depth_ops = Some(wgpu::Operations {
        load: wgpu::LoadOp::Clear(1.0),
        store: true,
    });
    let depth_stencil_view_handle = Some(depth_texture_handle);

    let bind_group_layouts = &[
        ctx.wrangler
            .get_bind_group_layout(&camera_bind_group_layout_handle),
        ctx.wrangler
            .get_bind_group_layout(&shadow_bind_group_layout_handle),
        ctx.wrangler
            .get_bind_group_layout(&lights_bind_group_layout_handle),
    ];

    let pipeline_ctx = PipelineContext {
        uniform_bind_group_layout_handles: vec![
            camera_bind_group_layout_handle,
            shadow_bind_group_layout_handle,
            lights_bind_group_layout_handle,
        ],
        vs_path: Some("shaded.vert.spv"),
        fs_path: Some("shaded.frag.spv"),
        vert_desc: crate::base::vertex::ShadedVertex::desc,
        label: Some("shaded pipelines"),
    };

    let recreate_pipelines: RecreatePipelines =
        |pipelines: &mut Vec<wgpu::RenderPipeline>,
         pipeline_ctx: &PipelineContext,
         shader_ctx: &ShaderContext,
         layouts: &[&wgpu::BindGroupLayout],
         device: &wgpu::Device,
         surface_config: &wgpu::SurfaceConfiguration| {
            pipelines.clear();

            let non_fill_polygon_modes = device
                .features()
                .contains(wgpu::Features::POLYGON_MODE_LINE & wgpu::Features::POLYGON_MODE_POINT);
            let render_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: pipeline_ctx.label,
                    bind_group_layouts: layouts,
                    push_constant_ranges: &[wgpu::PushConstantRange {
                        stages: wgpu::ShaderStages::VERTEX,
                        range: 0..(4 * 16),
                    }],
                });

            Topology::iterator(non_fill_polygon_modes).for_each(|mode| {
                let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: pipeline_ctx.label,
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader_ctx.make_module(device, &pipeline_ctx.vs_path.unwrap()),
                        entry_point: "main",
                        buffers: &[(pipeline_ctx.vert_desc)()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        // 2.
                        module: &shader_ctx.make_module(device, &pipeline_ctx.fs_path.unwrap()),
                        entry_point: "main",
                        targets: &[wgpu::ColorTargetState {
                            format: surface_config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        }],
                    }),

                    primitive: wgpu::PrimitiveState {
                        topology: mode.into(),
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        polygon_mode: mode.inner().into(),
                        unclipped_depth: false,
                        conservative: false,
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

                if usize::from(*mode) != pipelines.len()
                    && device.features().contains(
                        wgpu::Features::POLYGON_MODE_LINE & wgpu::Features::POLYGON_MODE_POINT,
                    )
                {
                    panic!("Render pipeline construction broke");
                }
                pipelines.push(pipeline);
            });
        };

    let mut pipelines = Vec::new();

    recreate_pipelines(
        &mut pipelines,
        &pipeline_ctx,
        &ctx.shader_context,
        bind_group_layouts,
        &ctx.device,
        &ctx.surface_config,
    );

    let shaded_pass = Pass {
        label: "shaded",
        pipeline_ctx,
        recreate_pipelines,
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

    ctx.wrangler.add_pass(shaded_pass, "shaded")
}
