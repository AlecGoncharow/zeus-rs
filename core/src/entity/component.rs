use super::Context;
use crate::camera::Camera;
use enum_dispatch::enum_dispatch;
use pantheon::math::Vec3;

#[enum_dispatch(EntityKind)]
pub trait DrawComponent {
    fn draw(&mut self, ctx: &mut Context);

    /// this offers an additional draw call to draw stuff like surface norms and what not
    /// to keep the main draw call lean
    fn debug_draw(&mut self, _ctx: &mut Context) {}
}

pub struct MousePick<'a> {
    pub entity: &'a mut dyn MouseComponent,
    pub point: Vec3,
    pub distance: f32,
}

impl<'a> MousePick<'a> {
    pub fn new(entity: &'a mut dyn MouseComponent, point: Vec3, distance: f32) -> Self {
        Self {
            entity,
            point,
            distance,
        }
    }
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
    ) -> Option<MousePick>;
}
