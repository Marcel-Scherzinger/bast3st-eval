mod entities;
mod hooks;
mod machine;
mod random_generation;
mod runtime;
mod structure;

use std::{cmp::Ordering, fmt::Debug, sync::Arc};

use derive_more::{Deref, From, Into};
use scratch_test_value::{SNumber, SValue};
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

impl PrimitiveValue {
    pub fn into_svalue(self) -> scratch_test_value::SValue {
        match self {
            Self::Str(t) => scratch_test_value::SValue::Text(t.into()),
            Self::Number(x) => scratch_test_value::SValue::from(x),
        }
    }
}

impl From<SValue> for PrimitiveValue {
    fn from(value: SValue) -> Self {
        match value {
            SValue::Int(x) => Self::Number(SNumber::Int(x)),
            SValue::Float(x) => Self::Number(SNumber::Float(x)),
            SValue::Text(x) => Self::Str(x.into()),
            SValue::Bool(true) => Self::Number(SNumber::Int(1)),
            SValue::Bool(false) => Self::Number(SNumber::Int(0)),
        }
    }
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
    derive_more::Display,
    PartialEq,
    PartialOrd,
    Deref,
    Clone,
    Serialize,
    Deserialize,
    Eq,
    Ord,
    From,
    Into,
    Hash,
)]
#[debug("{_0:?}")]
pub struct Text(Arc<str>);

impl Text {
    pub fn from_debug(x: impl Debug) -> Text {
        Text(format!("{x:?}").into())
    }
}

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

#[derive(
    derive_more::Debug, PartialEq, PartialOrd, Clone, From, Eq, Ord, Hash, Serialize, Deserialize,
)]
#[serde(untagged)]
pub enum MapKey {
    #[debug("{_0:?}")]
    Str(Text),
    #[debug("{_0:?}")]
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
