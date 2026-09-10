mod entities;
mod hooks;
mod structure;

use std::{cmp::Ordering, collections::BTreeMap};

use derive_more::{From, Into};
use either::Either;
use serde::{Deserialize, Serialize};

use crate::catchable::cerr;

pub use entities::*;
pub use hooks::*;
pub use structure::*;

#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PrimitiveValue {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}
