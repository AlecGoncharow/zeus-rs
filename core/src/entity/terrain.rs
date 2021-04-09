use super::component::*;
use crate::camera::Camera;
use crate::Color;
use pantheon::graphics::mode::DrawMode;
use pantheon::graphics::vertex::ShadedVertex;
use pantheon::graphics::vertex::Vertex;
use pantheon::graphics::Drawable;
use pantheon::graphics::PolygonMode;
use pantheon::graphics::Topology;
use pantheon::Vec3;
use pantheon::{context::Context, Mat4};

#[derive(Debug, Clone)]
pub struct Terrain {
    pub verts: Vec<ShadedVertex>,
    pub indices: Vec<u32>,
    pub center: Vec3,
    norm_debug: Vec<Vertex>,
}

impl Terrain {
    pub fn from_data(verts: Vec<ShadedVertex>, indices: Vec<u32>) -> Self {
        Self {
            norm_debug: Vec::with_capacity(verts.len() * 2),
            verts,
            indices,
            center: (0, 0, 0).into(),
        }
    }
}

impl Terrain {
    pub fn init(&mut self, _ctx: &mut Context) {
        let color = Color::new(0, 0, 0);
        let odd_color = Color::new(255, 0, 255);
        let even_color = Color::new(0, 255, 255);

        for (i, vert) in self.verts.iter().enumerate() {
            self.norm_debug.push((vert.position, color).into());

            self.norm_debug.push(
                (
                    vert.position + (3. * vert.normal),
                    if i % 2 == 0 { even_color } else { odd_color },
                )
                    .into(),
            );
        }
    }

    pub fn update(&mut self, _ctx: &mut Context) {}

    pub fn draw(&mut self, ctx: &mut Context) {
        ctx.set_model(self.model_matrix());
        ctx.draw_indexed(self.draw_mode(), &self.verts, &self.indices);
    }

    pub fn debug_draw(&mut self, ctx: &mut Context) {
        ctx.set_model(self.model_matrix());
        ctx.draw(
            DrawMode::Normal(Topology::LineList(PolygonMode::Fill)),
            &self.norm_debug,
        );
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
        Mat4::translation::<f32>((-1. * self.center).into())
    }

    fn draw_mode(&self) -> DrawMode {
        DrawMode::Shaded(Topology::TriangleList(PolygonMode::Fill))
    }

    fn rotate(&mut self, _theta: f32, _axis: Vec3) {}

    fn translate(&mut self, _tuple: (f32, f32, f32)) {}
}
