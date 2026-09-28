use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::evaluation::{FatalRunError, initial_block::InitialBlockAmbiguity};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Error, Serialize, Deserialize)]
pub enum NoError {}

#[derive(Debug, PartialEq, PartialOrd, thiserror::Error, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProgramDocError {
    #[error("model: {_0}")]
    Model(String),
    #[error("initial-block/green-flag: {_0}")]
    InitialBlock(#[from] InitialBlockAmbiguity),
    #[error("run: {_0}")]
    FatalRun(#[from] FatalRunError),
}
