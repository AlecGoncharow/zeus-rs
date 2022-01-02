use super::*;
use pantheon::prelude::*;

const CAMERA_UNIFORM_BUFFER_SIZE: wgpu::BufferAddress = 2 * 16 * 4 + 4 * 3 + 4 * 2 + 12;
const CAMERA_REFLECT: &'static str = "camera_reflect";

pub fn init_camera_resources(ctx: &mut Context) {
    let camera_bind_group_layout = ctx
        .wrangler
        .find_bind_group_layout(CAMERA_GLOBAL_LIGHT_UNIFORM);

    let camera_uniform_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Camera Uniform Buffer"),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
        size: CAMERA_UNIFORM_BUFFER_SIZE,
    });
    let global_light_uniform_buffer = ctx.wrangler.find_uniform_buffer(GLOBAL_LIGHT);

    let camera_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &camera_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: global_light_uniform_buffer.as_entire_binding(),
            },
        ],
        label: Some(CAMERA_GLOBAL_LIGHT_UNIFORM),
    });

    let reflect_camera_uniform_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Reflect Camera Uniform Buffer"),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
        size: CAMERA_UNIFORM_BUFFER_SIZE,
    });

    let reflect_camera_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &camera_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: reflect_camera_uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: global_light_uniform_buffer.as_entire_binding(),
            },
        ],
        label: Some("Reflect Camera Bind Group"),
    });

    let _camera_bind_group_handle = ctx
        .wrangler
        .add_bind_group(camera_bind_group, CAMERA_GLOBAL_LIGHT_UNIFORM);

    let _camera_uniform_buffer = ctx
        .wrangler
        .add_uniform_buffer(camera_uniform_buffer, CAMERA_GLOBAL_LIGHT_UNIFORM);

    let _reflect_camera_bind_group_handle = ctx
        .wrangler
        .add_bind_group(reflect_camera_bind_group, CAMERA_REFLECT);

    let _reflect_camera_uniform_buffer = ctx
        .wrangler
        .add_uniform_buffer(reflect_camera_uniform_buffer, CAMERA_REFLECT);
}
