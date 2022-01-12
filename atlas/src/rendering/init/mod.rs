pub mod camera;
pub mod lights;
pub mod shaded;
pub mod texture;
pub mod water;

use self::prelude::GlobalLightUniforms;

pub use super::*;
pub use camera::*;
pub use lights::*;
pub use shaded::*;
pub use texture::*;
pub use water::*;

// @TODO FIXME this is arbitrary
pub const VERTEX_BUFFER_SIZE: wgpu::BufferAddress = ((3 + 4 + 3) * 4 * 3) * 200_000;
// @TODO FIXME this is arbitrary
pub const INDEX_BUFFER_SIZE: wgpu::BufferAddress = 4 * 2_000_000;

pub const DEPTH_CLEAR: f32 = 0.0;

pub const UNIFORM_BUFFER_VERTEX: &'static str = "uniform_buffer_vertex";
pub const UNIFORM_BUFFER_FRAGMENT: &'static str = "uniform_buffer_fragment";
pub const UNIFORM_BUFFER_VERTEX_FRAGMENT: &'static str = "uniform_buffer_vertex_fragment";

pub fn init_shared(ctx: &mut Context) {
    let padding_bgl = ctx
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[],
            label: Some("padding for no pass bgl and some draw call bgl"),
        });
    let _handle = ctx
        .wrangler
        .add_bind_group_layout(padding_bgl, "pass_padding");

    let buffer_bgl = ctx
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
            label: Some(UNIFORM_BUFFER_VERTEX),
        });
    let _handle = ctx
        .wrangler
        .add_bind_group_layout(buffer_bgl, UNIFORM_BUFFER_VERTEX);

    let buffer_bgl = ctx
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },

                count: None,
            }],
            label: Some(UNIFORM_BUFFER_FRAGMENT),
        });
    let _handle = ctx
        .wrangler
        .add_bind_group_layout(buffer_bgl, UNIFORM_BUFFER_FRAGMENT);

    let buffer_bgl = ctx
        .device
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
            label: Some(UNIFORM_BUFFER_VERTEX_FRAGMENT),
        });
    let _handle = ctx
        .wrangler
        .add_bind_group_layout(buffer_bgl, UNIFORM_BUFFER_VERTEX_FRAGMENT);
}

pub fn init_entity_resources(ctx: &mut Context) {
    let depth_texture = Texture::create_depth_texture(&ctx.device, &ctx.surface_config, "depth");

    let _depth_texture_handle = ctx.wrangler.add_texture(depth_texture, "depth");
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

pub struct InitParams {
    pub global_light_uniforms: GlobalLightUniforms,
    pub water_height: f32,
    pub refraction_offset: f32,
}

pub fn init<'a>(ctx: &mut Context<'a>, params: InitParams) {
    init::init_shared(ctx);
    init::init_global_light(ctx, params.global_light_uniforms);
    init::init_camera_resources(ctx);
    init::init_shaded_resources(ctx, "shaded", params.water_height, params.refraction_offset);
    init::init_reflection_pass(ctx);
    init::init_refraction_pass(ctx);
    init::init_shaded_pass(ctx);
    init::init_water_pass(ctx);
    init::init_basic_textured_pass(ctx);
}
