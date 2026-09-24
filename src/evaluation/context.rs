use std::{collections::BTreeMap, sync::Arc};

use scratch_test_interpreter::Limits;
use scratch_test_model::{Id, ProjectDoc};

use crate::spec::{Entity, EntityId};

#[derive(Debug, Clone)]
pub struct Context {
    max_list_length: u32,
    limits: Limits,
    doc: ProjectDoc,
    initial_block: Id,
    entities: Arc<BTreeMap<EntityId, Entity>>,
}
impl Context {
    pub fn entities(&self) -> &BTreeMap<EntityId, Entity> {
        &self.entities
    }
    pub fn initial_block(&self) -> &Id {
        &self.initial_block
    }
    pub fn doc(&self) -> &ProjectDoc {
        &self.doc
    }
    pub fn max_list_length(&self) -> u32 {
        self.max_list_length
    }
    pub fn limits(&self) -> &Limits {
        &self.limits
    }
}
