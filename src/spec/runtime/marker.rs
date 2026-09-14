use std::{borrow::Cow, sync::Arc};

use crate::{
    catchable::cerr,
    spec::{
        ActionEntity, CriterionEntity, Entity, EntityId, MapKey, Numeric, PrimitiveValue, Text,
        ValueReference,
        runtime::{Array, Mapping, RuntimeAction, RuntimeCriterion, RuntimeValue, Selector},
    },
};

use super::RuntimeAny;

mod _sealed {
    use std::borrow::Cow;

    use crate::{catchable::cerr, spec::runtime::RuntimeAny};

    pub trait SpecializeFrom<Target = RuntimeAny>: Clone {
        fn specialize_from<'a>(any: &'a Target) -> Result<Cow<'a, Self>, cerr>
        where
            Self: Sized;
    }
    pub trait PossibleRuntimeValue<RuntimeT> {}
    pub trait CheapBorrowFromAny {}
}
pub(crate) use _sealed::{CheapBorrowFromAny, PossibleRuntimeValue, SpecializeFrom};
use scratch_test_value::SNumber;
pub trait MachineReturnVal {}

macro_rules! impl_specialize {
    ($ty: ty, $err: expr, $t: pat, $o: ident) => {
        impl SpecializeFrom for $ty {
            fn specialize_from<'a>(any: &'a RuntimeAny) -> Result<Cow<'a, Self>, cerr>
            where
                Self: Sized,
            {
                match any {
                    $t => Ok(Cow::Borrowed($o)),
                    RuntimeAny::Catchable(old_err) => Err(*old_err),
                    _ => Err($err),
                }
            }
        }
    };
}

macro_rules! impl_possible {
    ( $type: ty: $($rt: ty),+ $(,)?) => {
        $(
            impl PossibleRuntimeValue<$rt> for $type {}
        )+
    };
}

macro_rules! impl_marker {
    ($marker: ident: $($type:ty),* $(,)?) => {
        $(
            impl $marker for $type {}
        )*
    };
}

// ###################################
// ### Definitions ###################
// ###################################

impl_specialize!(
    RuntimeAction,
    cerr::typing_notAction,
    RuntimeAny::Action(o),
    o
);
impl_specialize!(
    RuntimeCriterion,
    cerr::typing_notCriterion,
    RuntimeAny::Criterion(o),
    o
);
impl_specialize!(RuntimeValue, cerr::typing_notValue, RuntimeAny::Value(o), o);
impl_specialize!(
    PrimitiveValue,
    cerr::typing_notPrimitive,
    RuntimeAny::Value(RuntimeValue::Prim(o)),
    o
);
impl_specialize!(
    Numeric,
    cerr::typing_notNumeric,
    RuntimeAny::Value(RuntimeValue::Prim(PrimitiveValue::Number(o))),
    o
);

impl_specialize!(
    Array,
    cerr::typing_notArray,
    RuntimeAny::Value(RuntimeValue::Array(o)),
    o
);
impl_specialize!(
    Mapping,
    cerr::typing_notMapping,
    RuntimeAny::Value(RuntimeValue::Mapping(o)),
    o
);

impl_possible!(EntityId<Entity>:
    //
    RuntimeAny,
    //
    RuntimeAction,
    RuntimeCriterion,
    //
    RuntimeValue, PrimitiveValue, Array, Mapping,
    Text, Numeric, MapKey
);
impl_possible!(EntityId<ActionEntity>: RuntimeAction, );
impl_possible!(EntityId<CriterionEntity>: RuntimeCriterion, );

impl_possible!(ValueReference:
    RuntimeAny,
    //
    RuntimeValue, PrimitiveValue, Array, Mapping, Text, Numeric, MapKey
);
impl_possible!(Selector: RuntimeValue, Array, Mapping);

impl_marker!(
    CheapBorrowFromAny:
    RuntimeAny,
    RuntimeAction,
    RuntimeValue,
    RuntimeCriterion,
    PrimitiveValue,
    Array,
    Mapping,
    Text,
    SNumber
);
impl_marker!(MachineReturnVal: RuntimeAction, RuntimeCriterion, RuntimeValue);

impl SpecializeFrom<Text> for PrimitiveValue {
    fn specialize_from<'a>(any: &'a Text) -> Result<Cow<'a, Self>, cerr>
    where
        Self: Sized,
    {
        Ok(Cow::Owned(Self::Str(any.clone())))
    }
}

impl SpecializeFrom<RuntimeValue> for Array {
    fn specialize_from<'a>(any: &'a RuntimeValue) -> Result<Cow<'a, Self>, cerr>
    where
        Self: Sized,
    {
        match any {
            RuntimeValue::Array(a) => Ok(Cow::Borrowed(a)),
            _ => Err(cerr::typing_notArray),
        }
    }
}
impl SpecializeFrom<RuntimeValue> for Mapping {
    fn specialize_from<'a>(any: &'a RuntimeValue) -> Result<Cow<'a, Self>, cerr>
    where
        Self: Sized,
    {
        match any {
            RuntimeValue::Mapping(a) => Ok(Cow::Borrowed(a)),
            _ => Err(cerr::typing_notMapping),
        }
    }
}
impl SpecializeFrom for MapKey {
    fn specialize_from<'a>(any: &'a RuntimeAny) -> Result<Cow<'a, Self>, cerr>
    where
        Self: Sized,
    {
        match any {
            RuntimeAny::Value(RuntimeValue::Prim(p)) => MapKey::specialize_from(p),
            _ => Err(cerr::typing_notMapkey),
        }
    }
}
impl SpecializeFrom<PrimitiveValue> for MapKey {
    fn specialize_from<'a>(any: &'a PrimitiveValue) -> Result<Cow<'a, Self>, cerr>
    where
        Self: Sized,
    {
        match any {
            PrimitiveValue::Str(s) => Ok(Cow::Owned(Self::Str(s.clone()))),
            PrimitiveValue::Bool(b) => Ok(Cow::Owned(Self::Bool(*b))),
            PrimitiveValue::Number(Numeric::Int(i)) => Ok(Cow::Owned(Self::Int(*i))),
            _ => Err(cerr::typing_notMapkey),
        }
    }
}

impl<T: Clone> SpecializeFrom<T> for T {
    fn specialize_from<'a>(any: &'a T) -> Result<Cow<'a, Self>, cerr>
    where
        Self: Sized,
    {
        Ok(Cow::Borrowed(any))
    }
}

impl SpecializeFrom for Text {
    fn specialize_from<'a>(any: &'a RuntimeAny) -> Result<Cow<'a, Self>, cerr>
    where
        Self: Sized,
    {
        match any {
            RuntimeAny::Value(RuntimeValue::Prim(PrimitiveValue::Str(s))) => Ok(Cow::Borrowed(s)),
            _ => Err(cerr::typing_notText),
        }
    }
}
