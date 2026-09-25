use std::collections::BTreeMap;

use derive_getters::Getters;
use serde::{Deserialize, Serialize};

use crate::spec::{
    AlternativeTestHooks, CategoryHooks, CriterionEntity, Entity, EntityId, MainTestHooks,
    PrimitiveValue, RandomGeneration, hooks::SpecHooks,
};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Getters)]
pub struct Bast3StSpec {
    title: String,
    description: Option<String>,
    #[serde(default)]
    categories: Vec<Category>,
    #[serde(default)]
    hooks: SpecHooks,
    #[serde(rename = "nodes")]
    entities: BTreeMap<EntityId, Entity>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Getters, Clone)]
pub struct Category {
    title: String,
    description: Option<String>,
    #[serde(default)]
    tests: Vec<MainTest>,
    #[serde(default)]
    hooks: CategoryHooks,
}
#[derive(Debug, PartialEq, Serialize, Deserialize, Getters, Clone)]
pub struct MainTest {
    #[serde(flatten)]
    general: GeneralTest<MainTestHooks>,
    #[serde(rename = "tests", default)]
    alternative_tests: Vec<AlternativeTest>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Getters, Clone)]
pub struct AlternativeTest {
    #[serde(flatten)]
    general: GeneralTest<AlternativeTestHooks>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Getters, Clone)]
pub struct GeneralTest<Hooks> {
    title: String,
    criterion: EntityId<CriterionEntity>,
    input: Option<Vec<PrimitiveValue>>,
    #[serde(default)]
    random_generation: RandomGeneration,
    predefined_randoms: Option<Vec<scratch_test_value::SNumber>>,
    initial_variables: Option<BTreeMap<String, PrimitiveValue>>,
    initial_lists: Option<BTreeMap<String, Vec<PrimitiveValue>>>,
    #[serde(default)]
    hooks: Hooks,
}

impl AsRef<GeneralTest<MainTestHooks>> for MainTest {
    fn as_ref(&self) -> &GeneralTest<MainTestHooks> {
        &self.general
    }
}
impl AsRef<GeneralTest<AlternativeTestHooks>> for AlternativeTest {
    fn as_ref(&self) -> &GeneralTest<AlternativeTestHooks> {
        &self.general
    }
}
