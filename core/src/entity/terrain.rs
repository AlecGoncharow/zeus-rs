use super::component::*;
use crate::camera::Camera;
use pantheon::graphics::mode::DrawMode;
use pantheon::graphics::vertex::ShadedVertex;
use pantheon::graphics::Drawable;
use pantheon::graphics::PolygonMode;
use pantheon::graphics::Topology;
use pantheon::Vec3;
use pantheon::{context::Context, Mat4};

#[derive(Debug, Clone)]
pub struct Terrain {
    pub verts: Vec<ShadedVertex>,
    pub indices: Vec<u32>,
}

impl Terrain {
    pub fn from_data(verts: Vec<ShadedVertex>, indices: Vec<u32>) -> Self {
        Self { verts, indices }
    }
}

impl Terrain {
    pub fn update(&mut self, _ctx: &mut Context) {}

    pub fn draw(&mut self, ctx: &mut Context) {
        //ctx.gfx_context.model_transform = self.model_matrix();

        ctx.draw_indexed(self.draw_mode(), &self.verts, &self.indices);
    }

    pub fn click_start(&mut self, _ctx: &mut Context) {}
    pub fn click_end(&mut self, _ctx: &mut Context) {}

    pub fn mouse_over(&mut self, _ctx: &mut Context, _pos: Vec3, _camera: &Camera) {}

    pub fn check_collision(
        &mut self,
        _ctx: &mut Context,
        _camera_origin: Vec3,
        _mouse_direction: Vec3,
    ) -> Option<MousePick> {
        None
    }
}

impl Drawable for Terrain {
    fn model_matrix(&self) -> Mat4 {
        Mat4::scalar_from_one(1)
    }

    fn draw_mode(&self) -> DrawMode {
        DrawMode::Shaded(Topology::TriangleList(PolygonMode::Fill))
    }

    fn rotate(&mut self, theta: f32, axis: Vec3) {}

    fn translate(&mut self, tuple: (f32, f32, f32)) {}
}
