use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Default)]
pub struct Features(u32);

impl std::fmt::Debug for Features {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        bitflags::parser::to_writer(self, f)
    }
}

impl Serialize for Features {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        bitflags::serde::serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for Features {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        bitflags::serde::deserialize(deserializer)
    }
}

bitflags::bitflags! {
    impl Features : u32 {
        /// Allow access to INPUT, OUTPUT, RANDOMS, LISTS, ...
        const READ_RUNDATA = 1 << 0;
        /// End a current test (available in test hooks and tests)
        const END_THIS_TEST = 1 << 1;
        /// Send global message (available if message sending is allowed)
        const SENDMSG_SPEC = 1 << 2;
        /// Send message to category (available in category and its tests)
        const SENDMSG_CATEGORY = 1 << 3;
        /// Send message to the maintest level (only in main- and alternative test)
        const SENDMSG_MAINTEST = 1 << 4;
        /// Send message to the current entity level
        const SENDMSG_CURRENT = 1 << 5;

        const PermittedFEAT_SpecHook = Self::SENDMSG_SPEC.0 | Self::SENDMSG_CURRENT.0;
        const PermittedFEAT_CategoryHook =
              Self::SENDMSG_SPEC.0
            | Self::SENDMSG_CATEGORY.0
            | Self::SENDMSG_CURRENT.0;

        const PermittedFEAT_PreTestHook =
              Self::END_THIS_TEST.0
            | Self::SENDMSG_SPEC.0
            | Self::SENDMSG_CATEGORY.0
            | Self::SENDMSG_MAINTEST.0
            | Self::SENDMSG_CURRENT.0;
        const PermittedFEAT_TestScope =
              Self::PermittedFEAT_PreTestHook.0
            | Self::READ_RUNDATA.0;
        const PermittedFEAT_PassTestCrit = Self::PermittedFEAT_TestScope.0;
        const PermittedFEAT_PostTestHook = Self::PermittedFEAT_TestScope.0;
    }
}

pub trait RequiredFeatures {
    fn required_features(&self) -> Features;
}
