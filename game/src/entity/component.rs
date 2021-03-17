use super::Context;
use crate::camera::Camera;
use engine::math::Vec3;
use enum_dispatch::enum_dispatch;

#[enum_dispatch(EntityKind)]
pub trait DrawComponent {
    fn draw(&mut self, ctx: &mut Context);
}

/// this is useful because it allows 3D picking to ignore entities which aren't part of the
/// clickable environment
#[enum_dispatch(EntityKind)]
pub trait MouseComponent {
    // TODO think about x/y/z and hover events
    fn click_start(&mut self, ctx: &mut Context);
    fn click_end(&mut self, ctx: &mut Context);
    fn mouse_over(&mut self, ctx: &mut Context, pos: Vec3, cam: &Camera);
    fn check_collision(
        &mut self,
        ctx: &mut Context,
        camera_origin: Vec3,
        mouse_direction: Vec3,
    ) -> Option<(&mut dyn MouseComponent, Vec3, f32)>;
}
