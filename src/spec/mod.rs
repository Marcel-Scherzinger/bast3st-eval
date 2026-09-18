mod entities;
mod hooks;
mod machine;
mod random_generation;
mod runtime;
mod structure;

use std::{cmp::Ordering, sync::Arc};

use derive_more::{Deref, From};
use serde::{Deserialize, Serialize};

pub use entities::*;
pub use hooks::*;
pub use machine::{
    FatalError, Machine, MachineConstruction, MachineConstructionN, MissingValue, UnfinishedMachine,
};
pub use random_generation::RandomGeneration;
pub use runtime::*;
pub use structure::*;

#[derive(derive_more::Debug, Serialize, Deserialize, Clone, From)]
#[serde(untagged)]
pub enum PrimitiveValue {
    // Bool(bool),
    #[debug("{_0:?}")]
    Str(Text),
    #[debug("{_0}")]
    Number(Numeric),
}

impl PartialEq for PrimitiveValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Str(a), Self::Str(b)) => a == b,
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::Str(s), Self::Number(n)) | (Self::Number(n), Self::Str(s)) => {
                if let Ok(i) = s.parse() {
                    Numeric::Int(i) == *n
                } else if let Ok(f) = s.parse() {
                    Numeric::Float(f) == *n
                } else {
                    false
                }
            }
        }
    }
}
impl PartialOrd for PrimitiveValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Str(a), Self::Str(b)) => a.partial_cmp(b),
            (Self::Number(a), Self::Number(b)) => a.partial_cmp(b),
            (Self::Str(s), Self::Number(n)) => s.as_ref().partial_cmp(n.to_string().as_str()),
            (Self::Number(n), Self::Str(s)) => n.to_string().as_str().partial_cmp(s.as_ref()),
        }
    }
}

impl PrimitiveValue {
    pub fn into_text(self) -> Text {
        match self {
            Self::Str(t) => t,
            Self::Number(n) => n.to_string().into(),
            // Self::Bool(b) => b.to_string().into(),
        }
    }
}

pub type Numeric = scratch_test_value::SNumber;

#[derive(
    derive_more::Debug,
    PartialEq,
    PartialOrd,
    Deref,
    Clone,
    Serialize,
    Deserialize,
    Eq,
    Ord,
    From,
    Hash,
)]
#[debug("{_0:?}")]
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
    // Bool(bool),
}
impl MapKey {
    pub fn into_text(self) -> Text {
        match self {
            Self::Str(t) => t,
            Self::Int(i) => i.to_string().into(),
            // Self::Bool(b) => b.to_string().into(),
        }
    }
    pub fn to_prim(&self) -> PrimitiveValue {
        self.clone().into()
    }
}

impl From<MapKey> for PrimitiveValue {
    fn from(value: MapKey) -> Self {
        match value {
            MapKey::Str(t) => Self::Str(t),
            MapKey::Int(i) => Self::Number(Numeric::Int(i)),
            // MapKey::Bool(b) => Self::Bool(b),
        }
    }
}
