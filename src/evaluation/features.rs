use crate::spec::Selector;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Default)]
pub struct Features(u32);

impl std::fmt::Debug for Features {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        bitflags::parser::to_writer(self, f)
    }
}

bitflags::bitflags! {
    impl Features : u32 {
        /// Allow access to INPUT, OUTPUT, RANDOMS, LISTS, ...
        const READ_RUNDATA = 1 << 0;
        const END_THIS_TEST = 1 << 1;
        const SENDMSG_SPEC = 1 << 2;
        const SENDMSG_CATEGORY = 1 << 3;
        const SENDMSG_MAINTEST = 1 << 4;
        const SENDMSG_THISTEST = 1 << 5; // TODO: replace with current?

        const PermittedFEAT_TestScope =
              Self::SENDMSG_SPEC.0
            | Self::SENDMSG_CATEGORY.0
            | Self::SENDMSG_MAINTEST.0
            | Self::SENDMSG_THISTEST.0
            | Self::READ_RUNDATA.0;
        const PermittedFEAT_PassTestCrit =
              Self::PermittedFEAT_TestScope.0
            | Self::END_THIS_TEST.0;
    }
}

pub trait RequiredFeatures {
    fn required_features(&self) -> Features;
}
