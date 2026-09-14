use std::{borrow::Cow, collections::BTreeMap, hash::Hash, sync::Arc};

use either::Either;

use crate::spec::runtime::*;
use crate::{
    catchable::cerr,
    spec::{
        ActionEntity, CriterionEntity, Entity, EntityId, PrimitiveValue, Text, ValueEntity,
        ValueReference, runtime::ClarifiedCerrMerging,
    },
};

pub type AllRuntimeValues = BTreeMap<EntityId<Entity>, RuntimeAny>;

pub type Param<'a> = Result<&'a RuntimeAny, FatalError>;

pub type Pushable<R> = Box<dyn for<'a> FnOnce(Param<'a>) -> Machine<R>>;

enum InnerMachine<R> {
    Final(R),
    Pushable(MissingValue, Pushable<R>),
    Catchable(cerr),
}
pub enum FatalError {
    DeliveredValueStillMissing(EntityId<Entity>),
}
/// value that is missing and needs to be computed first before this
/// machine can continue
#[derive(Debug, derive_more::From)]
pub(super) enum MissingValue {
    EntityId(EntityId<Entity>),
    Selector(Selector),
    /// This task first needs to be picked up and is then replaced by
    /// a [`Selector::TaskResult`] that contains an id making the resulting
    /// value of the required task avaiable
    SelectableTaskRequest(SelectableTaskRequest),
    /// The stored closure doesn't require any value at all and it is expected to *not*
    /// process that argument of type Param<'_> in any way. To ensure that no wrong
    /// values propagate, closures requiring no value will be called with a [`FatalError`]
    NoValueNeeded,
}

pub struct Machine<R> {
    /// other nodes whose value changes will affect the value of this computation
    dependencies: Vec<EntityId<Entity>>,
    inner: Result<InnerMachine<R>, FatalError>,
    actions: Vec<RuntimeAction>,
}

impl<R> Machine<R> {
    pub fn with_action(mut self, action: RuntimeAction) -> Self {
        self.actions.push(action);
        self
    }
    pub(super) fn from_pushable(id: impl Into<MissingValue>, pushable: Pushable<R>) -> Self {
        Self {
            dependencies: vec![],
            inner: Ok(InnerMachine::Pushable(id.into(), pushable)),
            actions: vec![],
        }
    }
    pub(super) fn from_task(task: impl Into<SelectableTaskRequest>, pushable: Pushable<R>) -> Self {
        Self::from_pushable(task.into(), pushable)
    }

    pub fn from_final(v: R) -> Self {
        Self {
            dependencies: vec![],
            inner: Ok(InnerMachine::Final(v)),
            actions: vec![],
        }
    }
}
impl<R: ClarifiedCerrMerging + 'static> Machine<R> {
    pub(super) fn from_res(result: Result<Machine<R>, Either<cerr, FatalError>>) -> Self {
        match result {
            Ok(r) => r,
            Err(err) => Self::from_err(err),
        }
        .maybe_merge_catchable()
    }
    pub(super) fn from_err(err: Either<cerr, FatalError>) -> Self {
        Self {
            dependencies: vec![],
            inner: err.map_left(InnerMachine::Catchable).flip().into(),
            actions: vec![],
        }
        .maybe_merge_catchable()
    }
}

pub(super) fn cast_param<'a, RuntimeT: SpecializeFrom>(
    param: Param<'a>,
) -> Result<Cow<'a, RuntimeT>, Either<cerr, FatalError>> {
    match param {
        Ok(any) => RuntimeT::specialize_from(any).map_err(Either::Left),
        Err(err) => Err(Either::Right(err)),
    }
}

impl<R: ClarifiedCerrMerging + 'static> Machine<R> {
    pub fn maybe_merge_catchable(self) -> Self {
        R::maybe_merge(self)
    }
    /// Alias of [`Self::and_then`]
    pub fn query<U>(self, closure: impl FnOnce(R) -> Machine<U> + 'static) -> Machine<U> {
        self.and_then(closure)
    }

    pub fn and_then<U>(self, closure: impl FnOnce(R) -> Machine<U> + 'static) -> Machine<U> {
        let new_inner = match self.inner {
            Ok(InnerMachine::Pushable(missing, inner_closure)) => {
                let new_inner = InnerMachine::Pushable(
                    missing,
                    Box::new(|param: Param<'_>| {
                        let first = inner_closure(param);
                        first.maybe_merge_catchable().and_then(closure)
                    }),
                );
                Ok(new_inner)
            }
            Ok(InnerMachine::Final(r)) => {
                let new_machine = closure(r);
                let (mut dependencies, mut actions) = (self.dependencies, self.actions);
                let inner = new_machine.inner;
                dependencies.extend(new_machine.dependencies);
                actions.extend(new_machine.actions);
                return Machine {
                    dependencies,
                    inner,
                    actions,
                };
            }
            Ok(InnerMachine::Catchable(c)) => Ok(InnerMachine::Catchable(c)),
            Err(err) => Err(err),
        };
        Machine {
            dependencies: self.dependencies,
            actions: self.actions,
            inner: new_inner,
        }
    }
    pub fn map<U>(self, closure: impl FnOnce(R) -> U + 'static) -> Machine<U> {
        let new_inner = match self.inner {
            Ok(InnerMachine::Pushable(missing, inner_closure)) => {
                let mapped_closure = move |val: Param| -> Machine<U> {
                    let r_value: Machine<R> = inner_closure(val);
                    r_value.maybe_merge_catchable().map(closure)
                };
                Ok(InnerMachine::Pushable(missing, Box::new(mapped_closure)))
            }
            Ok(InnerMachine::Final(r)) => Ok(InnerMachine::Final(closure(r))),
            Ok(InnerMachine::Catchable(c)) => Ok(InnerMachine::Catchable(c)),
            Err(err) => Err(err),
        };
        Machine {
            dependencies: self.dependencies,
            actions: self.actions,
            inner: new_inner,
        }
    }
}

impl<R: From<cerr>> Machine<R> {
    /// Types that can wrap [`cerr`] offer a second place inside [`Machine`]
    /// where errors can be stored. This method ensures that all catchable
    /// errors are stored as part of the main value and not in the Machine-specific
    /// variant for errors that exist for all wrapped types. This ensures
    /// that closures that explicitly want to process errors, still see all errors.
    ///
    /// This method is only available if the operation is useful, see [`Machine::maybe_merge_catchable`]
    /// for a version that works on more types.
    pub fn merge_catchables(mut self) -> Self {
        self.inner = self.inner.map(|inner| match inner {
            InnerMachine::Catchable(err) => InnerMachine::Final(R::from(err)),
            x => x,
        });
        self
    }
}

impl<R> From<FatalError> for Machine<R> {
    fn from(err: FatalError) -> Machine<R> {
        Machine {
            dependencies: vec![],
            actions: vec![],
            inner: Err(err),
        }
    }
}

impl<R: MachineReturnVal> From<R> for Machine<R> {
    fn from(value: R) -> Self {
        Self::from_final(value)
    }
}

impl<'a, R: MachineReturnVal + Clone> From<Cow<'a, R>> for Machine<R> {
    fn from(value: Cow<'a, R>) -> Self {
        Self::from_final(value.into_owned())
    }
}

impl<R: ClarifiedCerrMerging + 'static> From<cerr> for Machine<R> {
    fn from(value: cerr) -> Self {
        Self::from_err(Either::Left(value))
    }
}
