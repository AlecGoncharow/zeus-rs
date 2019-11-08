use my_engine::context::Context;

pub mod component;
use component::AsComponent;

pub mod cube;

enum Message {
    Foo,
    Bar,
}

pub trait Entity: AsComponent {
    // TODO add callback message function
    fn update(&mut self, ctx: &mut Context);
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

    pub fn draw(&mut self, ctx: &mut Context) {
        self.entities.iter_mut().for_each(|entity| {
            if let Some(drawable) = entity.as_drawable() {
                drawable.draw(ctx);
            }
        });
    }

    pub fn push_entity(&mut self, entity: impl Entity + 'static) {
        self.entities.push(Box::new(entity));
    }
}
