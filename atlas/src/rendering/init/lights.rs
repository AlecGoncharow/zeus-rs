use crate::entity::light::infinite::{CASCADE_COUNT, MAP_SIZE};

use super::SHADED_WGSL;
use super::{prelude::*, DEPTH_CLEAR, SHADED, VS_BAKE};
use pantheon::graphics::prelude::*;
use pantheon::prelude::*;
use pantheon::wgpu;
use pantheon::wgpu::util::DeviceExt;
use pantheon::wrangler::PASS_PADDING;

/*
const LIGHT_UNIFORM_BUFFER_SIZE: wgpu::BufferAddress = (16 + 3 + 1 + 4) * 4;
const MAX_LIGHTS: usize = 1;
const SHADOW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;
const SHADOW_SIZE: wgpu::Extent3d = wgpu::Extent3d {
    width: 4096,
    height: 4096,
    depth_or_array_layers: MAX_LIGHTS as u32,
};
*/

const GLOBAL_SHADOW_BAKE_BUFFER_SIZE: wgpu::BufferAddress =
    std::mem::size_of::<ShadowBakeUniforms>() as u64;
const GLOBAL_SHADOW_BUFFER_SIZE: wgpu::BufferAddress =
    std::mem::size_of::<GlobalShadowUniforms>() as u64;

const GLOBAL_LIGHT_SHADOW_SIZE: wgpu::Extent3d = wgpu::Extent3d {
    width: (MAP_SIZE as usize * CASCADE_COUNT) as u32,
    height: MAP_SIZE as u32,
    depth_or_array_layers: 1,
};

pub const GLOBAL_LIGHT: &'static str = "global_light";
pub const GLOBAL_LIGHT_SHADOW: &'static str = "global_light_shadow";
pub const GLOBAL_LIGHT_BAKE_SHADOW: &'static str = "global_light_shadow";
pub const GLOBAL_LIGHT_SHADOW_FMT: &'static str = "global_light_shadow_{}";
pub const GLOBAL_LIGHT_SHADOW_0: &'static str = "global_light_shadow_0";
pub const GLOBAL_LIGHT_SHADOW_1: &'static str = "global_light_shadow_1";
pub const GLOBAL_LIGHT_SHADOW_2: &'static str = "global_light_shadow_2";
pub const GLOBAL_LIGHT_SHADOW_3: &'static str = "global_light_shadow_3";

pub const fn int_to_cascade(i: usize) -> &'static str {
    match i {
        0 => GLOBAL_LIGHT_SHADOW_0,
        1 => GLOBAL_LIGHT_SHADOW_1,
        2 => GLOBAL_LIGHT_SHADOW_2,
        3 => GLOBAL_LIGHT_SHADOW_3,
        _ => unreachable!(),
    }
}

pub fn init_global_light(ctx: &mut Context, global_light_uniforms: GlobalLightUniforms) {
    let buffer_bgl = ctx
        .device
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
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },

                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
            ],
            label: Some(GLOBAL_LIGHT),
        });

    let light_uniform_buffer = ctx
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(GLOBAL_LIGHT),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(global_light_uniforms.as_bytes()),
        });
    //
    // this is samplers for shaded pass
    let texture = Texture::create_depth_texture_with_size(
        &ctx.device,
        GLOBAL_LIGHT_SHADOW_SIZE,
        &wgpu::TextureViewDescriptor {
            label: Some(GLOBAL_LIGHT_SHADOW),
            format: None,
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0 as u32,
            array_layer_count: std::num::NonZeroU32::new(1),
        },
        GLOBAL_LIGHT_SHADOW,
    );
    let shadow_uniform_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(&GLOBAL_LIGHT_SHADOW),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
        size: GLOBAL_SHADOW_BUFFER_SIZE,
    });

    let sampler = Texture::shadow_texture_sampler(&ctx.device);

    let lights_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &buffer_bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: light_uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: shadow_uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
        label: Some(GLOBAL_LIGHT),
    });

    ctx.wrangler.add_texture(texture, GLOBAL_LIGHT_SHADOW);

    ctx.wrangler
        .add_uniform_buffer(shadow_uniform_buffer, GLOBAL_LIGHT_SHADOW);

    let shadow_bgl = ctx
        .device
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
            label: Some(GLOBAL_LIGHT_SHADOW),
        });
    /*
     * bad vibes
    let texture = Texture::create_surface_texture_size(
        &ctx.device,
        (
            GLOBAL_LIGHT_SHADOW_SIZE.width,
            GLOBAL_LIGHT_SHADOW_SIZE.height,
        ),
        GLOBAL_LIGHT_SHADOW_COLOR,
    );
    ctx.wrangler.add_texture(texture, GLOBAL_LIGHT_SHADOW_COLOR);
    */

    for i in 0..CASCADE_COUNT {
        let s = int_to_cascade(i);
        let buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&s),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
            size: GLOBAL_SHADOW_BAKE_BUFFER_SIZE,
        });
        let bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &shadow_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some(&s),
        });

        ctx.wrangler.add_uniform_buffer(buffer, s);
        ctx.wrangler.add_bind_group(bg, s);
    }

    ctx.wrangler
        .add_bind_group_layout(shadow_bgl, GLOBAL_LIGHT_BAKE_SHADOW);

    let _handle = ctx
        .wrangler
        .add_uniform_buffer(light_uniform_buffer, GLOBAL_LIGHT);

    let bgl_handle = ctx.wrangler.add_bind_group_layout(buffer_bgl, GLOBAL_LIGHT);
    let bg_handle = ctx.wrangler.add_bind_group(lights_bind_group, GLOBAL_LIGHT);
    ctx.wrangler.frame_bind_group_layout_handle = bgl_handle;
    ctx.wrangler.frame_bind_group_handle = bg_handle;
}

pub fn init_global_bake_shadow_pass<'a>(ctx: &mut Context<'a>) {
    for i in 0..CASCADE_COUNT {
        let pass_label = int_to_cascade(i);
        // @TODO this needs to make it's own view per layer

        let vertex_buffer_handle = ctx.wrangler.handle_to_vertex_buffer(SHADED).unwrap();
        let index_buffer_handle = ctx.wrangler.handle_to_index_buffer(SHADED).unwrap();

        let bind_group_layout = ctx
            .wrangler
            .handle_to_bind_group_layout(GLOBAL_LIGHT_BAKE_SHADOW)
            .unwrap();
        let bind_group = ctx.wrangler.handle_to_bind_group(pass_label).unwrap();

        let pass_bind_group_handle = Some(bind_group);

        // Bad vibes
        let color_attachment_ops = None;
        let color_attachment_view_handle = None;

        let depth_ops = if i == 0 {
            Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(DEPTH_CLEAR),
                store: true,
            })
        } else {
            Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            })
        };

        /*
        let depth_texture = ctx.wrangler.find_texture(GLOBAL_LIGHT_SHADOW);
        let depth_view = depth_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                label: Some(pass_label),
                format: None,
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: i as u32,
                array_layer_count: std::num::NonZeroU32::new(1),
            });
        let depth_stencil_view = Some(ViewKind::View(depth_view));
        */

        let depth_stencil_view = Some(ViewKind::Handle(
            ctx.wrangler.handle_to_texture(GLOBAL_LIGHT_SHADOW).unwrap(),
        ));

        let push_constant_ranges = &[wgpu::PushConstantRange {
            stages: wgpu::ShaderStages::VERTEX,
            range: 0..16 * 4,
        }];

        let frame_bind_group_layout_handle_override = Some(
            ctx.wrangler
                .handle_to_bind_group_layout(PASS_PADDING)
                .unwrap(),
        );

        let pipeline_ctx = PipelineContext {
            pass_bind_group_layout_handle: Some(bind_group_layout),
            draw_call_bind_group_layout_handle: None,
            frame_bind_group_layout_handle_override,

            push_constant_ranges,
            vs_module_name: Some(SHADED_WGSL),
            vs_entry_point: Some(VS_BAKE),
            fs_module_name: None,
            fs_entry_point: None,
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
                unclipped_depth: true,
                conservative: false,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: pantheon::graphics::texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::GreaterEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: 2, // https://github.com/gfx-rs/wgpu/blob/master/wgpu/examples/shadow/main.rs#L524
                    slope_scale: 2.0,
                    clamp: 0.0,
                },
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

        let viewport = Some(Viewport::new(
            i as f32 * MAP_SIZE,
            0.,
            MAP_SIZE,
            MAP_SIZE,
            0.0,
            1.0,
        ));

        let frame_bind_group_handle_override =
            Some(ctx.wrangler.handle_to_bind_group(PASS_PADDING).unwrap());

        let pass = Pass {
            label: pass_label,
            pipeline_ctx,
            pipelines,
            color_attachment_ops,
            color_attachment_view_handle,
            depth_ops,
            stencil_ops: None,
            depth_stencil_view,
            viewport,
            frame_bind_group_handle_override,
            pass_bind_group_handle,
            vertex_buffer_handle,
            index_buffer_handle,
        };

        let _handle = ctx.wrangler.add_pass(pass, pass_label);
    }
}
