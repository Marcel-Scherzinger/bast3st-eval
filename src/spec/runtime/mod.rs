use std::collections::BTreeMap;
mod marker;

pub use marker::MachineReturnVal;
pub(super) use marker::{CheapBorrowFromAny, PossibleRuntimeValue, Specialize, SpecializeFrom};

use crate::{
    catchable::cerr,
    spec::{EntityId, PrimitiveValue},
};

#[derive(Debug, Clone)]
pub enum RuntimeAny {
    Criterion(RuntimeCriterion),
    Action(RuntimeAction),
    Value(RuntimeValue),
}

#[derive(Debug, Clone, Hash)]
pub struct RuntimeCriterion(pub(super) bool);
#[derive(Debug, Clone, Hash)]
pub struct RuntimeAction;

#[derive(Debug, Clone)]
pub struct Array(Vec<PrimitiveValue>);
#[derive(Debug, Clone)]
pub struct Mapping(BTreeMap<PrimitiveValue, RuntimeValue>);

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
