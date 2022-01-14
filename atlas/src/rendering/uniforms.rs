use pantheon::graphics::prelude::*;
use pantheon::math::prelude::*;
use wgpu::util::DeviceExt;

pub trait AlignedGLSL {
    /// This doesn't do any enforcment of alignment because I have no idea how to do that that
    /// wouldn't just require manual entry anyways, just requires me to keep updating this function
    /// if the size of the struct changes
    /// https://learnopengl.com/Advanced-OpenGL/Advanced-GLSL
    fn validate_alignment(&self);
}

pub trait Uniforms: 'static + Sized + Send + Sync + std::fmt::Debug + AlignedGLSL {
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
        self.validate_alignment();

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
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
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

type Padding32 = u32;
type Padding64 = u64;

impl Uniforms for CameraUniforms {}
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CameraUniforms {
    pub view: Mat4,
    pub projection: Mat4,
    pub position: Vec3,
    _pad0: Padding32,
    pub w: Vec3,
    _pad1: Padding32,
    pub planes: Vec2,
    _pad2: Padding64,
}

impl CameraUniforms {
    pub fn new(view: Mat4, projection: Mat4, position: Vec3, w: Vec3, planes: Vec2) -> Self {
        Self {
            view,
            projection,
            position,
            _pad0: 0,
            w,
            _pad1: 0,
            planes,
            _pad2: 0,
        }
    }
}
impl AlignedGLSL for CameraUniforms {
    fn validate_alignment(&self) {
        assert_eq!(
            64 + 64 + 12 + 4 + 12 + 4 + 8 + 8,
            std::mem::size_of::<Self>()
        );
    }
}

impl Uniforms for GlobalLightUniforms {}
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct GlobalLightUniforms {
    pub direction: Vec3,
    _pad0: Padding32,
    pub color: Vec3,
    _pad1: Padding32,
    pub bias: Vec2,
    _pad2: Padding64,
}
impl AlignedGLSL for GlobalLightUniforms {
    fn validate_alignment(&self) {
        assert_eq!(12 + 4 + 12 + 4 + 8 + 8, std::mem::size_of::<Self>());
    }
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

impl Uniforms for GlobalShadowUniforms {}
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct GlobalShadowUniforms {
    pub shadow0: Mat4,
    pub cascade_offsets: [Vec4; 3],
    pub cascade_scales: [Vec4; 3],
    pub cascade_planes: [Vec4; 3],
}
impl AlignedGLSL for GlobalShadowUniforms {
    fn validate_alignment(&self) {
        assert_eq!(16 + 12 + 12 + 12, std::mem::size_of::<Self>());
    }
}

impl GlobalShadowUniforms {
    pub fn new(
        shadow0: Mat4,
        cascade_offsets: [Vec4; 3],
        cascade_scales: [Vec4; 3],
        cascade_planes: [Vec4; 3],
    ) -> Self {
        Self {
            shadow0,
            cascade_offsets,
            cascade_scales,
            cascade_planes,
        }
    }
}

impl Uniforms for StaticEntityUniforms {}
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct StaticEntityUniforms {
    pub model_matrix: Mat4,
}
impl AlignedGLSL for StaticEntityUniforms {
    fn validate_alignment(&self) {
        assert_eq!(64, std::mem::size_of::<Self>());
    }
}

impl StaticEntityUniforms {
    pub fn new(model_matrix: Mat4) -> Self {
        Self { model_matrix }
    }
}

impl Uniforms for ShadowBakeUniforms {}
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ShadowBakeUniforms {
    pub projection: Mat4,
    pub view: Mat4,
}
impl AlignedGLSL for ShadowBakeUniforms {
    fn validate_alignment(&self) {
        assert_eq!(64 + 64, std::mem::size_of::<Self>());
    }
}

impl ShadowBakeUniforms {
    pub fn new(projection: Mat4, view: Mat4) -> Self {
        Self { projection, view }
    }
}
