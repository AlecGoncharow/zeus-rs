use core::entity::EntityKind;

#[allow(dead_code)]
pub struct EntityManager {
    pub entities: Vec<EntityKind>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self { entities: vec![] }
    }

    pub fn push_entity(&mut self, entity: EntityKind) {
        self.entities.push(entity);
    }
}
