pub mod color;
pub mod renderer;
pub mod texture;
pub mod topology;
pub mod vertex;

pub use color::Color;
pub use topology::Mode;
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

    /// vertex buffer values (Position, Color, Normal)
    fn vertices(&self) -> &[(Vec3, color::Color, Vec3)];

    /// index buffer values
    fn indices(&self) -> Option<&[u16]> {
        None
    }

    fn draw_mode(&self) -> Mode {
        Mode::Normal(Topology::TriangleList(PolygonMode::Fill))
    }

    fn rotate(&mut self, _theta: f32, _axis: Vec3) {}
    fn translate(&mut self, (_x_tr, _y_tr, _z_tr): (f32, f32, f32)) {}
}
