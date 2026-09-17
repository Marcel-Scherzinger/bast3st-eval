use std::{borrow::Cow, collections::BTreeMap, sync::Arc};

use bitflags::iter::IterNames;
use derive_more::From;
use either::Either;

use crate::{
    catchable::cerr,
    spec::{MapKey, PrimitiveValue, RuntimeValue},
};

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, From)]
pub struct Array(Arc<[RuntimeValue]>);

#[derive(derive_more::Debug, Clone, PartialEq, PartialOrd, From)]
pub enum MappingOrArray {
    #[debug("{_0:?}")]
    Array(Array),
    #[debug("{_0:?}")]
    Mapping(RealMapping),
}
impl MappingOrArray {
    pub fn len(&self) -> usize {
        match self {
            Self::Array(a) => a.len(),
            Self::Mapping(a) => a.len(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn keys<'a>(&'a self) -> impl Iterator<Item = Cow<'a, MapKey>> {
        match self {
            Self::Array(a) => Either::Left(a.keys().map(Cow::Owned)),
            Self::Mapping(m) => Either::Right(m.keys().map(Cow::Borrowed)),
        }
    }
    pub fn value_array(&self) -> Array {
        match self {
            Self::Array(a) => a.clone(),
            Self::Mapping(m) => m.values().cloned().collect(),
        }
    }
    pub fn get(&self, key: &MapKey) -> Result<&RuntimeValue, cerr> {
        match (self, key) {
            (Self::Array(array), MapKey::Int(key)) => array.get(*key),
            (Self::Mapping(mapping), key) => mapping.get(key),
            (Self::Array(_), MapKey::Str(_)) => Err(cerr::collection_keyInvalidForArray),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, From)]
pub struct RealMapping {
    #[from]
    mapping: Arc<BTreeMap<MapKey, RuntimeValue>>,
    #[from(skip)]
    default: Option<Box<RuntimeValue>>,
}

impl RealMapping {
    pub fn new_with_default(
        mapping: Arc<BTreeMap<MapKey, RuntimeValue>>,
        default: Option<Box<RuntimeValue>>,
    ) -> Self {
        Self { mapping, default }
    }
    pub fn keys(&self) -> std::collections::btree_map::Keys<'_, MapKey, RuntimeValue> {
        self.mapping.keys()
    }
    pub fn values(&self) -> std::collections::btree_map::Values<'_, MapKey, RuntimeValue> {
        self.mapping.values()
    }
    pub fn len(&self) -> usize {
        self.mapping.len()
    }
    pub fn get(&self, key: &MapKey) -> Result<&RuntimeValue, cerr> {
        self.mapping
            .get(key)
            .or(self.default.as_deref())
            .ok_or(cerr::collection_valueNotFound)
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
impl Array {
    pub fn new(array: Arc<[RuntimeValue]>) -> Self {
        Self(array)
    }
    pub fn keys(&self) -> impl Iterator<Item = MapKey> {
        std::iter::successors(Some(0), |x| Some(x + 1))
            .map(MapKey::Int)
            .take(self.len())
    }
    pub fn value_array(&self) -> Array {
        self.clone()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get(&self, mut key: i64) -> Result<&RuntimeValue, cerr> {
        let len: Result<i64, _> = self.len().try_into();
        if let Ok(len) = len
            && key < 0
        {
            key += len;
        }

        let key: usize = key
            .try_into()
            .map_err(|_| cerr::collection_keyInvalidForArray)?;
        self.0.get(key).ok_or(cerr::collection_valueNotFound)
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn iter(&self) -> impl Iterator<Item = &RuntimeValue> {
        self.0.iter()
    }
}

impl<P: Into<RuntimeValue>> FromIterator<P> for Array {
    fn from_iter<T: IntoIterator<Item = P>>(iter: T) -> Self {
        Self(iter.into_iter().map(|x| x.into()).collect())
    }
}
impl From<BTreeMap<MapKey, RuntimeValue>> for RealMapping {
    fn from(value: BTreeMap<MapKey, RuntimeValue>) -> Self {
        Self {
            mapping: value.into(),
            ..Default::default()
        }
    }
}
impl From<Vec<RuntimeValue>> for Array {
    fn from(value: Vec<RuntimeValue>) -> Self {
        Self(value.into())
    }
}
