mod entity_action;
mod entity_criterion;
mod entity_id;
mod entity_value;

use derive_more::From;
use serde::{Deserialize, Serialize};

use crate::spec::{Machine, RuntimeAny, Text};

pub use entity_action::*;
pub use entity_criterion::*;
pub use entity_id::*;
pub use entity_value::*;

#[derive(Debug, PartialEq, Serialize, Deserialize, From)]
#[serde(untagged)]
pub enum Entity {
    Criterion(CriterionEntity),
    Action(ActionEntity),
    Value(ValueEntity),
}

impl Entity {
    pub fn machine(&self) -> Machine<RuntimeAny> {
        match self {
            Self::Criterion(i) => i.machine().map(RuntimeAny::Criterion),
            Self::Action(i) => i.machine().map(RuntimeAny::Action),
            Self::Value(i) => i.machine().map(RuntimeAny::Value),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ValueReference {
    LitString(Text),
    EntityId(EntityId<ValueEntity>),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum MessageSeverity {
    Error,
    Warning,
    Info,
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "UPPERCASE")]
pub enum NetworkMethod {
    Get,
    Post,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum EndThisTestMode {
    Pass,
    Fail,
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum SetFlagMode {
    Keep,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum MessageSendingLevel {
    Spec,
    Category,
    Maintest,
}
