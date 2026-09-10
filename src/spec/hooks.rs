// #####################################
// HOOKS
// #####################################

use serde::{Deserialize, Serialize};

use crate::spec::{ActionEntity, CriterionEntity, EntityId};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct SpecHooks {
    #[serde(rename = "before-all-categories", default)]
    before_all_categories: HookList,
    #[serde(rename = "after-all-categories", default)]
    after_all_categories: HookList,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct CategoryHooks {
    #[serde(rename = "before-all-tests", default)]
    before_all_tests: HookList,
    #[serde(rename = "after-all-tests", default)]
    after_all_tests: HookList,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct MainTestHooks {
    #[serde(rename = "before-main", default)]
    before_main: HookList,
    #[serde(rename = "before-alternatives", default)]
    before_alternatives: HookList,
    #[serde(rename = "after-alternatives", default)]
    after_alternatives: HookList,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct AlternativeTestHooks {
    #[serde(rename = "before-alt", default)]
    before_alt: HookList,
    #[serde(rename = "after-alt", default)]
    after_alt: HookList,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct HookList(Vec<(EntityId<CriterionEntity>, EntityId<ActionEntity>)>);
