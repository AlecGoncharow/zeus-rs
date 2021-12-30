use pantheon::graphics::prelude::*;
use pantheon::prelude::*;
use pantheon::Mat4;

/// here be dragons
pub mod init;

pub mod prelude {
    pub use super::init::init_shaded_pass;
    pub use super::CameraUniforms;
}

#[repr(C)]
pub struct CameraUniforms {
    pub view: Mat4,
    pub projection: Mat4,
}

impl CameraUniforms {
    pub fn new(view: Mat4, projection: Mat4) -> Self {
        Self { view, projection }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let data_ptr: *const Self = self;
            let byte_ptr: *const u8 = data_ptr as *const _;
            std::slice::from_raw_parts(byte_ptr, std::mem::size_of::<Self>())
        }
    }

    pub fn push(&self, ctx: &mut Context, buffer_handle: &BufferHandle) {
        let buffer = ctx.wrangler.get_uniform_buffer(&buffer_handle);

        ctx.queue.write_buffer(buffer, 0, self.as_bytes());
    }
}

pub fn register<'a, T>(
    ctx: &mut Context<'a>,
    pass_labels: &[&'a str],
    vertex_label: &'a str,
    topology: Topology,
    verts: &[T],
    instances: std::ops::Range<u32>,
    push_constant: Option<PushConstant>,
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

    let draw_call = DrawCall::Vertex {
        vertices,
        instances,
        push_constant,
        topology,
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

    let draw_call = DrawCall::Indexed {
        indices,
        base_vertex: vertex_cursor as i32,
        instances,
        push_constant,
        topology,
    };

    println!("[register_indexed] draw_call: {:#?}", draw_call);

    let handle = ctx.wrangler.add_draw_call(draw_call, vertex_label);

    pass_labels.iter().for_each(|label| {
        let pass_handle = ctx
            .wrangler
            .handle_to_pass(label)
            .expect("resource needs to be init first");
        let pass = ctx.wrangler.get_pass_mut(&pass_handle);
        pass.draw_call_handles.push(handle);
        println!(
            "[register_indexed] pass.draw_call_handles: {:#?}",
            pass.draw_call_handles
        );
    });

    handle
}
