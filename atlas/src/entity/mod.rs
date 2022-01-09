use pantheon::context::Context;
use pantheon::math::Vec3;

pub mod component;
// use component::AsComponent;
use super::camera::Camera;
use component::*;
use cube::Cuboid;
use enum_dispatch::enum_dispatch;
use sun::Sun;

pub mod cube;
pub mod light;
pub mod plane;
pub mod sun;
pub mod terrain;
pub mod triangle;
pub mod water;

//use plane::Plane;
//use triangle::Triangle;

#[allow(dead_code)]
enum Message {
    Foo,
    Bar,
}

pub enum EntityPod {
    PodCuboid,
}

#[enum_dispatch(EntityKind)]
pub trait Entity {
    fn init(&mut self, _ctx: &mut Context) {}
    // TODO add callback message function
    fn update(&mut self, ctx: &mut Context);
}

#[enum_dispatch]
#[derive(Debug, Copy, Clone)]
pub enum EntityKind<'a> {
    Cuboid(Cuboid<'a>),
    Sun(Sun<'a>),
    //Plane,
    //Triangle,
}

// scaffolding to allow for undoable/redoable actions
//#[enum_dispatch(CommandKind)]
pub trait Command {
    fn execute(&mut self);
    fn undo(&mut self);
}
