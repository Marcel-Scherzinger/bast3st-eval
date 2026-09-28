use scratch_test_model::{
    ProjectDoc,
    blocks::{BlockKindUnit, EventBlockKindUnit},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, PartialOrd, Clone, thiserror::Error, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InitialBlockAmbiguity {
    #[error("not found")]
    No,
    #[error("too many candidates")]
    Multiple,
}
pub const GREEN_FLAG: BlockKindUnit =
    BlockKindUnit::Event(EventBlockKindUnit::EventWhenflagclicked);

pub fn find_initial_block(
    doc: &ProjectDoc,
) -> Result<&scratch_test_model::Id, InitialBlockAmbiguity> {
    let mut green_flags = doc
        .ids_with_opcodes()
        .filter_map(|(id, opcode)| (opcode == GREEN_FLAG).then_some(id));
    let first = green_flags.next();
    if green_flags.next().is_some() {
        Err(InitialBlockAmbiguity::Multiple)
    } else {
        first.ok_or(InitialBlockAmbiguity::No)
    }
}
