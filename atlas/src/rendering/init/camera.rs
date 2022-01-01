use pantheon::prelude::*;

const CAMERA_UNIFORM_BUFFER_SIZE: wgpu::BufferAddress = 2 * 16 * 4 + 4 * 3 + 4 * 2 + 12;

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
