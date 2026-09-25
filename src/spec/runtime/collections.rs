use std::{
    borrow::Cow,
    collections::{BTreeMap, VecDeque},
    iter::Sum,
    sync::Arc,
};

use derive_more::From;
use either::Either;
use scratch_test_value::{SList, SNumber};
use serde::{Deserialize, Serialize};

use crate::{
    catchable::cerr,
    spec::{MapKey, Numeric, PrimitiveValue, RuntimeValue},
};

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, From, Serialize, Deserialize)]
pub struct Array(Arc<[RuntimeValue]>);

#[derive(derive_more::Debug, Clone, PartialEq, PartialOrd, From, Serialize, Deserialize)]
#[serde(untagged)]
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
    pub fn sum(&self) -> Numeric {
        match self {
            Self::Array(a) => a.sum(),
            Self::Mapping(m) => m.values().sum(),
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
    pub fn to_btreemap(&self) -> BTreeMap<MapKey, RuntimeValue> {
        match self {
            Self::Mapping(x) => x.mapping.as_ref().clone(),
            Self::Array(x) => x.clone().into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, From, Serialize, Deserialize)]
#[serde(
    from = "Arc<BTreeMap<MapKey, RuntimeValue>>",
    into = "Arc<BTreeMap<MapKey, RuntimeValue>>"
)]
pub struct RealMapping {
    #[from]
    mapping: Arc<BTreeMap<MapKey, RuntimeValue>>,
    #[from(skip)]
    #[serde(skip)]
    default: Option<Box<RuntimeValue>>,
}

impl From<Arc<BTreeMap<MapKey, RuntimeValue>>> for RealMapping {
    fn from(value: Arc<BTreeMap<MapKey, RuntimeValue>>) -> Self {
        Self {
            mapping: value,
            default: None,
        }
    }
}

impl From<RealMapping> for Arc<BTreeMap<MapKey, RuntimeValue>> {
    fn from(value: RealMapping) -> Self {
        value.mapping
    }
}

impl AsRef<BTreeMap<MapKey, RuntimeValue>> for RealMapping {
    fn as_ref(&self) -> &BTreeMap<MapKey, RuntimeValue> {
        &self.mapping
    }
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
    /// Ok(None) means self (empty key)
    pub fn get_by_seq<'a, 'k>(
        &'a self,
        key: impl IntoIterator<Item = &'k MapKey>,
    ) -> Result<Option<&'a RuntimeValue>, cerr> {
        let mut key = key.into_iter();
        if let Some(first) = key.next() {
            let mut curr = self.get(first)?;
            for k in key {
                if let RuntimeValue::Comp(coll) = curr {
                    curr = coll.get(k)?;
                } else {
                    return Err(cerr::typing_notCollection);
                }
            }
            Ok(Some(curr))
        } else {
            Ok(None)
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn iter(&self) -> impl Iterator<Item = (&MapKey, &RuntimeValue)> {
        self.mapping.iter()
    }

    pub fn with_merged(&self, other: &BTreeMap<MapKey, RuntimeValue>) -> Self {
        let merged = self.with_new_keys(
            other
                .iter()
                .map(|(key, val)| (vec![key.clone()].into(), val.clone())),
        );
        log::trace!("merge of {self:?} with {other:?} resulted in {merged:?}");
        merged
    }

    pub fn with_new_keys<K: Into<MapKey>, V: Into<RuntimeValue>>(
        &self,
        other: impl IntoIterator<Item = (VecDeque<K>, V)>,
    ) -> RealMapping {
        let mut extendable = DeepExtendable::from(self.mapping.as_ref().clone());
        extendable.ensure_inner();

        for (keys, value) in other {
            let value = value.into();
            extendable.insert_path(keys.into_iter().map(|k| k.into()).collect(), value.into());
        }

        let extendable = if let RuntimeValue::Comp(MappingOrArray::Mapping(m)) = extendable.into() {
            m
        } else {
            Default::default()
        };

        RealMapping {
            mapping: extendable.mapping,
            default: self.default.clone(),
        }
    }
}

// TODO: vulnerable to stack overflow
impl From<RuntimeValue> for DeepExtendable {
    fn from(value: RuntimeValue) -> Self {
        match value {
            RuntimeValue::Prim(x) => Self::Leaf(x),
            RuntimeValue::Comp(c) => Self::from(c.to_btreemap()),
        }
    }
}

// TODO: vulnerable to stack overflow
impl From<DeepExtendable> for RuntimeValue {
    fn from(value: DeepExtendable) -> Self {
        match value {
            DeepExtendable::Leaf(l) => Self::Prim(l),
            DeepExtendable::Inner(i) => RealMapping::from(
                i.into_iter()
                    .map(|(k, v)| (k, RuntimeValue::from(v)))
                    .collect::<BTreeMap<_, _>>(),
            )
            .into(),
        }
    }
}

impl FromIterator<(MapKey, RuntimeValue)> for RealMapping {
    fn from_iter<T: IntoIterator<Item = (MapKey, RuntimeValue)>>(iter: T) -> Self {
        Self {
            mapping: BTreeMap::from_iter(iter).into(),
            default: None,
        }
    }
}

// TODO: vulnerable to stack overflow
impl From<BTreeMap<MapKey, RuntimeValue>> for DeepExtendable {
    fn from(value: BTreeMap<MapKey, RuntimeValue>) -> Self {
        Self::Inner(
            value
                .into_iter()
                .map(|(key, val)| {
                    (
                        key,
                        match val {
                            RuntimeValue::Prim(x) => DeepExtendable::Leaf(x),
                            RuntimeValue::Comp(x) => DeepExtendable::from(x.to_btreemap()),
                        },
                    )
                })
                .collect(),
        )
    }
}

#[derive(Debug, From, Clone)]
enum DeepExtendable {
    Inner(BTreeMap<MapKey, DeepExtendable>),
    Leaf(PrimitiveValue),
}
impl DeepExtendable {
    // TODO: vulnerable to stack overflow
    fn insert_path(&mut self, mut keys: VecDeque<MapKey>, value: DeepExtendable) {
        if let Some(first_key) = keys.pop_front() {
            let inner = self.ensure_inner().entry(first_key).or_default();
            if keys.is_empty() {
                inner.merge_with(value);
            } else {
                inner.insert_path(keys, value);
            }
        }
    }
    // TODO: vulnerable to stack overflow
    fn merge_with(&mut self, value: DeepExtendable) {
        match value {
            DeepExtendable::Leaf(leaf) => *self = DeepExtendable::Leaf(leaf),
            DeepExtendable::Inner(map) => {
                for (key, val) in map.into_iter() {
                    self.insert_path(vec![key].into(), val);
                }
            }
        }
    }

    #[allow(unused)]
    fn insert_single(&mut self, key: MapKey, value: PrimitiveValue) {
        self.ensure_inner().insert(key, value.into());
    }
    fn ensure_inner(&mut self) -> &mut BTreeMap<MapKey, DeepExtendable> {
        match self {
            Self::Inner(_) => (),
            Self::Leaf(l) => {
                let l = std::mem::replace(l, PrimitiveValue::Number(SNumber::Int(0)));
                *self = Self::Inner(
                    vec![(MapKey::Str("".into()), DeepExtendable::Leaf(l))]
                        .into_iter()
                        .collect(),
                )
            }
        }
        if let Self::Inner(x) = self {
            x
        } else {
            unreachable!()
        }
    }
}

impl Default for DeepExtendable {
    fn default() -> Self {
        Self::Inner(Default::default())
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
    pub fn sum(&self) -> Numeric {
        self.iter().sum()
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

impl<'a> Sum<&'a RuntimeValue> for Numeric {
    fn sum<I: Iterator<Item = &'a RuntimeValue>>(iter: I) -> Self {
        let mut out = Numeric::Int(0);
        for v in iter {
            match v {
                RuntimeValue::Prim(PrimitiveValue::Number(v)) => {
                    out = out.q_add_numbers(v, &mut ())
                }
                RuntimeValue::Prim(PrimitiveValue::Str(t)) => {
                    if let Ok(t) = t.parse() {
                        out = out.q_add_numbers(&Numeric::Int(t), &mut ())
                    } else if let Ok(t) = t.parse() {
                        out = out.q_add_numbers(&Numeric::Float(t), &mut ())
                    }
                }
                RuntimeValue::Comp(_) => (),
            }
        }
        out
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

impl From<SList> for Array {
    fn from(value: SList) -> Self {
        value.into_iter().map(RuntimeValue::from).collect()
    }
}

impl From<Array> for BTreeMap<MapKey, RuntimeValue> {
    fn from(value: Array) -> Self {
        value
            .keys()
            .zip(value.iter().cloned())
            .collect::<BTreeMap<_, _>>()
    }
}
