pub mod renderer;
pub mod topology;

pub use topology::PolygonMode;
pub use topology::Topology;

use crate::math::Mat4;
use crate::math::Vec3;

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

pub trait Drawable {
    /// R*T Matrix to translate model from model space to world space
    fn model_matrix(&self) -> Mat4 {
        Mat4::identity()
    }

    /// vertex buffer values (Position, Color)
    fn vertices(&self) -> &Vec<(Vec3, Vec3)>;

    /// index buffer values
    fn indices(&self) -> Option<&[u16]> {
        None
    }

    fn draw_mode(&self) -> Topology {
        Topology::TriangleList(PolygonMode::Fill)
    }

    fn rotate(&mut self, _theta: f64, _axis: Vec3) {}
    fn translate(&mut self, (_x_tr, _y_tr, _z_tr): (f64, f64, f64)) {}
}