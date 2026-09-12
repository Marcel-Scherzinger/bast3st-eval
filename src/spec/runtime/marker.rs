use std::{borrow::Cow, sync::Arc};

use crate::{
    catchable::cerr,
    spec::{
        ActionEntity, CriterionEntity, Entity, EntityId, Numeric, PrimitiveValue, Text,
        ValueReference,
        runtime::{Array, Mapping, RuntimeAction, RuntimeCriterion, RuntimeValue},
    },
};

use super::RuntimeAny;

mod _sealed {
    use std::borrow::Cow;

    use crate::{catchable::cerr, spec::runtime::RuntimeAny};

    pub trait Specialize: Clone {
        fn from_any<'a>(any: &'a RuntimeAny) -> Result<Cow<'a, Self>, cerr>
        where
            Self: Sized;
    }

    pub trait SpecializeFrom<Target>: Clone {
        fn specialize_from<'a>(any: &'a Target) -> Result<Cow<'a, Self>, cerr>
        where
            Self: Sized;
    }
    pub trait PossibleRuntimeValue<RuntimeT> {}
    pub trait CheapBorrowFromAny {}
}
pub(crate) use _sealed::{CheapBorrowFromAny, PossibleRuntimeValue, Specialize, SpecializeFrom};
pub trait MachineReturnVal {}

macro_rules! impl_specialize {
    ($ty: ty, $err: expr, $t: pat, $o: ident) => {
        impl Specialize for $ty {
            fn from_any<'a>(any: &'a RuntimeAny) -> Result<Cow<'a, Self>, cerr>
            where
                Self: Sized,
            {
                match any {
                    $t => Ok(Cow::Borrowed($o)),
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

impl_possible!(EntityId<Entity>:
    //
    RuntimeAny,
    //
    RuntimeAction,
    RuntimeCriterion,
    //
    RuntimeValue, PrimitiveValue, Array, Mapping,
    Text, Numeric
);
impl_possible!(EntityId<ActionEntity>: RuntimeAction, );
impl_possible!(EntityId<CriterionEntity>: RuntimeCriterion, );

impl_possible!(ValueReference:
    //
    RuntimeValue, PrimitiveValue, Array, Mapping, Text, Numeric
);

impl_marker!(
    CheapBorrowFromAny:
    RuntimeAny,
    RuntimeAction,
    RuntimeValue,
    RuntimeCriterion,
    PrimitiveValue,
    Array,
    Mapping,
    Text
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
impl<T: Clone> SpecializeFrom<T> for T {
    fn specialize_from<'a>(any: &'a T) -> Result<Cow<'a, Self>, cerr>
    where
        Self: Sized,
    {
        Ok(Cow::Borrowed(any))
    }
}

impl Specialize for Text {
    fn from_any<'a>(any: &'a RuntimeAny) -> Result<Cow<'a, Self>, cerr>
    where
        Self: Sized,
    {
        match any {
            RuntimeAny::Value(RuntimeValue::Prim(PrimitiveValue::Str(s))) => Ok(Cow::Borrowed(s)),
            _ => Err(cerr::typing_notText),
        }
    }
}
