mod sel_fal;
mod source_flagdata;
mod source_rundata;

pub use crate::evaluation::features::RequiredFeatures;

pub use sel_fal::*;
pub use source_flagdata::*;
pub use source_rundata::*;

use crate::{
    evaluation::features::Features,
    spec::{FatalError, RuntimeAny, Selector},
};

pub trait SelectableSource {
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
    ) -> impl Future<Output = Result<&'a RuntimeAny, FatalError>>;
}

/// `()` can be used as a fallback [`SelectableSource`] that always returns
/// `Err(`[`FatalError::RequestedSelectorValueNotAvailable`]`(selector.clone()))`
impl SelectableSource for () {
    async fn request<'a>(
        &'a self,
        selector: &Selector,
        allowed_features: Features,
    ) -> Result<&'a RuntimeAny, FatalError> {
        Err(FatalError::RequestedSelectorValueNotAvailable(
            selector.clone(),
        ))
    }
}
