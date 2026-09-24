use std::borrow::Cow;

use derive_getters::Getters;
use itertools::Itertools;

use crate::{
    Features,
    evaluation::{FlagData, SelectableSource},
    spec::{FatalError, ProcessedAction, RuntimeAction, RuntimeAny},
};

#[derive(Debug, Default, Getters)]
pub struct Effects {
    actions: Vec<ProcessedAction>,
    flags: FlagData,
}

impl Effects {
    pub fn into_parts(self) -> (Vec<ProcessedAction>, FlagData) {
        (self.actions, self.flags)
    }
}

impl SelectableSource for Effects {
    async fn request<'a>(
        &'a self,
        selector: &crate::spec::Selector,
        allowed_features: Features,
    ) -> Result<Cow<'a, RuntimeAny>, FatalError> {
        self.flags.request(selector, allowed_features).await
    }
}

impl Extend<RuntimeAction> for Effects {
    fn extend<T: IntoIterator<Item = RuntimeAction>>(&mut self, iter: T) {
        let (other, processed): (Vec<RuntimeAction>, Vec<ProcessedAction>) = iter
            .into_iter()
            .partition_map(|item| item.into_processed().into());
        self.flags.extend(&other);
        self.actions.extend(processed);
    }
}
