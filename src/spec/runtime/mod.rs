use std::borrow::Cow;
use std::{collections::BTreeMap, sync::Arc};
mod collections;
mod criterion;
mod marker;
mod network;
mod selector;
mod tasks;

pub use crate::spec::RuntimeAction;
pub use crate::spec::runtime::criterion::{InnerRuntimeCriterion, RuntimeCriterion};
use bitflags::iter::IterNames;
pub use collections::{Array, MappingOrArray, RealMapping};
use derive_more::{Deref, From};
use either::Either;
pub use marker::MachineReturnVal;
pub(super) use marker::{
    CheapBorrowFromAny, ClarifiedCerrMerging, PossibleRuntimeValue, SpecializeFrom,
};
pub use network::{InnerNetworkRequest, NetworkRequest, NetworkResponse};
pub use selector::Selector;
use serde_json::Map;
pub use tasks::{CompiledRegex, SpecificTaskRequest};

use crate::spec::Numeric;
use crate::{
    catchable::cerr,
    spec::{MapKey, PrimitiveValue},
};

#[derive(Debug, Clone, From)]
pub enum RuntimeAny {
    Criterion(RuntimeCriterion),
    Action(RuntimeAction),
    Value(RuntimeValue),
    Catchable(cerr),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, From, Default)]
pub enum MaybeEval<T> {
    #[default]
    Unevaluated,
    Evaluated(T),
}
impl<T> MaybeEval<T> {
    pub fn as_eval(&self) -> Option<&T> {
        if let Self::Evaluated(t) = self {
            Some(t)
        } else {
            None
        }
    }
    pub fn into_eval(self) -> Option<T> {
        if let Self::Evaluated(t) = self {
            Some(t)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum RuntimeValue {
    Prim(PrimitiveValue),
    Comp(MappingOrArray),
}

impl<T> From<T> for RuntimeValue
where
    PrimitiveValue: From<T>,
{
    fn from(value: T) -> Self {
        Self::Prim(value.into())
    }
}

impl From<MappingOrArray> for RuntimeValue {
    fn from(value: MappingOrArray) -> Self {
        Self::Comp(value)
    }
}
impl From<Array> for RuntimeValue {
    fn from(value: Array) -> Self {
        Self::Comp(value.into())
    }
}
impl From<RealMapping> for RuntimeValue {
    fn from(value: RealMapping) -> Self {
        Self::Comp(value.into())
    }
}

impl From<u16> for PrimitiveValue {
    fn from(value: u16) -> Self {
        Self::Number(Numeric::Int(value.into()))
    }
}

impl From<RealMapping> for RuntimeAny {
    fn from(value: RealMapping) -> Self {
        Self::Value(value.into())
    }
}
impl From<Array> for RuntimeAny {
    fn from(value: Array) -> Self {
        Self::Value(value.into())
    }
}
