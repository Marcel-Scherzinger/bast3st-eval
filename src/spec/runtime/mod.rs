use std::{collections::BTreeMap, sync::Arc};
mod marker;
mod selector;

use derive_more::{Deref, From};
pub use marker::MachineReturnVal;
pub(super) use marker::{CheapBorrowFromAny, PossibleRuntimeValue, SpecializeFrom};
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

#[derive(Debug, Clone, Deref)]
pub struct Array(Arc<[RuntimeValue]>);
#[derive(Debug, Clone, Deref)]
pub struct Mapping(Arc<BTreeMap<PrimitiveValue, RuntimeValue>>);

impl<P: Into<RuntimeValue>> FromIterator<P> for Array {
    fn from_iter<T: IntoIterator<Item = P>>(iter: T) -> Self {
        Self(iter.into_iter().map(|x| x.into()).collect())
    }
}

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
