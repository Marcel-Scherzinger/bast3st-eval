use std::{collections::BTreeMap, sync::Arc};
mod marker;
mod network;
mod selector;
mod tasks;

pub use crate::spec::RuntimeAction;
use derive_more::{Deref, From};
pub use marker::MachineReturnVal;
pub(super) use marker::{
    CheapBorrowFromAny, ClarifiedCerrMerging, PossibleRuntimeValue, SpecializeFrom,
};
pub use network::{InnerNetworkRequest, NetworkRequest, NetworkResponse};
pub use selector::Selector;
pub use tasks::{CompiledRegex, SpecificTaskRequest};

use crate::spec::Numeric;
use crate::spec::Text;
use crate::{
    catchable::cerr,
    spec::{EntityId, MapKey, PrimitiveValue},
};

#[derive(Debug, Clone, From)]
pub enum RuntimeAny {
    Criterion(RuntimeCriterion),
    Action(RuntimeAction),
    Value(RuntimeValue),
    Catchable(cerr),
}

#[derive(Debug, Clone, Hash)]
pub struct RuntimeCriterion {
    is_fulfilled: bool,
    failure_explaination: Option<Text>,
}

impl RuntimeCriterion {
    pub fn new(is_fulfilled: bool, failure_explaination: Option<Text>) -> Self {
        Self {
            is_fulfilled,
            failure_explaination,
        }
    }
    pub fn new_fulfilled() -> Self {
        Self::new(true, None)
    }
    pub fn new_unfulfilled(failure_explaination: Option<Text>) -> Self {
        Self::new(true, failure_explaination)
    }
    pub fn is_fulfilled(&self) -> bool {
        self.is_fulfilled
    }
    pub fn is_not_fulfilled(&self) -> bool {
        !self.is_fulfilled
    }
    pub fn failure_explaination(&self) -> Option<&Text> {
        self.failure_explaination.as_ref()
    }
}

#[derive(Debug, Clone, Deref, PartialEq, PartialOrd, Default, From)]
pub struct Array(Arc<[RuntimeValue]>);
#[derive(Debug, Clone, Deref, PartialEq, PartialOrd, Default, From)]
pub struct Mapping(Arc<BTreeMap<MapKey, RuntimeValue>>);

impl<P: Into<RuntimeValue>> FromIterator<P> for Array {
    fn from_iter<T: IntoIterator<Item = P>>(iter: T) -> Self {
        Self(iter.into_iter().map(|x| x.into()).collect())
    }
}
impl From<BTreeMap<MapKey, RuntimeValue>> for Mapping {
    fn from(value: BTreeMap<MapKey, RuntimeValue>) -> Self {
        Self(value.into())
    }
}
impl From<Vec<RuntimeValue>> for Array {
    fn from(value: Vec<RuntimeValue>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
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

impl From<u16> for PrimitiveValue {
    fn from(value: u16) -> Self {
        Self::Number(Numeric::Int(value.into()))
    }
}
