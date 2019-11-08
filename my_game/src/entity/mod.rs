use my_engine::context::Context;
use std::any::Any;

pub mod cube;

enum Message {
    Foo,
    Bar,
}

pub trait Entity: Any {
    // TODO add callback message function
    fn update(&mut self, ctx: &mut Context);
}

pub trait DrawComponent {
    fn draw(&mut self, ctx: &mut Context);
}

/// this is useful because it allows 3D picking to ignore entities which aren't part of the
/// clickable environment
pub trait MouseComponent {
    // TODO think about x/y/z and hover events
    fn click_start(&mut self, ctx: &mut Context);
    fn click_end(&mut self, ctx: &mut Context);
}

pub struct EntityManager {
    entities: Vec<Box<dyn Entity>>,
    commands: Vec<Box<dyn Command>>,
}

// scaffolding to allow for undoable/redoable actions
pub trait Command {
    fn execute(&mut self);
    fn undo(&mut self);
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            entities: vec![],
            commands: vec![],
        }
    }

    pub fn update(&mut self, ctx: &mut Context) {
        self.entities
            .iter_mut()
            .for_each(|entity| entity.update(ctx));
    }
}
