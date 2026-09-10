use std::{cmp::Ordering, collections::BTreeMap};

use derive_more::{From, Into};
use either::Either;
use serde::{Deserialize, Serialize};

use crate::catchable::cerr;

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

#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PrimitiveValue {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
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

// #####################################
// ENTITIES
// #####################################

#[derive(Debug, Serialize, Deserialize)]
#[serde(from = "u64", into = "u64")]
pub struct EntityId<T>(u64, std::marker::PhantomData<T>);

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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Entity {
    Criterion(CriterionEntity),
    Action(ActionEntity),
    Value(ValueEntity),
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case")]
pub enum CriterionEntity {
    #[serde(rename = "negated")]
    Negated {
        #[serde(rename = "fexp")]
        failure_explaination: Option<ValueReference>,
        #[serde(rename = "c")]
        clause: EntityId<CriterionEntity>,
    },
    #[serde(rename = "any_of")]
    AnyOf {
        #[serde(rename = "fexp")]
        failure_explaination: Option<ValueReference>,
        #[serde(rename = "a")]
        clauses: Vec<EntityId<CriterionEntity>>,
    },
    #[serde(rename = "all_of")]
    AllOf {
        #[serde(rename = "fexp")]
        failure_explaination: Option<ValueReference>,
        #[serde(rename = "a")]
        clauses: Vec<EntityId<CriterionEntity>>,
    },
    ContainOnlynum {
        #[serde(rename = "fexp")]
        failure_explaination: Option<ValueReference>,
        sub: ValueReference,
        sup: ValueReference,
    },
    ContainNum {
        #[serde(rename = "fexp")]
        failure_explaination: Option<ValueReference>,
        sub: ValueReference,
        sup: ValueReference,
    },
    ContainText {
        #[serde(rename = "fexp")]
        failure_explaination: Option<ValueReference>,
        sub: ValueReference,
        sup: ValueReference,
    },

    #[serde(rename = "contain-wgaps")]
    ContainWithGaps {
        #[serde(rename = "fexp")]
        failure_explaination: Option<ValueReference>,
        #[serde(rename = "a")]
        parts: Vec<ValueReference>,
        sup: ValueReference,
    },
    #[serde(rename = "regex")]
    MatchesRegex {
        #[serde(rename = "fexp")]
        failure_explaination: Option<ValueReference>,
        pattern: ValueReference,
        sup: ValueReference,
    },

    #[serde(rename = ">")]
    GreaterThen {
        #[serde(rename = "l")]
        left: ValueReference,
        #[serde(rename = "r")]
        right: ValueReference,
    },
    #[serde(rename = "<")]
    LowerThen {
        #[serde(rename = "l")]
        left: ValueReference,
        #[serde(rename = "r")]
        right: ValueReference,
    },
    #[serde(rename = ">=")]
    GreaterEqual {
        #[serde(rename = "l")]
        left: ValueReference,
        #[serde(rename = "r")]
        right: ValueReference,
    },
    #[serde(rename = "<=")]
    LowerEqual {
        #[serde(rename = "l")]
        left: ValueReference,
        #[serde(rename = "r")]
        right: ValueReference,
    },
    #[serde(rename = "!=")]
    NotEqual {
        #[serde(rename = "l")]
        left: ValueReference,
        #[serde(rename = "r")]
        right: ValueReference,
    },
    #[serde(rename = "==")]
    Equal {
        #[serde(rename = "l")]
        left: ValueReference,
        #[serde(rename = "r")]
        right: ValueReference,
    },
    Catch {
        #[serde(rename = "c")]
        criterion: EntityId<CriterionEntity>,
        #[serde(with = "bitflags::serde")]
        error: cerr,
        #[serde(rename = "only-if")]
        only_if: Option<EntityId<CriterionEntity>>,
        #[serde(rename = "default-value")]
        default_value: Option<EntityId<CriterionEntity>>,
        action: Option<EntityId<ActionEntity>>,
    },
    #[serde(rename = "cifte")]
    IfThenElse {
        #[serde(rename = "i")]
        if_: EntityId<CriterionEntity>,
        #[serde(rename = "t")]
        then_: EntityId<CriterionEntity>,
        #[serde(rename = "e")]
        else_: EntityId<CriterionEntity>,
    },
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ValueReference {
    LitString(String),
    EntityId(EntityId<ValueEntity>),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MessageSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EndThisTestMode {
    Pass,
    Fail,
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SetFlagMode {
    Keep,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case")]
pub enum ActionEntity {
    SendMsg {
        #[serde(rename = "t")]
        text: ValueReference,
        #[serde(rename = "s")]
        severity: MessageSeverity,
        #[serde(rename = "l")]
        level: Option<MessageSendingLevel>,
    },
    EndThisTest {
        #[serde(rename = "m")]
        mode: EndThisTestMode,
        #[serde(rename = "e")]
        explaination: String,
    },

    SetFlag {
        #[serde(rename = "m")]
        mode: SetFlagMode,
        #[serde(rename = "k")]
        key: ValueReference,
        #[serde(rename = "v")]
        value: ValueReference,
    },
    #[serde(rename = "aifte")]
    IfThenElse {
        #[serde(rename = "i")]
        if_: EntityId<CriterionEntity>,
        #[serde(rename = "t")]
        then_: EntityId<ActionEntity>,
        #[serde(rename = "e")]
        else_: EntityId<ActionEntity>,
    },
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MessageSendingLevel {
    Spec,
    Category,
    Maintest,
}

pub type MappingReference = ValueReference;

#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case")]
pub enum ValueEntity {
    Lit {
        #[serde(rename = "v")]
        value: PrimitiveValue,
    },
    Concat {
        a: Vec<ValueReference>,
    },
    ReadInput {},
    ReadOutput {},
    ReadLists {},
    ReadVariables {},
    ReadRandoms {},
    ReadBlockcount {},
    ReadParam {},

    #[serde(rename = "network")]
    NetworkRequest {
        server: String,
        route: ValueReference,
        // GET or POST
        method: String,
        #[serde(rename = "allowed-status")]
        allowed_status: Option<Vec<u16>>,
        json: Option<Vec<(ValueReference, ValueReference)>>,
    },

    View {
        #[serde(rename = "p")]
        perspective: String,
        #[serde(rename = "m")]
        mapping: MappingReference,
    },

    Mapitem {
        #[serde(rename = "m")]
        mapping: MappingReference,
        #[serde(rename = "k")]
        keys: Vec<ValueReference>,
    },
    Add {
        #[serde(rename = "l")]
        left: ValueReference,
        #[serde(rename = "r")]
        right: ValueReference,
    },
    Sub {
        #[serde(rename = "l")]
        left: ValueReference,
        #[serde(rename = "r")]
        right: ValueReference,
    },
    Mul {
        #[serde(rename = "l")]
        left: ValueReference,
        #[serde(rename = "r")]
        right: ValueReference,
    },
    Truediv {
        #[serde(rename = "l")]
        left: ValueReference,
        #[serde(rename = "r")]
        right: ValueReference,
    },
    Floordiv {
        #[serde(rename = "l")]
        left: ValueReference,
        #[serde(rename = "r")]
        right: ValueReference,
    },
    Mod {
        #[serde(rename = "l")]
        left: ValueReference,
        #[serde(rename = "r")]
        right: ValueReference,
    },
    Pow {
        #[serde(rename = "l")]
        left: ValueReference,
        #[serde(rename = "r")]
        right: ValueReference,
    },
    Neg {
        #[serde(rename = "v")]
        value: ValueReference,
    },
    Floor {
        #[serde(rename = "v")]
        value: ValueReference,
    },
    Ceil {
        #[serde(rename = "v")]
        value: ValueReference,
    },
    Round {
        #[serde(rename = "v")]
        value: ValueReference,
    },
    Abs {
        #[serde(rename = "v")]
        value: ValueReference,
    },
    #[serde(rename = "first-capture")]
    FirstCaptureOfRegex {
        #[serde(rename = "fexp")]
        failure_explaination: Option<ValueReference>,
        #[serde(rename = "p")]
        pattern: ValueReference,
        sup: ValueReference,
    },
    Catch {
        #[serde(rename = "v")]
        value: ValueReference,
        #[serde(with = "bitflags::serde")]
        error: cerr,
        #[serde(rename = "only-if")]
        only_if: Option<EntityId<CriterionEntity>>,
        #[serde(rename = "default-value")]
        default_value: Option<ValueReference>,
        action: Option<EntityId<ActionEntity>>,
    },
    ToUpper {
        #[serde(rename = "v")]
        value: ValueReference,
    },
    ToLower {
        #[serde(rename = "v")]
        value: ValueReference,
    },
    TrimStart {
        #[serde(rename = "v")]
        value: ValueReference,
    },
    TrimEnd {
        #[serde(rename = "v")]
        value: ValueReference,
    },
    Trim {
        #[serde(rename = "v")]
        value: ValueReference,
    },
    #[serde(rename = "vifte")]
    IfThenElse {
        #[serde(rename = "i")]
        if_: EntityId<CriterionEntity>,
        #[serde(rename = "t")]
        then_: ValueReference,
        #[serde(rename = "e")]
        else_: ValueReference,
    },
    Length {
        // can also be mapping/array
        #[serde(rename = "v")]
        value: ValueReference,
    },
}

// #####################################
// HOOKS
// #####################################

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
