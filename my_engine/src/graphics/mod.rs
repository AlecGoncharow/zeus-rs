pub mod context;

use crate::math::Mat4;

pub trait CameraProjection {
    fn projection_view_matrix(&self) -> Mat4;
    fn projection_matrix(&self) -> Mat4;
    fn view_matrix(&self) -> Mat4;
}
