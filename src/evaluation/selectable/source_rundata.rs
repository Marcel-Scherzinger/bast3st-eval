use std::borrow::Cow;

use crate::spec::{Array, Coremapping, RealMapping, RuntimeAny, Selector};

use super::{Features, SelectableSource};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Rundata {
    input: RuntimeAny,     // Array
    output: RuntimeAny,    // Array
    randoms: RuntimeAny,   // Array
    lists: RuntimeAny,     // RealMapping
    variables: RuntimeAny, // RealMapping
}

impl Rundata {
    pub fn new(
        input: impl Into<Array>,
        output: impl Into<Array>,
        randoms: impl Into<Array>,
        lists: impl Into<RealMapping>,
        variables: impl Into<RealMapping>,
    ) -> Self {
        Rundata {
            input: input.into().into(),
            output: output.into().into(),
            randoms: randoms.into().into(),
            lists: lists.into().into(),
            variables: variables.into().into(),
        }
    }
}

impl SelectableSource for Rundata {
    async fn request<'a>(
        &'a self,
        selector: &crate::spec::Selector,
        allowed_features: Features,
    ) -> Result<Cow<'a, crate::spec::RuntimeAny>, crate::spec::FatalError> {
        Self::enforce_required(Features::READ_RUNDATA, allowed_features)?;

        Ok(Cow::Borrowed(match selector {
            Selector::Coremap(mapping) => match mapping {
                Coremapping::Input => &self.input,
                Coremapping::Variables => &self.variables,
                Coremapping::Randoms => &self.randoms,
                Coremapping::Output => &self.output,
                Coremapping::Lists => &self.lists,
                Coremapping::Flags | Coremapping::Param => {
                    Self::mark_fallback_need(selector.clone())?
                }
            },
            Selector::CoremapItem { .. } => Self::mark_fallback_need(selector.clone())?,
        }))
    }
}
