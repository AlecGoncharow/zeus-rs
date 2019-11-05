pub mod context;
pub mod topology;

pub use topology::PolygonMode;
pub use topology::Topology;

use crate::math::Mat4;

pub trait CameraProjection {
    fn projection_view_matrix(&self) -> Mat4;
    fn projection_matrix(&self) -> Mat4;
    fn view_matrix(&self) -> Mat4;
}

pub struct DefaultCamera {}

impl CameraProjection for DefaultCamera {
    fn projection_view_matrix(&self) -> Mat4 {
        Mat4::identity()
    }
    fn projection_matrix(&self) -> Mat4 {
        Mat4::identity()
    }
    fn view_matrix(&self) -> Mat4 {
        Mat4::identity()
    }
}

pub trait ModelProjection {
    fn model_matrix(&self) -> Mat4;
}

pub struct DefaultModelProjecton {}

impl ModelProjection for DefaultModelProjecton {
    fn model_matrix(&self) -> Mat4 {
        Mat4::identity()
    }
}
