mod checked;
mod normal;
mod on_task;

pub use checked::*;
pub(crate) use normal::cast_param;
pub use normal::*;
pub use on_task::OnTask;

use crate::{
    catchable::cerr,
    spec::{
        Entity, EntityId, RuntimeAction,
        runtime::{CompiledRegex, NetworkRequest, SpecificTaskRequest},
    },
};

pub type Param<'a> = Result<&'a crate::spec::runtime::RuntimeAny, FatalError>;

pub type Pushable<R> = Box<dyn for<'a> FnOnce(Param<'a>) -> Machine<R>>;
type TaskPushable<T, R> = Box<
    dyn for<'a> FnOnce(std::borrow::Cow<'a, <T as SpecificTaskRequest>::MainOutput>) -> Machine<R>,
>;

pub type Finished<R> =
    Result<Result<(R, (Vec<EntityId<Entity>>, Vec<RuntimeAction>)), cerr>, FatalError>;

#[derive(Debug)]
pub struct MachineWith<Status, R> {
    machine: Machine<R>,
    phantom: std::marker::PhantomData<Status>,
}

#[derive(Debug, derive_more::From)]
pub enum UnfinishedMachine<R> {
    Missing(MachineWith<MissingValue, R>),
    NetworkTask(MachineWithTask<NetworkRequest, R>),
    RegexTask(MachineWithTask<CompiledRegex, R>),
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum FatalError {}
