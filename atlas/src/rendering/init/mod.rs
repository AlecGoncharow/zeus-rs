pub mod camera;
pub mod lights;
pub mod shaded;
pub mod texture;
pub mod water;

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
