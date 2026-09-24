mod sel_fal;
mod sel_merge;
mod source_effects;
mod source_flagdata;
mod source_rundata;

use std::borrow::Cow;

#[allow(unused)]
pub use crate::evaluation::features::RequiredFeatures;

pub use sel_fal::*;
pub use source_effects::*;
pub use source_flagdata::*;
pub use source_rundata::*;

use crate::{
    evaluation::features::Features,
    spec::{FatalError, RuntimeAny, Selector},
};

pub trait SelectableSource: Send + Sync {
    fn enforce_required(required: Features, provided: Features) -> Result<(), FatalError> {
        if provided.contains(required) {
            Ok(())
        } else {
            Err(FatalError::FeatureMissmatch { required, provided })
        }
    }
    fn mark_fallback_need<O>(selector: Selector) -> Result<O, FatalError> {
        Err(FatalError::RequestedSelectorValueNotAvailable(selector))
    }

    /// This is a way to request the value for a selector
    /// and explicitly state which [`Features`] are allowed for the
    /// specific evaluation at hand
    fn request<'a>(
        &'a self,
        selector: &Selector,
        allowed_features: Features,
    ) -> impl Future<Output = Result<Cow<'a, RuntimeAny>, FatalError>> + Send;
}

/// `()` can be used as a fallback [`SelectableSource`] that always returns
/// `Err(`[`FatalError::RequestedSelectorValueNotAvailable`]`(selector.clone()))`
impl SelectableSource for () {
    async fn request<'a>(
        &'a self,
        selector: &Selector,
        _allowed_features: Features,
    ) -> Result<Cow<'a, RuntimeAny>, FatalError> {
        Err(FatalError::RequestedSelectorValueNotAvailable(
            selector.clone(),
        ))
    }
}

impl<A: SelectableSource> SelectableSource for &mut A {
    async fn request<'z>(
        &'z self,
        selector: &crate::spec::Selector,
        allowed_features: crate::Features,
    ) -> Result<Cow<'z, crate::spec::RuntimeAny>, FatalError> {
        <A as SelectableSource>::request(self, selector, allowed_features).await
    }
}

impl<A: SelectableSource> SelectableSource for &A {
    async fn request<'z>(
        &'z self,
        selector: &crate::spec::Selector,
        allowed_features: crate::Features,
    ) -> Result<Cow<'z, crate::spec::RuntimeAny>, FatalError> {
        <A as SelectableSource>::request(self, selector, allowed_features).await
    }
}

impl<A: SelectableSource> SelectableSource for Option<A> {
    async fn request<'a>(
        &'a self,
        selector: &Selector,
        allowed_features: Features,
    ) -> Result<Cow<'a, RuntimeAny>, FatalError> {
        if let Some(x) = self {
            x.request(selector, allowed_features).await
        } else {
            ().request(selector, allowed_features).await
        }
    }
}
