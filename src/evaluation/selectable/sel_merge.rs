use std::borrow::Cow;

use crate::{evaluation::SelectableSource, spec::FatalError};

impl<A: SelectableSource, B: SelectableSource> SelectableSource for (A, B) {
    async fn request<'z>(
        &'z self,
        selector: &crate::spec::Selector,
        allowed_features: crate::Features,
    ) -> Result<Cow<'z, crate::spec::RuntimeAny>, FatalError> {
        match self.0.request(selector, allowed_features).await {
            Err(FatalError::RequestedSelectorValueNotAvailable(_)) => {
                self.1.request(selector, allowed_features).await
            }
            x => x,
        }
    }
}

impl<A: SelectableSource, B: SelectableSource, C: SelectableSource> SelectableSource for (A, B, C) {
    async fn request<'z>(
        &'z self,
        selector: &crate::spec::Selector,
        allowed_features: crate::Features,
    ) -> Result<Cow<'z, crate::spec::RuntimeAny>, FatalError> {
        match self.0.request(selector, allowed_features).await {
            Err(FatalError::RequestedSelectorValueNotAvailable(_)) => {
                match self.1.request(selector, allowed_features).await {
                    Err(FatalError::RequestedSelectorValueNotAvailable(_)) => {
                        self.2.request(selector, allowed_features).await
                    }
                    x => x,
                }
            }
            x => x,
        }
    }
}
