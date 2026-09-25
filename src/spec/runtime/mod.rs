mod collections;
mod criterion;
mod marker;
mod network;
mod selector;
mod tasks;

pub use crate::spec::RuntimeAction;
pub use crate::spec::runtime::criterion::{InnerRuntimeCriterion, RuntimeCriterion};
pub use collections::{Array, MappingOrArray, RealMapping};
use derive_more::From;
pub(super) use marker::{CheapBorrowFromAny, ClarifiedCerrMerging, PossibleRuntimeValue};
pub(crate) use marker::{MachineReturnVal, SpecializeFrom};
pub use network::{InnerNetworkRequest, NetworkRequest, NetworkResponse};
pub use selector::Selector;
use serde::{Deserialize, Serialize};
pub use tasks::{CompiledRegex, SpecificTaskRequest};

use crate::spec::Numeric;
use crate::{catchable::cerr, spec::PrimitiveValue};

#[derive(Debug, PartialEq, PartialOrd, Clone, From)]
pub enum RuntimeAny<E = cerr> {
    #[from]
    Criterion(RuntimeCriterion),
    #[from]
    Action(RuntimeAction),
    #[from]
    Value(RuntimeValue),
    Catchable(E),
}
pub type RuntimeAnyWithoutCerr = RuntimeAny<std::convert::Infallible>;

impl From<cerr> for RuntimeAny {
    fn from(value: cerr) -> Self {
        Self::Catchable(value)
    }
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

#[derive(derive_more::Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RuntimeValue {
    #[debug("Prim({_0:?})")]
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

pub trait IntoRuntimeAny {
    fn into_runtimeany(self) -> RuntimeAny;
}
impl<T: Into<RuntimeAny>> IntoRuntimeAny for Result<T, cerr> {
    fn into_runtimeany(self) -> RuntimeAny {
        self.map_or_else(RuntimeAny::Catchable, |x| x.into())
    }
}
