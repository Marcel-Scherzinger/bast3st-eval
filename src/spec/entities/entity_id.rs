use derive_more::{Debug, Deref};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Deref)]
#[serde(from = "u64", into = "u64")]
#[debug("EntityId<{}>({_0})", std::any::type_name::<T>().split("::").last().unwrap_or_default())]
pub struct EntityId<T>(#[deref] pub(super) u64, std::marker::PhantomData<T>);

impl<T> Clone for EntityId<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for EntityId<T> {}
impl<T> Eq for EntityId<T> {}
impl<T> PartialEq for EntityId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<T> Ord for EntityId<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
impl<T> PartialOrd for EntityId<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> From<u64> for EntityId<T> {
    fn from(value: u64) -> Self {
        Self(value, Default::default())
    }
}

impl<T> From<EntityId<T>> for u64 {
    fn from(value: EntityId<T>) -> Self {
        value.0
    }
}

impl<T> EntityId<T> {
    pub(super) fn _cast_id<U>(self) -> EntityId<U> {
        EntityId(self.0, Default::default())
    }
    pub fn cast_id<U>(self) -> EntityId<U>
    where
        U: From<T>,
    {
        EntityId(self.0, Default::default())
    }
}
