use std::borrow::Cow;

use either::Either;

use crate::Features;
use crate::spec::machine::definition::TaskPushable;
use crate::spec::machine::{
    FatalError, Finished, MachineWith, MachineWithTask, OnTask, Param, Pushable, UnfinishedMachine,
};
use crate::spec::{RuntimeAction, runtime::*};
use crate::{
    catchable::cerr,
    spec::{Entity, EntityId, runtime::ClarifiedCerrMerging},
};
pub(super) enum InnerMachine<R> {
    Final(R),
    Pushable(MissingValue, Pushable<R>),
    OnTask(OnTask<R>),
    Catchable(cerr),
}

/// value that is missing and needs to be computed first before this
/// machine can continue
#[derive(Debug, derive_more::From, PartialEq, PartialOrd, Clone)]
pub enum MissingValue {
    EntityId(EntityId<Entity>),
    Selector(Selector),
    /// The stored closure doesn't require any value at all and it is expected to *not*
    /// process that argument of type Param<'_> in any way.
    NoValueNeeded,
}

#[derive(derive_more::Debug, Default, PartialEq, PartialOrd)]
pub struct MachineMeta {
    pub(super) actions: Vec<RuntimeAction>,
    pub(super) features: Features,
    // other nodes whose value changes will affect the value of this computation
    // pub(super) dependencies: Vec<EntityId<Entity>>,
}

impl MachineMeta {
    pub fn extend_with(&mut self, other: MachineMeta) {
        // self.dependencies.extend(other.dependencies);
        self.actions.extend(other.actions);
        self.features |= other.features;
    }
    pub fn extended_with(mut self, other: MachineMeta) -> Self {
        self.extend_with(other);
        self
    }
    pub fn features(&self) -> Features {
        self.features
    }
    pub fn into_actions(self) -> Vec<RuntimeAction> {
        self.actions
    }
    pub fn actions(&self) -> &[RuntimeAction] {
        &self.actions
    }
}

#[derive(derive_more::Debug)]
#[debug("Machine {{ meta: {meta:?}, ...}}")]
pub struct Machine<R> {
    pub(super) meta: MachineMeta,
    pub(super) inner: Result<InnerMachine<R>, FatalError>,
}
impl<R> Machine<R> {
    pub(super) fn continued_from_meta(mut self, meta: MachineMeta) -> Machine<R> {
        self.meta.extend_with(meta);
        self
    }
}

impl<R: ClarifiedCerrMerging + 'static> Machine<R> {
    pub fn big_red_and_btn(self) -> Either<Finished<R>, UnfinishedMachine<R>> {
        match self.inner {
            Ok(InnerMachine::Final(val)) => Either::Left(Ok(Ok((val, self.meta)))),
            Ok(InnerMachine::Pushable(_, _)) => {
                Either::Right(UnfinishedMachine::Missing(MachineWith::new(self)))
            }
            Ok(InnerMachine::OnTask(task)) => Either::Right(match task {
                OnTask::CompiledRegex((task, closure)) => UnfinishedMachine::RegexTask(
                    MachineWithTask::into_task_specific(self.meta, task, closure),
                ),
                OnTask::NetworkRequest((task, closure)) => UnfinishedMachine::NetworkTask(
                    MachineWithTask::into_task_specific(self.meta, task, closure),
                ),
            }),
            Ok(InnerMachine::Catchable(err)) => Either::Left(Ok(Err(err))),
            Err(fatal) => Either::Left(Err(fatal)),
        }
    }
}

impl<R> Machine<R> {
    pub(super) fn inner(&self) -> Result<&InnerMachine<R>, &FatalError> {
        self.inner.as_ref()
    }
}

impl<R> Machine<R> {
    pub fn with_action(mut self, action: RuntimeAction) -> Self {
        self.meta.actions.push(action);
        self
    }
    /// Add extra features that need to be enabled for the current expression to be usable.
    /// Already present features will be ignored (bitwise or)
    pub fn require_features(mut self, required_features: Features) -> Self {
        self.meta.features |= required_features;
        self
    }
    pub(crate) fn from_pushable(id: impl Into<MissingValue>, pushable: Pushable<R>) -> Self {
        Self {
            meta: Default::default(),
            inner: Ok(InnerMachine::Pushable(id.into(), pushable)),
        }
    }
    pub(crate) fn from_task<T>(task: T, pushable: TaskPushable<T, R>) -> Self
    where
        OnTask<R>: From<(T, TaskPushable<T, R>)>,
        T: SpecificTaskRequest,
    {
        Self {
            meta: Default::default(),
            inner: Ok(InnerMachine::OnTask((task, pushable).into())),
        }
    }

    pub fn from_final(v: R) -> Self {
        Self {
            meta: Default::default(),
            inner: Ok(InnerMachine::Final(v)),
        }
    }
}
impl<R: ClarifiedCerrMerging + 'static> Machine<R> {
    pub(crate) fn from_res(result: Result<Machine<R>, Either<cerr, FatalError>>) -> Self {
        match result {
            Ok(r) => r,
            Err(err) => Self::from_err(err),
        }
        .maybe_merge_catchable()
    }
    pub(super) fn from_err(err: Either<cerr, FatalError>) -> Self {
        Self {
            meta: Default::default(),
            inner: err.map_left(InnerMachine::Catchable).flip().into(),
        }
        .maybe_merge_catchable()
    }
}

pub(crate) fn cast_param<'a, RuntimeT: SpecializeFrom>(
    param: Param<'a>,
) -> Result<Cow<'a, RuntimeT>, Either<cerr, FatalError>> {
    match param {
        Ok(any) => RuntimeT::specialize_from(Cow::Borrowed(any)).map_err(Either::Left),
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
                return Machine {
                    inner: new_machine.inner,
                    meta: self.meta.extended_with(new_machine.meta),
                };
            }
            Ok(InnerMachine::OnTask(on_task)) => {
                Ok(InnerMachine::OnTask(on_task.and_then(closure)))
            }
            Ok(InnerMachine::Catchable(c)) => Ok(InnerMachine::Catchable(c)),
            Err(err) => Err(err),
        };
        Machine {
            meta: self.meta,
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
            Ok(InnerMachine::OnTask(on_task)) => Ok(InnerMachine::OnTask(on_task.map(closure))),
            Ok(InnerMachine::Final(r)) => Ok(InnerMachine::Final(closure(r))),
            Ok(InnerMachine::Catchable(c)) => Ok(InnerMachine::Catchable(c)),
            Err(err) => Err(err),
        };
        Machine {
            meta: self.meta,
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
            meta: Default::default(),
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
impl<R: ClarifiedCerrMerging + 'static> From<Result<Machine<R>, cerr>> for Machine<R> {
    fn from(value: Result<Machine<R>, cerr>) -> Self {
        Self::from_res(value.map_err(Either::Left))
    }
}
impl<R: ClarifiedCerrMerging + 'static> From<Result<R, cerr>> for Machine<R> {
    fn from(value: Result<R, cerr>) -> Self {
        Self::from_res(value.map_err(Either::Left).map(Machine::from_final))
    }
}
