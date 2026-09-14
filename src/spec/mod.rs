mod entities;

mod hooks;
mod machine;
mod runtime;
mod structure;

use runtime::Selector;
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

#[derive(
    Debug, PartialEq, PartialOrd, Deref, Clone, Serialize, Deserialize, Eq, Ord, From, Hash,
)]
pub struct Text(Arc<str>);

impl From<String> for Text {
    fn from(value: String) -> Self {
        Self(value.into())
    }
}

impl<'a> From<&'a str> for Text {
    fn from(value: &'a str) -> Self {
        Self(value.into())
    }
}

impl<'a> From<&'a String> for Text {
    fn from(value: &'a String) -> Self {
        Self(value.to_owned().into())
    }
}
impl<'a> From<&'a str> for PrimitiveValue {
    fn from(value: &'a str) -> Self {
        Self::Str(value.into())
    }
}
impl From<String> for PrimitiveValue {
    fn from(value: String) -> Self {
        Self::Str(value.into())
    }
}
impl<'a> From<&'a str> for MapKey {
    fn from(value: &'a str) -> Self {
        Self::Str(value.into())
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, From, Eq, Ord, Hash)]
pub enum MapKey {
    Str(Text),
    Int(i64),
    Bool(bool),
}
impl From<MapKey> for PrimitiveValue {
    fn from(value: MapKey) -> Self {
        match value {
            MapKey::Str(t) => Self::Str(t),
            MapKey::Int(i) => Self::Number(Numeric::Int(i)),
            MapKey::Bool(b) => Self::Bool(b),
        }
    }
}
