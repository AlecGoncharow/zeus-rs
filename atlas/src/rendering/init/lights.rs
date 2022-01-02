use super::prelude::*;
use pantheon::graphics::prelude::*;
use pantheon::prelude::*;
use wgpu::util::DeviceExt;

const LIGHT_UNIFORM_BUFFER_SIZE: wgpu::BufferAddress = (16 + 3 + 1 + 4) * 4;
const MAX_LIGHTS: usize = 1;
const SHADOW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
const SHADOW_SIZE: wgpu::Extent3d = wgpu::Extent3d {
    width: 4096,
    height: 4096,
    depth_or_array_layers: MAX_LIGHTS as u32,
};

pub const GLOBAL_LIGHT: &'static str = "global_light";

pub fn init_global_light(ctx: &mut Context, global_light_uniforms: GlobalLightUniforms) {
    let light_uniform_buffer = ctx
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(GLOBAL_LIGHT),
            usage: wgpu::BufferUsages::UNIFORM,
            contents: bytemuck::cast_slice(global_light_uniforms.as_bytes()),
        });

    let _handle = ctx
        .wrangler
        .add_uniform_buffer(light_uniform_buffer, GLOBAL_LIGHT);
}

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
        format: SHADOW_FORMAT,
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
