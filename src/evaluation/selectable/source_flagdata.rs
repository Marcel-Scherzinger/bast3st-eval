use std::{borrow::Cow, collections::BTreeMap};

use super::sel_fal::SelFal;
use crate::spec::{
    Array, Coremapping, FatalError, RealMapping, RuntimeAction, RuntimeAny, RuntimeValue, Selector,
    SetFlagMode, Text,
};

use super::{Features, SelectableSource};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct FlagData {
    flags: BTreeMap<Text, RuntimeValue>,
}
impl Extend<(Text, RuntimeValue)> for FlagData {
    fn extend<T: IntoIterator<Item = (Text, RuntimeValue)>>(&mut self, iter: T) {
        self.flags.extend(iter);
    }
}

impl SelectableSource for FlagData {
    async fn request<'a>(
        &'a self,
        selector: &crate::spec::Selector,
        allowed_features: Features,
    ) -> Result<&'a RuntimeAny, FatalError> {
        todo!()
    }
}

impl FlagData {
    pub fn extend_from_actions<'a>(
        &mut self,
        actions: impl IntoIterator<Item = &'a RuntimeAction>,
    ) {
        for action in actions {
            if let RuntimeAction::SetFlag { mode, key, value } = action {
                match mode {
                    SetFlagMode::Keep => self.flags.insert(key.clone(), value.clone().into()),
                };
            }
        }
    }
}
