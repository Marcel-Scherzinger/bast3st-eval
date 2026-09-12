use std::{collections::BTreeMap, sync::Arc};
mod marker;
mod selector;

use derive_more::From;
pub use marker::MachineReturnVal;
pub(super) use marker::{CheapBorrowFromAny, PossibleRuntimeValue, Specialize, SpecializeFrom};
pub use selector::Selector;

use crate::{
    catchable::cerr,
    spec::{EntityId, PrimitiveValue},
};

#[derive(Debug, Clone)]
pub enum RuntimeAny {
    Criterion(RuntimeCriterion),
    Action(RuntimeAction),
    Value(RuntimeValue),
    Catchable(cerr),
}

#[derive(Debug, Clone, Hash)]
pub struct RuntimeCriterion(pub(super) bool);
#[derive(Debug, Clone, Hash)]
pub struct RuntimeAction;

#[derive(Debug, Clone)]
pub struct Array(Arc<[PrimitiveValue]>);
#[derive(Debug, Clone)]
pub struct Mapping(Arc<BTreeMap<PrimitiveValue, RuntimeValue>>);

#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Prim(PrimitiveValue),
    Array(Array),
    Mapping(Mapping),
}

impl<T> From<T> for RuntimeValue
where
    PrimitiveValue: From<T>,
{
    fn from(value: T) -> Self {
        Self::Prim(value.into())
    }
}
impl From<Array> for RuntimeValue {
    fn from(value: Array) -> Self {
        Self::Array(value)
    }
}
impl From<Mapping> for RuntimeValue {
    fn from(value: Mapping) -> Self {
        Self::Mapping(value)
    }
}
