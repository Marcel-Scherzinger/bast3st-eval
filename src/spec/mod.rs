mod entities;

mod hooks;
mod machine;
mod runtime;
mod structure;

use std::{cmp::Ordering, collections::BTreeMap, sync::Arc};

use derive_more::{Deref, From, Into};
use either::Either;
use serde::{Deserialize, Serialize};

use crate::catchable::cerr;

pub use entities::*;
pub use hooks::*;
pub use machine::{MachineConstruction, MachineConstructionN};
pub use structure::*;

#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize, Clone, From)]
#[serde(untagged)]
pub enum PrimitiveValue {
    Str(Text),
    Number(Numeric),
    Bool(bool),
}
pub type Numeric = scratch_test_value::SNumber;

#[derive(Debug, PartialEq, PartialOrd, Deref, Clone, Serialize, Deserialize, Eq, Ord, From)]
pub struct Text(Arc<str>);

impl From<String> for Text {
    fn from(value: String) -> Self {
        Self(value.into())
    }
}
