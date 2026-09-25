// #####################################
// HOOKS
// #####################################

use derive_getters::Getters;
use derive_more::Deref;
use serde::{Deserialize, Serialize};

use crate::spec::{ActionEntity, CriterionEntity, EntityId};

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default, Clone, Getters,
)]
pub struct SpecHooks<HL = HookList> {
    #[serde(rename = "before-all-categories", default)]
    pub(crate) before_all_categories: HL,
    #[serde(rename = "after-all-categories", default)]
    pub(crate) after_all_categories: HL,
}

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default, Clone, Getters,
)]
pub struct CategoryHooks<HL = HookList> {
    #[serde(rename = "before-all-tests", default)]
    pub(crate) before_all_tests: HL,
    #[serde(rename = "after-all-tests", default)]
    pub(crate) after_all_tests: HL,
}

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default, Getters, Clone,
)]
pub struct MainTestHooks<HL = HookList> {
    #[serde(rename = "before-main", default)]
    pub(crate) before_main: HL,
    #[serde(rename = "before-alternatives", default)]
    pub(crate) before_alternatives: HL,
    #[serde(rename = "after-alternatives", default)]
    pub(crate) after_alternatives: HL,
}

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default, Getters, Clone,
)]
pub struct AlternativeTestHooks<HL = HookList> {
    #[serde(rename = "before-alt", default)]
    pub(crate) before_alt: HL,
    #[serde(rename = "after-alt", default)]
    pub(crate) after_alt: HL,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default, Deref, Clone)]
pub struct HookList(Vec<(EntityId<CriterionEntity>, EntityId<ActionEntity>)>);

impl<'a> IntoIterator for &'a HookList {
    type Item = (EntityId<CriterionEntity>, EntityId<ActionEntity>);
    type IntoIter = std::iter::Cloned<std::slice::Iter<'a, Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().cloned()
    }
}
impl HookList {
    pub fn iter(
        &self,
    ) -> impl ExactSizeIterator<Item = (EntityId<CriterionEntity>, EntityId<ActionEntity>)>
    + DoubleEndedIterator<Item = (EntityId<CriterionEntity>, EntityId<ActionEntity>)> {
        self.into_iter()
    }
}
