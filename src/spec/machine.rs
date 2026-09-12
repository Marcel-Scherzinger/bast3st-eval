use std::{borrow::Cow, collections::BTreeMap, hash::Hash, sync::Arc};

use super::runtime::*;
use crate::{
    catchable::cerr,
    spec::{
        ActionEntity, CriterionEntity, Entity, EntityId, PrimitiveValue, Text, ValueEntity,
        ValueReference,
    },
};

pub type AllRuntimeValues = BTreeMap<EntityId<Entity>, RuntimeAny>;

pub type Pushable<R> = Box<dyn for<'a> FnOnce(&'a AllRuntimeValues) -> Machine<R>>;

pub enum InnerMachine<R> {
    Final(R),
    Pushable(Pushable<R>),
    Catchable(cerr),
}
pub enum FatalError {
    DeliveredValueStillMissing(EntityId<Entity>),
}

pub struct Machine<R> {
    /// value that is missing and needs to be computed first before this
    /// machine can continue
    missing_value: Option<EntityId<Entity>>,
    /// other nodes whose value changes will affect the value of this computation
    dependencies: Vec<EntityId<Entity>>,
    inner: Result<InnerMachine<R>, FatalError>,
}
impl<R> Machine<R> {
    pub fn from_final(v: R) -> Self {
        Self {
            missing_value: None,
            dependencies: vec![],
            inner: Ok(InnerMachine::Final(v)),
        }
    }
}

impl<R: 'static> Machine<R> {
    pub fn map<U>(self, closure: impl FnOnce(R) -> U + 'static) -> Machine<U> {
        let new_inner = match self.inner {
            Ok(InnerMachine::Pushable(inner_closure)) => {
                let mapped_closure = move |all: &AllRuntimeValues| -> Machine<U> {
                    let r_value: Machine<R> = inner_closure(all);
                    r_value.map(closure)
                };
                Ok(InnerMachine::Pushable(Box::new(mapped_closure)))
            }
            Ok(InnerMachine::Final(r)) => Ok(InnerMachine::Final(closure(r))),
            Ok(InnerMachine::Catchable(c)) => Ok(InnerMachine::Catchable(c)),
            Err(err) => Err(err),
        };
        Machine {
            missing_value: self.missing_value,
            dependencies: self.dependencies,
            inner: new_inner,
        }
    }
}

impl<R> From<FatalError> for Machine<R> {
    fn from(err: FatalError) -> Machine<R> {
        Machine {
            missing_value: None,
            dependencies: vec![],
            inner: Err(err),
        }
    }
}

impl<R: MachineReturnVal> From<R> for Machine<R> {
    fn from(value: R) -> Self {
        Self {
            missing_value: None,
            dependencies: vec![],
            inner: Ok(InnerMachine::Final(value)),
        }
    }
}

impl<'a, R: MachineReturnVal + Clone> From<Cow<'a, R>> for Machine<R> {
    fn from(value: Cow<'a, R>) -> Self {
        Self {
            missing_value: None,
            dependencies: vec![],
            inner: Ok(InnerMachine::Final(value.into_owned())),
        }
    }
}

impl<R> From<cerr> for Machine<R> {
    fn from(value: cerr) -> Self {
        Self {
            missing_value: None,
            dependencies: vec![],
            inner: Ok(InnerMachine::Catchable(value)),
        }
    }
}

pub trait MachineConstruction<E, RuntimeT>: PossibleRuntimeValue<RuntimeT> {
    fn and_then_cow<R>(
        &self,
        closure: impl for<'a> FnOnce(Cow<'a, RuntimeT>) -> Machine<R> + 'static,
    ) -> Machine<R>;
    fn and_then<R>(
        &self,
        closure: impl for<'a> FnOnce(RuntimeT) -> Machine<R> + 'static,
    ) -> Machine<R>
    where
        RuntimeT: Clone,
    {
        self.and_then_cow(|cow| closure(cow.into_owned()))
    }
    fn and_then_ref<R>(
        &self,
        closure: impl for<'a> FnOnce(&'a RuntimeT) -> Machine<R> + 'static,
    ) -> Machine<R>
    where
        RuntimeT: Clone + CheapBorrowFromAny,
    {
        self.and_then_cow(|cow| closure(&cow))
    }
}

impl<E, RuntimeT> MachineConstruction<E, RuntimeT> for EntityId<E>
where
    EntityId<E>: PossibleRuntimeValue<RuntimeT>,
    RuntimeT: Clone + Specialize,
    Entity: From<E>,
{
    fn and_then_cow<R>(
        &self,
        closure: impl for<'a> FnOnce(Cow<'a, RuntimeT>) -> Machine<R> + 'static,
    ) -> Machine<R> {
        let id: EntityId<Entity> = self.cast_id();
        let on_push = move |val: &AllRuntimeValues| {
            let Some(delivered) = val.get(&id) else {
                return FatalError::DeliveredValueStillMissing(id).into();
            };
            let converted: Cow<'_, RuntimeT> = match RuntimeT::from_any(delivered) {
                Ok(o) => o,
                Err(e) => return e.into(),
            };
            closure(converted)
        };
        Machine {
            missing_value: Some(id),
            dependencies: vec![],
            inner: Ok(InnerMachine::Pushable(Box::new(on_push))),
        }
    }
}

impl<RuntimeT> MachineConstruction<ValueReference, RuntimeT> for ValueReference
where
    ValueReference: PossibleRuntimeValue<RuntimeT>,
    RuntimeT: Clone + Specialize + /*SpecializeFrom<Text> +*/ 'static,
    EntityId<Entity>: PossibleRuntimeValue<RuntimeT>,
{
    fn and_then_cow<R>(
        &self,
        closure: impl for<'a> FnOnce(Cow<'a, RuntimeT>) -> Machine<R> + 'static,
    ) -> Machine<R> {
        match self {
            Self::EntityId(id) => id.cast_id::<Entity>().and_then_cow(closure),
            Self::LitString(text) => {
                let any = RuntimeAny::Value(RuntimeValue::Prim(PrimitiveValue::Str(text.clone())));
                let run = RuntimeT::from_any(&any).map(|c| c.into_owned());
                let on_push = move |val: &AllRuntimeValues| {
                    let converted: Cow<'_, RuntimeT> = match run {
                        Ok(o) => Cow::Owned(o),
                        Err(e) => return e.into(),
                    };
                    closure(converted)
                };
                Machine {
                    missing_value: None,
                    dependencies: vec![],
                    inner: Ok(InnerMachine::Pushable(Box::new(on_push))),
                }
            }
        }
    }
}

fn eval_if_then_else(
    if_: EntityId<CriterionEntity>,
    then_: EntityId<ActionEntity>,
    else_: EntityId<ActionEntity>,
) -> Machine<RuntimeAction> {
    if_.and_then_ref(move |if_: &RuntimeCriterion| {
        if if_.0 {
            then_.and_then(|then_: RuntimeAction| then_.clone().into())
        } else {
            else_.and_then_cow(|else_: Cow<'_, RuntimeAction>| else_.into())
        }
    })
}

fn eval_any_of(
    mut reversed_conditions: Vec<EntityId<CriterionEntity>>,
) -> Machine<RuntimeCriterion> {
    if let Some(x) = reversed_conditions.pop() {
        x.and_then_ref(|crit: &RuntimeCriterion| {
            if crit.0 {
                crit.clone().into()
            } else {
                eval_any_of(reversed_conditions)
            }
        })
    } else {
        RuntimeCriterion(false).into()
    }
}

fn eval_concat(
    mut reversed_exprs: Vec<ValueReference>,
    mut finished: String,
) -> Machine<RuntimeValue> {
    if let Some(x) = reversed_exprs.pop() {
        x.and_then_ref(|next: &Text| {
            finished.push_str(next);
            eval_concat(reversed_exprs, finished)
        })
    } else {
        Machine::from_final(RuntimeValue::Prim(PrimitiveValue::Str(finished.into())))
    }
}

pub trait MachineConstructionN<Closure, Extra, Ret> {
    fn and_then_n(self, closure: Closure) -> Machine<Ret>;
}

impl<C, E1, E2, R1, R2, Ret, T1: ToOwned, T2: ToOwned>
    MachineConstructionN<C, (E1, E2, R1, R2), Ret> for (&T1, &T2)
where
    C: FnOnce(R1, R2) -> Machine<Ret> + 'static,
    R1: Clone + 'static,
    R2: Clone + 'static,
    T1::Owned: MachineConstruction<E1, R1> + 'static,
    T2::Owned: MachineConstruction<E2, R2> + 'static,
{
    fn and_then_n(self, closure: C) -> Machine<Ret> {
        let t1 = self.0.to_owned();
        let t2 = self.1.to_owned();
        t1.and_then::<Ret>(move |r1: R1| t2.and_then::<Ret>(move |r2: R2| closure(r1, r2)))
    }
}
