pub mod mat2;
pub mod mat3;
pub mod mat4;
pub mod vec2;
pub mod vec3;
pub mod vec4;

pub mod quaternion;

pub mod prelude {
    pub use super::quaternion::Quaternion;

    pub use super::mat2::Mat2;
    pub use super::mat3::Mat3;
    pub use super::mat4::Mat4;
    pub use super::vec2::Vec2;
    pub use super::vec3::Vec3;
    pub use super::vec4::Vec4;

    pub use super::Dim;
}

pub use quaternion::Quaternion;

pub use mat2::Mat2;
pub use mat3::Mat3;
pub use mat4::Mat4;
pub use vec2::Vec2;
pub use vec3::Vec3;
pub use vec4::Vec4;

unsafe impl bytemuck::Pod for Mat4 {}
unsafe impl bytemuck::Zeroable for Mat4 {}

unsafe impl bytemuck::Pod for Vec4 {}
unsafe impl bytemuck::Zeroable for Vec4 {}

pub enum Dim {
    X,
    Y,
    Z,
    W,
}
