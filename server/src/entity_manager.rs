use atlas::entity::EntityKind;

#[allow(dead_code)]
pub struct EntityManager<'a> {
    pub entities: Vec<EntityKind<'a>>,
}

impl<'a> EntityManager<'a> {
    pub fn new() -> Self {
        Self { entities: vec![] }
    }

    pub fn push_entity(&mut self, entity: EntityKind<'a>) {
        self.entities.push(entity);
    }
}
