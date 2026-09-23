use std::collections::BTreeMap;

use scratch_test_model::{Id, ProjectDoc};

use crate::spec::{Entity, EntityId};

pub struct Context<'p, 'e> {
    max_list_length: u32,
    doc: &'p ProjectDoc,
    initial_block: &'p Id,
    entities: &'e BTreeMap<EntityId, Entity>,
}
impl<'p, 'e> Context<'p, 'e> {
    pub fn entities(&self) -> &'e BTreeMap<EntityId, Entity> {
        self.entities
    }
    pub fn initial_block(&self) -> &'p Id {
        self.initial_block
    }
    pub fn doc(&self) -> &'p ProjectDoc {
        self.doc
    }
    pub fn max_list_length(&self) -> u32 {
        self.max_list_length
    }
}
