use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::spec::{
    AlternativeTestHooks, CriterionEntity, Entity, EntityId, MainTestHooks, PrimitiveValue,
    hooks::SpecHooks,
};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Bast3StSpec {
    title: String,
    description: Option<String>,
    #[serde(default)]
    categories: Vec<Category>,
    #[serde(default)]
    hooks: SpecHooks,
    #[serde(rename = "nodes")]
    entities: BTreeMap<u64, Entity>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Category {
    title: String,
    description: Option<String>,
    #[serde(default)]
    tests: Vec<MainTest>,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct MainTest {
    title: String,
    criterion: EntityId<CriterionEntity>,
    input: Option<Vec<PrimitiveValue>>,
    random_generation: Option<either::Either<u64, bool>>,
    predefined_randoms: Option<Vec<either::Either<u64, f64>>>,
    initial_variables: Option<BTreeMap<String, PrimitiveValue>>,
    initial_lists: Option<BTreeMap<String, Vec<PrimitiveValue>>>,
    #[serde(rename = "tests", default)]
    alternative_tests: Vec<AlternativeTest>,
    #[serde(default)]
    hooks: MainTestHooks,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AlternativeTest {
    title: String,
    criterion: EntityId<CriterionEntity>,
    input: Option<Vec<PrimitiveValue>>,
    random_generation: Option<either::Either<u64, bool>>,
    predefined_randoms: Option<Vec<either::Either<u64, f64>>>,
    initial_variables: Option<BTreeMap<String, PrimitiveValue>>,
    initial_lists: Option<BTreeMap<String, Vec<PrimitiveValue>>>,
    #[serde(default)]
    hooks: AlternativeTestHooks,
}
