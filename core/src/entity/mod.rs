use pantheon::context::Context;
use pantheon::math::Vec3;

pub mod component;
// use component::AsComponent;
use crate::camera::Camera;
use component::DrawComponent;
use component::MouseComponent;
use enum_dispatch::enum_dispatch;
use hermes::message::Pod;

pub mod cube;
pub mod plane;
pub mod triangle;

use cube::Cuboid;
//use plane::Plane;
//use triangle::Triangle;

#[allow(dead_code)]
enum Message {
    Foo,
    Bar,
}

#[enum_dispatch(EntityKind)]
pub trait Entity: Pod {
    // TODO add callback message function
    fn update(&mut self, ctx: &mut Context);
}

#[enum_dispatch]
#[derive(Debug, Copy, Clone)]
pub enum EntityKind {
    Cuboid,
    //Plane,
    //Triangle,
}

// scaffolding to allow for undoable/redoable actions
//#[enum_dispatch(CommandKind)]
pub trait Command {
    fn execute(&mut self);
    fn undo(&mut self);
}
