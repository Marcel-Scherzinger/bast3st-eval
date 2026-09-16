use crate::spec::{
    MachineConstruction,
    machine::Machine,
    runtime::{PossibleRuntimeValue, RuntimeValue, SpecializeFrom},
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum Selector {
    Input,
    Output,
    Randoms,
    Lists,
    Variables,
    Blockcount,
    Param,
    Flags,
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
