use std::{collections::BTreeMap, fmt::Debug};

use serde::{Deserialize, Serialize};

use crate::{
    catchable::cerr,
    spec::{ActionEntity, Entity, EntityId, ValueReference},
};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case")]
pub enum CriterionEntity {
    Negated {
        #[serde(rename = "fexp")]
        failure_explaination: Option<ValueReference>,
        #[serde(rename = "c")]
        clause: EntityId<CriterionEntity>,
    },
    AnyOf {
        #[serde(rename = "fexp")]
        failure_explaination: Option<ValueReference>,
        #[serde(rename = "a")]
        clauses: Vec<EntityId<CriterionEntity>>,
    },
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
