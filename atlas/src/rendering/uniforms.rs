use pantheon::graphics::prelude::*;
use pantheon::math::prelude::*;
use wgpu::util::DeviceExt;

pub trait Uniforms: 'static + Sized + Send + Sync + std::fmt::Debug {
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            let data_ptr: *const Self = self;
            let byte_ptr: *const u8 = data_ptr as *const _;
            std::slice::from_raw_parts(byte_ptr, std::mem::size_of::<Self>())
        }
    }

    fn push(&self, ctx: &mut pantheon::context::Context<'_>, buffer_handle: &BufferHandle<'_>) {
        let buffer = ctx.wrangler.get_uniform_buffer(&buffer_handle);

        ctx.queue.write_buffer(buffer, 0, self.as_bytes());
    }

    fn register<'a>(
        &self,
        ctx: &mut pantheon::context::Context<'a>,
        layout_label: &'a str,
        label: &'a str,
    ) -> (BindGroupHandle<'a>, BufferHandle<'a>) {
        let bind_group_layout_handle = ctx
            .wrangler
            .handle_to_bind_group_layout(&layout_label)
            .expect(&format!("{} bind group layout not init", layout_label));

        let bind_group_layout = ctx
            .wrangler
            .get_bind_group_layout(&bind_group_layout_handle);

        let buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                usage: wgpu::BufferUsages::UNIFORM,
                contents: bytemuck::cast_slice(self.as_bytes()),
            });

        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some(label),
        });

        (
            ctx.wrangler.add_bind_group(bind_group, label),
            ctx.wrangler.add_uniform_buffer(buffer, label),
        )
    }
}

impl Uniforms for CameraUniforms {}
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CameraUniforms {
    pub view: Mat4,
    pub projection: Mat4,
    pub position: Vec3,
    _pad0: u32,
    pub planes: Vec2,
    _pad1: u64,
}

impl CameraUniforms {
    pub fn new(view: Mat4, projection: Mat4, position: Vec3, planes: Vec2) -> Self {
        Self {
            view,
            projection,
            position,
            planes,
            _pad0: 0,
            _pad1: 0,
        }
    }
}

impl Uniforms for GlobalLightUniforms {}
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct GlobalLightUniforms {
    pub direction: Vec3,
    _pad0: u32,
    pub color: Vec3,
    _pad1: u32,
    pub bias: Vec2,
    _pad2: u64,
}

impl GlobalLightUniforms {
    pub fn new(direction: Vec3, color: Vec3, bias: Vec2) -> Self {
        Self {
            direction,
            color,
            bias,
            _pad0: 0,
            _pad1: 0,
            _pad2: 0,
        }
    }
}

impl Uniforms for StaticEntityUniforms {}
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct StaticEntityUniforms {
    pub model_matrix: Mat4,
}

impl StaticEntityUniforms {
    pub fn new(model_matrix: Mat4) -> Self {
        Self { model_matrix }
    }
}
