use std::sync::Arc;

use crate::spec::{
    MachineConstruction, Text,
    machine::Machine,
    runtime::{
        PossibleRuntimeValue, RuntimeAny, RuntimeValue, SpecializeFrom, network::NetworkRequest,
    },
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct TaskResultId(u64);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum Selector {
    Input,
    Output,
    Randoms,
    Lists,
    Variables,
    Blockcount,
    Param,
    Flags,
    TaskResult(TaskResultId),
}

#[derive(Debug, PartialEq, PartialOrd, Clone, derive_more::From)]
pub enum SelectableTaskRequest {
    NetworkRequest(NetworkRequest),
}

impl Selector {
    pub fn std_machine<Out>(&self) -> Machine<RuntimeValue>
    where
        Self: PossibleRuntimeValue<Out>,
        Out: SpecializeFrom,
        RuntimeValue: From<Out>,
    {
        self.query(|x: Out| Machine::from_final(RuntimeValue::from(x)))
    }
}
