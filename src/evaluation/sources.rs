use std::borrow::Cow;

use super::sel_fal::SelFal;
use crate::spec::{Array, FatalError, RealMapping, RuntimeAny, Selector};

use super::{Features, RequiredFeatures, SelectableSource};

#[derive(Debug, Clone)]
pub struct Rundata {
    input: RuntimeAny,     // Array
    output: RuntimeAny,    // Array
    randoms: RuntimeAny,   // Array
    lists: RuntimeAny,     // RealMapping
    variables: RuntimeAny, // RealMapping
}

pub type TestRundataSource<'f, Fallback> = SelFal<'f, Rundata, Fallback>;

impl<'f, F: SelectableSource + Clone> TestRundataSource<'f, F> {
    pub fn new(
        input: impl Into<Array>,
        output: impl Into<Array>,
        randoms: impl Into<Array>,
        lists: impl Into<RealMapping>,
        variables: impl Into<RealMapping>,
        fallback: Cow<'f, F>,
    ) -> Self {
        Self::new_cow(
            Rundata {
                input: input.into().into(),
                output: output.into().into(),
                randoms: randoms.into().into(),
                lists: lists.into().into(),
                variables: variables.into().into(),
            },
            fallback,
        )
    }
}

impl SelectableSource for Rundata {
    async fn request<'a>(
        &'a self,
        selector: &crate::spec::Selector,
        allowed_features: Features,
    ) -> Result<&'a crate::spec::RuntimeAny, crate::spec::FatalError> {
        Self::enforce_required(Features::READ_RUNDATA, allowed_features)?;

        Ok(match selector {
            Selector::Input => &self.input,
            Selector::Output => &self.output,
            Selector::Randoms => &self.randoms,
            Selector::Lists => &self.lists,
            Selector::Variables => &self.variables,
            Selector::Param | Selector::Blockcount | Selector::Flags => {
                Self::mark_fallback_need(selector.clone())?
            }
        })
    }
}
