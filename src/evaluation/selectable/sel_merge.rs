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

// TODO: inefficient
impl<A: SelectableSource + Copy, B: SelectableSource + Copy, C: SelectableSource + Copy>
    SelectableSource for (A, B, C)
where
    (A, (B, C)): SelectableSource,
{
    async fn request<'z>(
        &'z self,
        selector: &crate::spec::Selector,
        allowed_features: crate::Features,
    ) -> Result<Cow<'z, crate::spec::RuntimeAny>, FatalError> {
        (self.0, (self.1, self.2))
            .request(selector, allowed_features)
            .await
            .map(|c| Cow::Owned(c.into_owned()))
    }
}
