use derive_more::Display;

use crate::{
    evaluation::RequiredFeatures,
    spec::{
        Coremapping, MachineConstruction, MapKey,
        machine::Machine,
        runtime::{PossibleRuntimeValue, RuntimeValue, SpecializeFrom},
    },
};

#[derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum Selector {
    #[display("{_0}")]
    Coremap(Coremapping),
    #[display("{mapping}{key:?}")]
    CoremapItem {
        mapping: Coremapping,
        key: Vec<MapKey>,
    },
}

impl Selector {
    pub const INPUT: Selector = Selector::Coremap(Coremapping::Input);
    pub const OUTPUT: Selector = Selector::Coremap(Coremapping::Output);
    pub const RANDOMS: Selector = Selector::Coremap(Coremapping::Randoms);
    pub const VARIABLES: Selector = Selector::Coremap(Coremapping::Variables);
    pub const LISTS: Selector = Selector::Coremap(Coremapping::Lists);
    pub const PARAM: Selector = Selector::Coremap(Coremapping::Param);
    pub const FLAGS: Selector = Selector::Coremap(Coremapping::Flags);

    pub fn std_machine<Out>(self) -> Machine<RuntimeValue>
    where
        Self: PossibleRuntimeValue<Out>,
        Out: SpecializeFrom + Send,
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
            Self::Coremap(mapping) | Self::CoremapItem { mapping, key: _ } => {
                mapping.required_features()
            }
        }
    }
}

impl From<Coremapping> for Selector {
    fn from(value: Coremapping) -> Self {
        Self::Coremap(value)
    }
}

impl RequiredFeatures for Coremapping {
    fn required_features(&self) -> crate::Features {
        match self {
            Self::Input | Self::Output | Self::Randoms | Self::Lists | Self::Variables => {
                crate::Features::READ_RUNDATA
            }
            Self::Param | Self::Flags => crate::Features::empty(),
        }
    }
}
