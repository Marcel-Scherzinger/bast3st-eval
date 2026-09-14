use std::sync::Arc;

use crate::{
    catchable::cerr,
    spec::{
        MachineConstruction, Text,
        machine::Machine,
        runtime::{
            NetworkResponse, PossibleRuntimeValue, RuntimeAny, RuntimeValue, SpecializeFrom,
            network::NetworkRequest,
        },
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
    CompiledRegex(CompiledRegex),
}

pub trait SpecificTaskRequest {
    type MainOutput: Clone;
}
impl SpecificTaskRequest for NetworkRequest {
    type MainOutput = NetworkResponse;
}
#[derive(Debug, PartialEq, PartialOrd, Clone, derive_more::From)]
pub struct CompiledRegex(Text);
impl SpecificTaskRequest for CompiledRegex {
    type MainOutput = regex::Regex;
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
