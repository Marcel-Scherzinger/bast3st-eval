use crate::{
    evaluation::RequiredFeatures,
    spec::{
        MachineConstruction,
        machine::Machine,
        runtime::{PossibleRuntimeValue, RuntimeValue, SpecializeFrom},
    },
};

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
}

impl Selector {
    pub fn std_machine<Out>(self) -> Machine<RuntimeValue>
    where
        Self: PossibleRuntimeValue<Out>,
        Out: SpecializeFrom,
        RuntimeValue: From<Out>,
    {
        let required = self.required_features();
        self.query(|x: Out| Machine::from_final(RuntimeValue::from(x)))
            .require_features(required)
    }
}

impl RequiredFeatures for Selector {
    fn required_features(&self) -> crate::Features {
        match self {
            Self::Input | Self::Output | Self::Randoms | Self::Lists | Self::Variables => {
                crate::Features::READ_RUNDATA
            }
            Self::Param | Self::Flags | Self::Blockcount => crate::Features::empty(),
        }
    }
}
