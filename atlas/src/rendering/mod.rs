use pantheon::graphics::prelude::*;
use pantheon::prelude::*;
use pantheon::*;

/// here be dragons
pub mod init;
pub mod uniforms;

pub mod prelude {
    pub use super::init;
    pub use super::uniforms::*;
    pub use crate::rendering;
}

pub fn register<'a, T>(
    ctx: &mut Context<'a>,
    pass_labels: &[&'a str],
    vertex_label: &'a str,
    topology: Topology,
    verts: &[T],
    instances: std::ops::Range<u32>,
    push_constant: Option<PushConstant>,
    bind_group_handles: &[BindGroupHandle<'a>],
) -> DrawCallHandle<'a>
where
    T: bytemuck::Pod,
{
    let vertex_cursor_handle = ctx
        .wrangler
        .handle_to_vertex_buffer_cursor(vertex_label)
        .expect("this should be init");
    let vertex_cursor = *ctx.wrangler.get_vertex_buffer_cursor(&vertex_cursor_handle);
    // @TODO maybe this should be provided by the caller?
    let vertex_buffer = ctx.wrangler.find_vertex_buffer(vertex_label);
    let vert_count = verts.len() as u64;
    let vert_data: &[u8] = bytemuck::cast_slice(&verts);
    ctx.queue.write_buffer(
        vertex_buffer,
        vertex_cursor * std::mem::size_of::<T>() as u64,
        vert_data,
    );

    let new_vert_cursor = vertex_cursor + vert_count;
    let new_vert_cursor = new_vert_cursor + (new_vert_cursor % wgpu::COPY_BUFFER_ALIGNMENT);
    ctx.wrangler
        .swap_vertex_buffer_cursor(vertex_cursor_handle, new_vert_cursor);
    let vertices = vertex_cursor as u32..vertex_cursor as u32 + vert_count as u32;

    let bind_group_handles = Vec::from(bind_group_handles);

    let draw_call = DrawCall::Vertex {
        vertices,
        instances,
        push_constant,
        topology,
        bind_group_handles,
    };

    let handle = ctx.wrangler.add_draw_call(draw_call, vertex_label);

    pass_labels.iter().for_each(|label| {
        let pass_handle = ctx
            .wrangler
            .handle_to_pass(label)
            .expect("resource needs to be init first");
        let pass = ctx.wrangler.get_pass_mut(&pass_handle);
        pass.draw_call_handles.push(handle);
    });

    handle
}

pub fn register_indexed<'a, T>(
    ctx: &mut Context<'a>,
    pass_labels: &[&'a str],
    vertex_label: &'a str,
    topology: Topology,
    verts: &[T],
    indices: &[u32],
    instances: std::ops::Range<u32>,
    push_constant: Option<PushConstant>,
    bind_group_handles: &[BindGroupHandle<'a>],
) -> DrawCallHandle<'a>
where
    T: bytemuck::Pod,
{
    let vertex_cursor_handle = ctx
        .wrangler
        .handle_to_vertex_buffer_cursor(vertex_label)
        .expect("this should be init");
    let vertex_cursor = *ctx.wrangler.get_vertex_buffer_cursor(&vertex_cursor_handle);
    // @TODO maybe this should be provided by the caller?
    let vertex_buffer = ctx.wrangler.find_vertex_buffer(vertex_label);
    let vert_count = verts.len() as u64;
    let vert_data: &[u8] = bytemuck::cast_slice(&verts);
    ctx.queue.write_buffer(
        vertex_buffer,
        vertex_cursor * std::mem::size_of::<T>() as u64,
        vert_data,
    );

    let new_vert_cursor = vertex_cursor + vert_count;
    let new_vert_cursor = new_vert_cursor + (new_vert_cursor % wgpu::COPY_BUFFER_ALIGNMENT);
    ctx.wrangler
        .swap_vertex_buffer_cursor(vertex_cursor_handle, new_vert_cursor);

    let index_cursor_handle = ctx
        .wrangler
        .handle_to_index_buffer_cursor(vertex_label)
        .expect("this should be init");
    let index_cursor = *ctx.wrangler.get_index_buffer_cursor(&index_cursor_handle);
    let index_buffer = ctx.wrangler.find_index_buffer(vertex_label);
    let index_data: &[u8] = bytemuck::cast_slice(&indices);
    // @TODO maybe this should be provided by the caller?
    let index_count = indices.len() as u64;
    ctx.queue.write_buffer(
        index_buffer,
        index_cursor * std::mem::size_of::<u32>() as u64,
        index_data,
    );

    let new_index_cursor = index_cursor + index_count;
    let new_index_cursor = new_index_cursor + (new_index_cursor % wgpu::COPY_BUFFER_ALIGNMENT);
    ctx.wrangler
        .swap_index_buffer_cursor(index_cursor_handle, new_index_cursor);

    let indices = index_cursor as u32..index_cursor as u32 + index_count as u32;

    let bind_group_handles = Vec::from(bind_group_handles);

    let draw_call = DrawCall::Indexed {
        indices,
        base_vertex: vertex_cursor as i32,
        instances,
        push_constant,
        topology,
        bind_group_handles,
    };

    println!("[register_indexed] draw_call: {:#?}", draw_call);

    let handle = ctx.wrangler.add_draw_call(draw_call, vertex_label);

    pass_labels.iter().for_each(|label| {
        let pass_handle = ctx
            .wrangler
            .handle_to_pass(label)
            .expect(&format!("No registered pass labeled {}", &label));
        let pass = ctx.wrangler.get_pass_mut(&pass_handle);
        pass.draw_call_handles.push(handle);
        println!(
            "[register_indexed] pass.draw_call_handles: {:#?}",
            pass.draw_call_handles
        );
    });

    handle
}

pub fn register_texture<'a>(
    ctx: &mut Context<'a>,
    texture: Texture,
    label: &'a str,
    layout_label: &'a str,
    sampler_override: Option<&wgpu::Sampler>,
) -> (BindGroupHandle<'a>, TextureHandle<'a>) {
    let bglh = ctx
        .wrangler
        .handle_to_bind_group_layout(layout_label)
        .expect(&format!(
            "bind group layout {} not registered",
            layout_label
        ));
    let layout = ctx.wrangler.get_bind_group_layout(&bglh);

    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(
                    sampler_override.unwrap_or(&texture.sampler),
                ),
            },
        ],
        label: Some(&format!("{} Sampler Bind Group", label)),
    });

    (
        ctx.wrangler.add_or_swap_bind_group(bind_group, label),
        ctx.wrangler.add_or_swap_texture(texture, label),
    )
}

pub fn recreate_water_sampler_bind_group(ctx: &mut Context) {
    use init::*;
    let texture_sampler_bind_group_layout = ctx
        .wrangler
        .find_bind_group_layout(WATER_TEXTURE_SAMPLER_UNIFORM);
    let reflection = ctx.wrangler.find_texture(REFLECTION);
    let refraction = ctx.wrangler.find_texture(REFRACTION);
    let refraction_depth = ctx.wrangler.find_texture(REFRACTION_DEPTH);

    let texture_sampler_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &texture_sampler_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&reflection.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&refraction.view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&refraction_depth.view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(&Texture::surface_texture_sampler(
                    &ctx.device,
                )),
            },
        ],
        label: Some(WATER_TEXTURE_SAMPLER_UNIFORM),
    });

    let _handle = ctx
        .wrangler
        .add_or_swap_bind_group(texture_sampler_bind_group, WATER_TEXTURE_SAMPLER_UNIFORM);
}
