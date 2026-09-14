use std::{borrow::Cow, collections::BTreeMap, fmt::Debug};

use serde::{Deserialize, Serialize};

use crate::{
    catchable::cerr,
    spec::{
        ActionEntity, Entity, EntityId, MachineConstruction, ValueReference,
        machine::Machine,
        runtime::{ClarifiedCerrMerging, PossibleRuntimeValue, RuntimeCriterion, SpecializeFrom},
    },
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

pub(super) fn eval_if_then_else<E, R: Clone + SpecializeFrom + ClarifiedCerrMerging + 'static>(
    if_: &EntityId<CriterionEntity>,
    then_: E,
    else_: E,
) -> Machine<R>
where
    E: PossibleRuntimeValue<R> + MachineConstruction<E, R> + 'static,
{
    if_.query_ref(move |if_: &RuntimeCriterion| {
        if if_.0 {
            then_.query(|then_| Machine::from_final(then_))
        } else {
            else_.query(|else_| Machine::from_final(else_))
        }
    })
}
fn eval_any_of(
    mut reversed_conditions: Vec<EntityId<CriterionEntity>>,
) -> Machine<RuntimeCriterion> {
    if let Some(x) = reversed_conditions.pop() {
        x.query_ref(|crit: &RuntimeCriterion| {
            if crit.0 {
                crit.clone().into()
            } else {
                eval_any_of(reversed_conditions)
            }
        })
    } else {
        RuntimeCriterion(false).into()
    }
}
