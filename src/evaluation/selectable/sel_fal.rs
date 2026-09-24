use std::borrow::Cow;

use crate::{Features, evaluation::SelectableSource, spec::FatalError};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct SelFal<'f, D, F: Clone> {
    data: D,
    fallback: Cow<'f, F>,
}

impl<'f, D, F: Clone> AsRef<D> for SelFal<'f, D, F> {
    fn as_ref(&self) -> &D {
        &self.data
    }
}

impl<'f, D: SelectableSource, F: SelectableSource + Clone> SelectableSource for SelFal<'f, D, F> {
    async fn request<'a>(
        &'a self,
        selector: &crate::spec::Selector,
        allowed_features: Features,
    ) -> Result<Cow<'a, crate::spec::RuntimeAny>, crate::spec::FatalError> {
        match self.data.request(selector, allowed_features).await {
            Err(FatalError::RequestedSelectorValueNotAvailable(_)) => {
                self.fallback.request(selector, allowed_features).await
            }
            x => x,
        }
    }
}

impl<D, F: SelectableSource + Clone> SelFal<'static, D, F> {
    pub fn new_owned(data: D, fallback: F) -> Self {
        Self {
            data,
            fallback: Cow::Owned(fallback),
        }
    }
}

impl<D> SelFal<'static, D, ()> {
    pub fn new_data(data: D) -> Self {
        Self {
            data,
            fallback: Cow::Owned(()),
        }
    }
}

impl<D> From<D> for SelFal<'static, D, ()> {
    fn from(data: D) -> Self {
        SelFal {
            data,
            fallback: Cow::Owned(()),
        }
    }
}

impl<'f, D, F: SelectableSource + Clone> SelFal<'f, D, F> {
    pub fn fallback(&'f self) -> &'f F {
        &self.fallback
    }
    pub fn data(&self) -> &D {
        &self.data
    }
    pub fn into_data(self) -> D {
        self.data
    }

    pub fn new_ref(data: D, fallback: &'f F) -> Self {
        Self {
            data,
            fallback: Cow::Borrowed(fallback),
        }
    }

    pub fn new_cow(data: D, fallback: Cow<'f, F>) -> Self {
        Self { data, fallback }
    }

    pub fn with_ref_fallback<'g, G: SelectableSource + Clone>(
        &self,
        other_fallback: &'g G,
    ) -> SelFal<'g, D, G>
    where
        D: Clone,
    {
        SelFal {
            fallback: Cow::Borrowed(other_fallback),
            data: self.data.clone(),
        }
    }

    pub fn with_owned_fallback<G: SelectableSource + Clone>(
        &self,
        other_fallback: G,
    ) -> SelFal<'static, D, G>
    where
        D: Clone,
    {
        SelFal {
            fallback: Cow::Owned(other_fallback),
            data: self.data.clone(),
        }
    }

    pub fn map_data<X>(self, closure: impl FnOnce(D) -> X) -> SelFal<'f, X, F> {
        SelFal {
            data: closure(self.data),
            fallback: self.fallback,
        }
    }

    pub fn mutate_data(&mut self, closure: impl FnOnce(&mut D)) {
        closure(&mut self.data)
    }

    pub fn change_ref_fallback<'g, G: SelectableSource + Clone>(
        self,
        other_fallback: &'g G,
    ) -> SelFal<'g, D, G> {
        SelFal {
            data: self.data,
            fallback: Cow::Borrowed(other_fallback),
        }
    }
    pub fn change_owned_fallback<G: SelectableSource + Clone>(
        self,
        other_fallback: G,
    ) -> SelFal<'static, D, G> {
        SelFal {
            data: self.data,
            fallback: Cow::Owned(other_fallback),
        }
    }
}
