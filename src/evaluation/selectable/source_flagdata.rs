use std::{
    borrow::Cow,
    collections::{BTreeMap, VecDeque},
};

use crate::spec::{
    Coremapping, FatalError, IntoRuntimeAny, MapKey, RealMapping, RuntimeAction, RuntimeAny,
    RuntimeValue, Selector, SetFlagMode, Text,
};

use super::{Features, SelectableSource};

#[derive(derive_more::Debug, PartialEq, PartialOrd, Clone)]
pub struct MappingData<T> {
    data: RuntimeAny, // RealMapping
    #[debug(skip)]
    _phantom: std::marker::PhantomData<T>,
}

#[derive(derive_more::Debug, PartialEq, PartialOrd, Clone)]
pub struct SourceFlag(());
#[derive(derive_more::Debug, PartialEq, PartialOrd, Clone)]
pub struct SourceParam(());

pub type FlagData = MappingData<SourceFlag>;
pub type ParamData = MappingData<SourceParam>;

impl<X> Default for MappingData<X> {
    fn default() -> Self {
        Self {
            data: RuntimeAny::Value(RealMapping::default().into()),
            _phantom: Default::default(),
        }
    }
}

impl<X> From<BTreeMap<MapKey, RuntimeValue>> for MappingData<X> {
    fn from(value: BTreeMap<MapKey, RuntimeValue>) -> Self {
        Self {
            data: RuntimeAny::Value(RealMapping::from(value).into()),
            _phantom: Default::default(),
        }
    }
}

impl<X> From<RealMapping> for MappingData<X> {
    fn from(value: RealMapping) -> Self {
        Self {
            data: RuntimeAny::Value(value.into()),
            _phantom: Default::default(),
        }
    }
}

impl<X> From<BTreeMap<Text, RuntimeValue>> for MappingData<X> {
    fn from(value: BTreeMap<Text, RuntimeValue>) -> Self {
        Self::default().extended(value)
    }
}

impl<X, K: Into<MapKey>, V: Into<RuntimeValue>> Extend<(K, V)> for MappingData<X> {
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        let flags: RealMapping = self.as_ref().clone();
        self.data = flags
            .with_new_keys(
                iter.into_iter()
                    .map(|(text, val)| (vec![text.into()].into_iter().collect(), val.into())),
            )
            .into();
    }
}

impl<X, K: Into<MapKey>, V: Into<RuntimeValue>> Extend<(VecDeque<K>, V)> for MappingData<X> {
    fn extend<T: IntoIterator<Item = (VecDeque<K>, V)>>(&mut self, iter: T) {
        let flags: RealMapping = self.as_ref().clone();
        self.data = flags
            .with_new_keys(
                iter.into_iter()
                    .map(|(text, val)| (text.into_iter().map(|x| x.into()).collect(), val.into())),
            )
            .into();
    }
}

impl<X, K: Into<MapKey>, V: Into<RuntimeValue>> Extend<(Vec<K>, V)> for MappingData<X> {
    fn extend<T: IntoIterator<Item = (Vec<K>, V)>>(&mut self, iter: T) {
        let flags: RealMapping = self.as_ref().clone();
        self.data = flags
            .with_new_keys(iter.into_iter().map(|(text, val)| {
                let text = text.into_iter().map(|x| x.into()).collect();
                let val = val.into();
                (text, val)
            }))
            .into();
    }
}

impl<X> MappingData<X> {
    pub fn extended<E>(mut self, other: impl IntoIterator<Item = E>) -> Self
    where
        Self: Extend<E>,
    {
        self.extend(other);
        self
    }
    pub fn with_append(self, other: FlagData) -> Self {
        let m: RealMapping = self.as_ref().clone();
        Self {
            data: m.with_merged(other.as_ref().as_ref()).into(),
            _phantom: Default::default(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&MapKey, &RuntimeValue)> {
        self.as_ref().iter()
    }

    pub fn inefficient_insert(&mut self, key: impl Into<MapKey>, value: RuntimeValue) {
        self.extend(std::iter::once((key.into(), value)));
    }
}
impl<X> AsRef<RealMapping> for MappingData<X> {
    fn as_ref(&self) -> &RealMapping {
        if let RuntimeAny::Value(RuntimeValue::Comp(crate::spec::MappingOrArray::Mapping(
            mapping,
        ))) = &self.data
        {
            mapping
        } else {
            unreachable!("the type guarantees that the kept value is a real mapping")
        }
    }
}
impl<X> From<MappingData<X>> for RealMapping {
    fn from(value: MappingData<X>) -> Self {
        value.as_ref().clone()
    }
}

impl SelectableSource for ParamData {
    async fn request<'a>(
        &'a self,
        selector: &crate::spec::Selector,
        _allowed_features: Features,
    ) -> Result<Cow<'a, RuntimeAny>, FatalError> {
        match selector {
            Selector::Coremap(Coremapping::Param) => Ok(Cow::Borrowed(&self.data)),
            Selector::CoremapItem {
                mapping: Coremapping::Param,
                key,
            } => Ok(Cow::Owned(
                self.as_ref()
                    .get_by_seq(key)
                    .transpose()
                    .map(|x| x.cloned().into_runtimeany())
                    .unwrap_or(self.data.clone()),
            )),

            Selector::Coremap(_) | Selector::CoremapItem { .. } => {
                Self::mark_fallback_need(selector.clone())
            }
        }
    }
}

impl SelectableSource for FlagData {
    async fn request<'a>(
        &'a self,
        selector: &crate::spec::Selector,
        _allowed_features: Features,
    ) -> Result<Cow<'a, RuntimeAny>, FatalError> {
        match selector {
            Selector::Coremap(Coremapping::Flags) => Ok(Cow::Borrowed(&self.data)),
            Selector::CoremapItem {
                mapping: Coremapping::Flags,
                key,
            } => Ok(Cow::Owned(
                self.as_ref()
                    .get_by_seq(key)
                    .transpose()
                    .map(|x| x.cloned().into_runtimeany())
                    .unwrap_or(self.data.clone()),
            )),

            Selector::Coremap(_) | Selector::CoremapItem { .. } => {
                Self::mark_fallback_need(selector.clone())
            }
        }
    }
}

impl<'a> Extend<&'a RuntimeAction> for MappingData<SourceFlag> {
    fn extend<T: IntoIterator<Item = &'a RuntimeAction>>(&mut self, actions: T) {
        let actions = actions.into_iter().flat_map(|action: &'a RuntimeAction| {
            if let RuntimeAction::SetFlag(action) = action {
                let (mode, key, value) = action.clone().into_parts();
                match mode {
                    SetFlagMode::Keep => Some((key, value)),
                }
            } else {
                None
            }
        });
        self.extend(actions);
    }
}
