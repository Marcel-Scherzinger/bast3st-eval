use std::borrow::Cow;

use crate::spec::{
    Entity, EntityId, RuntimeAction, RuntimeAny, SpecificTaskRequest,
    machine::{
        FatalError, Machine, MachineWith, MissingValue,
        definition::{InnerMachine, TaskPushable},
    },
    runtime::ClarifiedCerrMerging,
};

impl<Status, R> MachineWith<Status, R> {
    pub(super) fn new(machine: Machine<R>) -> Self {
        Self {
            machine,
            phantom: Default::default(),
        }
    }
    pub fn delay(self) -> Machine<R> {
        self.machine
    }
}

impl<R> MachineWith<MissingValue, R> {
    pub fn missing_value(&self) -> &MissingValue {
        match self.machine.inner() {
            Ok(InnerMachine::Pushable(missing, _)) => missing,
            _ => unreachable!(),
        }
    }
    pub fn call(self, any: Result<&RuntimeAny, FatalError>) -> Machine<R> {
        match self.machine.inner {
            Ok(InnerMachine::Pushable(_, closure)) => {
                closure(any).continued_from(self.machine.dependencies, self.machine.actions)
            }
            _ => unreachable!(),
        }
    }
}

#[derive(derive_more::Debug)]
#[debug("MachineWithTask {{ request: {request:?}, ... }}")]
pub struct MachineWithTask<T, R>
where
    T: SpecificTaskRequest,
{
    pub request: T,
    pub closure: TaskPushable<T, R>,
}

impl<T: SpecificTaskRequest, R: ClarifiedCerrMerging + 'static> MachineWithTask<T, R>
where
    <T as SpecificTaskRequest>::MainOutput: 'static,
{
    pub(super) fn into_task_specific(
        deps: Vec<EntityId<Entity>>,
        actions: Vec<RuntimeAction>,
        task: T,
        closure: TaskPushable<T, R>,
    ) -> Self {
        MachineWithTask {
            request: task,
            closure: Box::new({
                move |out| {
                    let new_machine = closure(out).maybe_merge_catchable();
                    new_machine.continued_from(deps, actions)
                }
            }),
        }
    }
    pub fn call<'a>(self, val: Cow<'a, <T as SpecificTaskRequest>::MainOutput>) -> Machine<R> {
        (self.closure)(val)
    }
}

impl<Status, R> From<MachineWith<Status, R>> for Machine<R> {
    fn from(value: MachineWith<Status, R>) -> Self {
        value.delay()
    }
}
