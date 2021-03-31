use super::component::*;
use super::plane::Plane;
use super::Entity;
use crate::camera::Camera;
use pantheon::context::Context;
use pantheon::Vec3;

#[derive(Debug, Copy, Clone)]
pub struct Terrain {}

impl Entity for Terrain {
    fn update(&mut self, _ctx: &mut Context) {}
}

impl DrawComponent for Terrain {
    fn draw(&mut self, _ctx: &mut Context) {}
}

impl MouseComponent for Terrain {
    fn click_start(&mut self, _ctx: &mut Context) {}
    fn click_end(&mut self, _ctx: &mut Context) {}

    fn mouse_over(&mut self, _ctx: &mut Context, _pos: Vec3, _camera: &Camera) {}

    fn check_collision(
        &mut self,
        _ctx: &mut Context,
        _camera_origin: Vec3,
        _mouse_direction: Vec3,
    ) -> Option<MousePick> {
        None
    }
}
