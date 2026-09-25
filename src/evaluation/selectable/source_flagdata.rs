use std::{
    borrow::Cow,
    collections::{BTreeMap, VecDeque},
};

use crate::{
    evaluation::Effects,
    spec::{
        Coremapping, FatalError, IntoRuntimeAny, MapKey, RealMapping, RuntimeAction, RuntimeAny,
        RuntimeValue, Selector, SetFlagMode, Text,
    },
};

use super::{Features, SelectableSource};

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct FlagData {
    flags: RuntimeAny, // RealMapping
}

impl Default for FlagData {
    fn default() -> Self {
        Self {
            flags: RuntimeAny::Value(RealMapping::default().into()),
        }
    }
}

impl From<BTreeMap<MapKey, RuntimeValue>> for FlagData {
    fn from(value: BTreeMap<MapKey, RuntimeValue>) -> Self {
        Self {
            flags: RuntimeAny::Value(RealMapping::from(value).into()),
        }
    }
}

impl From<BTreeMap<Text, RuntimeValue>> for FlagData {
    fn from(value: BTreeMap<Text, RuntimeValue>) -> Self {
        Self::default().extended(value)
    }
}

impl<K: Into<MapKey>, V: Into<RuntimeValue>> Extend<(K, V)> for FlagData {
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        let flags: RealMapping = self.as_ref().clone();
        self.flags = flags
            .with_new_keys(
                iter.into_iter()
                    .map(|(text, val)| (vec![text.into()].into_iter().collect(), val.into())),
            )
            .into();
    }
}

impl<K: Into<MapKey>, V: Into<RuntimeValue>> Extend<(VecDeque<K>, V)> for FlagData {
    fn extend<T: IntoIterator<Item = (VecDeque<K>, V)>>(&mut self, iter: T) {
        let flags: RealMapping = self.as_ref().clone();
        self.flags = flags
            .with_new_keys(
                iter.into_iter()
                    .map(|(text, val)| (text.into_iter().map(|x| x.into()).collect(), val.into())),
            )
            .into();
    }
}

impl<K: Into<MapKey>, V: Into<RuntimeValue>> Extend<(Vec<K>, V)> for FlagData {
    fn extend<T: IntoIterator<Item = (Vec<K>, V)>>(&mut self, iter: T) {
        let flags: RealMapping = self.as_ref().clone();
        self.flags = flags
            .with_new_keys(
                iter.into_iter()
                    .map(|(text, val)| (text.into_iter().map(|x| x.into()).collect(), val.into())),
            )
            .into();
    }
}

impl FlagData {
    pub fn extended<E>(mut self, other: impl IntoIterator<Item = E>) -> Self
    where
        Self: Extend<E>,
    {
        self.extend(other);
        self
    }
    pub fn with_append(self, other: FlagData) -> Self {
        let m: RealMapping = self.as_ref().clone();
        m.with_merged(other.as_ref().as_ref());
        self
    }

    pub fn iter(&self) -> impl Iterator<Item = (&MapKey, &RuntimeValue)> {
        self.as_ref().iter()
    }

    pub fn inefficient_insert(&mut self, key: impl Into<MapKey>, value: RuntimeValue) {
        self.extend(std::iter::once((key.into(), value)));
    }
}
impl AsRef<RealMapping> for FlagData {
    fn as_ref(&self) -> &RealMapping {
        if let RuntimeAny::Value(RuntimeValue::Comp(crate::spec::MappingOrArray::Mapping(
            mapping,
        ))) = &self.flags
        {
            mapping
        } else {
            unreachable!("the type guarantees that the kept value is a real mapping")
        }
    }
}
impl From<FlagData> for RealMapping {
    fn from(value: FlagData) -> Self {
        value.as_ref().clone()
    }
}

impl SelectableSource for FlagData {
    async fn request<'a>(
        &'a self,
        selector: &crate::spec::Selector,
        _allowed_features: Features,
    ) -> Result<Cow<'a, RuntimeAny>, FatalError> {
        match selector {
            Selector::Coremap(Coremapping::Flags) => Ok(Cow::Borrowed(&self.flags)),
            Selector::CoremapItem {
                mapping: Coremapping::Flags,
                key,
            } => Ok(Cow::Owned(
                self.as_ref()
                    .get_by_seq(key)
                    .transpose()
                    .map(|x| x.cloned().into_runtimeany())
                    .unwrap_or(self.flags.clone()),
            )),

            Selector::Coremap(_) | Selector::CoremapItem { .. } => {
                Self::mark_fallback_need(selector.clone())
            }
        }
    }
}

impl<'a> Extend<&'a RuntimeAction> for FlagData {
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

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct WithEffects<T> {
    data: T,
    effects: Effects,
}
impl<T> WithEffects<T> {
    pub fn new(data: T, effects: Effects) -> Self {
        Self { data, effects }
    }
    pub fn with_flags(self, effects: Effects) -> Self {
        Self {
            data: self.data,
            effects,
        }
    }
    pub fn into_parts(self) -> (T, Effects) {
        (self.data, self.effects)
    }
}
