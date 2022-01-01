use super::Camera;
use super::Context;
use super::EntityPod;

use enum_dispatch::enum_dispatch;
use pantheon::math::Vec3;

#[enum_dispatch(EntityKind)]
pub trait DrawComponent<'a> {
    /// this is called when an entity is registered with a render pass, the entity should acquire
    /// it's position into its respective buffers and handles to the expected draw commands
    // @FIXME remove the default impl once existing entitys are updated
    fn register(&mut self, _ctx: &mut Context<'a>) {}

    fn draw(&mut self, ctx: &mut Context<'a>);

    /// this offers an additional draw call to draw stuff like surface norms and what not
    /// to keep the main draw call lean
    fn debug_draw(&mut self, _ctx: &mut Context<'a>) {}
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

///#[enum_dispatch(EntityKind)]
pub trait PodComponent {
    fn into_pod(&self) -> EntityPod;
}
