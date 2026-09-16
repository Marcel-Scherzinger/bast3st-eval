use std::{borrow::Cow, collections::BTreeMap, fmt::Debug};

use scratch_test_value::SNumber;
use serde::{Deserialize, Serialize};

use crate::{
    catchable::cerr,
    spec::{
        ActionEntity, Entity, EntityId, MachineConstruction, MachineConstructionN, Numeric,
        PrimitiveValue, RuntimeAction, Text, ValueReference,
        machine::Machine,
        runtime::{
            ClarifiedCerrMerging, CompiledRegex, InnerRuntimeCriterion, MaybeEval,
            PossibleRuntimeValue, RuntimeAny, RuntimeCriterion, RuntimeValue, SpecializeFrom,
        },
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
        #[serde(rename = "fexp")]
        failure_explaination: Option<ValueReference>,
        #[serde(rename = "l")]
        left: ValueReference,
        #[serde(rename = "r")]
        right: ValueReference,
    },
    #[serde(rename = "<")]
    LowerThen {
        #[serde(rename = "fexp")]
        failure_explaination: Option<ValueReference>,
        #[serde(rename = "l")]
        left: ValueReference,
        #[serde(rename = "r")]
        right: ValueReference,
    },
    #[serde(rename = ">=")]
    GreaterEqual {
        #[serde(rename = "fexp")]
        failure_explaination: Option<ValueReference>,
        #[serde(rename = "l")]
        left: ValueReference,
        #[serde(rename = "r")]
        right: ValueReference,
    },
    #[serde(rename = "<=")]
    LowerEqual {
        #[serde(rename = "fexp")]
        failure_explaination: Option<ValueReference>,
        #[serde(rename = "l")]
        left: ValueReference,
        #[serde(rename = "r")]
        right: ValueReference,
    },
    #[serde(rename = "!=")]
    NotEqual {
        #[serde(rename = "fexp")]
        failure_explaination: Option<ValueReference>,
        #[serde(rename = "l")]
        left: ValueReference,
        #[serde(rename = "r")]
        right: ValueReference,
    },
    #[serde(rename = "==")]
    Equal {
        #[serde(rename = "fexp")]
        failure_explaination: Option<ValueReference>,
        #[serde(rename = "l")]
        left: ValueReference,
        #[serde(rename = "r")]
        right: ValueReference,
    },
    Catch {
        #[serde(rename = "fexp")]
        failure_explaination: Option<ValueReference>,
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
        #[serde(rename = "fexp")]
        failure_explaination: Option<ValueReference>,
        #[serde(rename = "i")]
        if_: EntityId<CriterionEntity>,
        #[serde(rename = "t")]
        then_: EntityId<CriterionEntity>,
        #[serde(rename = "e")]
        else_: EntityId<CriterionEntity>,
    },
}

impl CriterionEntity {
    pub fn machine(&self) -> Machine<RuntimeCriterion> {
        match self {
            Self::AnyOf {
                failure_explaination,
                clauses,
            } => eval_any_of(
                clauses.iter().rev().cloned().collect(),
                vec![],
                failure_explaination.clone(),
            ),
            Self::AllOf {
                failure_explaination,
                clauses,
            } => eval_all_of(
                clauses.iter().rev().cloned().collect(),
                vec![],
                failure_explaination.clone(),
            ),
            Self::IfThenElse {
                failure_explaination,
                if_,
                then_,
                else_,
            } => eval_if_then_else(if_, *then_, *else_, failure_explaination.clone()),
            Self::Negated {
                failure_explaination,
                clause,
            } => eval_negated(*clause, failure_explaination.clone()),

            Self::Equal {
                failure_explaination,
                left,
                right,
            } => eval_comparison(left, right, failure_explaination.clone(), |l, r| {
                (l == r, InnerRuntimeCriterion::Equal { left: l, right: r })
            }),
            Self::NotEqual {
                failure_explaination,
                left,
                right,
            } => eval_comparison(left, right, failure_explaination.clone(), |l, r| {
                (
                    l != r,
                    InnerRuntimeCriterion::NotEqual { left: l, right: r },
                )
            }),
            Self::LowerThen {
                failure_explaination,
                left,
                right,
            } => eval_comparison(left, right, failure_explaination.clone(), |l, r| {
                (
                    l < r,
                    InnerRuntimeCriterion::LowerThen { left: l, right: r },
                )
            }),
            Self::LowerEqual {
                failure_explaination,
                left,
                right,
            } => eval_comparison(left, right, failure_explaination.clone(), |l, r| {
                (
                    l <= r,
                    InnerRuntimeCriterion::LowerEqual { left: l, right: r },
                )
            }),
            Self::GreaterThen {
                failure_explaination,
                left,
                right,
            } => eval_comparison(left, right, failure_explaination.clone(), |l, r| {
                (
                    l > r,
                    InnerRuntimeCriterion::GreaterThen { left: l, right: r },
                )
            }),
            Self::GreaterEqual {
                failure_explaination,
                left,
                right,
            } => eval_comparison(left, right, failure_explaination.clone(), |l, r| {
                (
                    l >= r,
                    InnerRuntimeCriterion::GreaterEqual { left: l, right: r },
                )
            }),
            Self::ContainText {
                failure_explaination,
                sub,
                sup,
            } => eval_containtext(sub, sup, failure_explaination.clone()),
            Self::ContainWithGaps {
                failure_explaination,
                parts,
                sup,
            } => {
                let rev_parts = parts.iter().rev().cloned().collect();
                let failure_explaination = failure_explaination.clone();
                sup.query(move |sup| {
                    eval_contain_with_gaps(rev_parts, vec![], sup, 0, failure_explaination)
                })
            }
            Self::ContainNum {
                failure_explaination,
                sub,
                sup,
            } => {
                let mut failure_explaination = failure_explaination.clone();
                (sub, sup).query_n(move |sub: Numeric, sup: Text| {
                    let found: Vec<_> = find_numbers(sup.as_ref()).collect();
                    let contains = found.contains(&sub);
                    if contains {
                        failure_explaination = None;
                    }
                    failure_explaination.query(move |failure_explaination: Option<Text>| {
                        InnerRuntimeCriterion::ContainNum { sub, sup, found }
                            .build(contains, failure_explaination)
                            .into()
                    })
                })
            }
            Self::ContainOnlynum {
                failure_explaination,
                sub,
                sup,
            } => {
                let mut failure_explaination = failure_explaination.clone();
                (sub, sup).query_n(move |sub: Numeric, sup: Text| {
                    let found: Vec<_> = find_numbers(sup.as_ref()).collect();
                    let contains_only = found.contains(&sub) && found.len() == 1;
                    if contains_only {
                        failure_explaination = None;
                    }
                    failure_explaination.query(move |failure_explaination: Option<Text>| {
                        InnerRuntimeCriterion::ContainNum { sub, sup, found }
                            .build(contains_only, failure_explaination)
                            .into()
                    })
                })
            }

            Self::MatchesRegex {
                failure_explaination,
                pattern,
                sup,
            } => {
                let mut failure_explaination = failure_explaination.clone();
                (pattern, sup).query_n(|pattern: Text, sup: RuntimeValue| {
                    CompiledRegex::from(pattern.clone()).query(|rx: regex::Regex| {
                        let mut is_match = false;
                        match sup {
                            RuntimeValue::Mapping(_)
                            | RuntimeValue::Prim(
                                PrimitiveValue::Number(_), /*| PrimitiveValue::Bool(_) */
                            ) => return cerr::regex_invalidHaystack.into(),
                            RuntimeValue::Array(ref array) => {
                                for item in array.iter() {
                                    if let RuntimeValue::Prim(PrimitiveValue::Str(text)) = item
                                        && rx.is_match(text)
                                    {
                                        is_match = true;
                                        break;
                                    }
                                }
                            }
                            RuntimeValue::Prim(PrimitiveValue::Str(ref text)) => {
                                is_match = rx.is_match(text);
                            }
                        }

                        if is_match {
                            failure_explaination = None;
                        }
                        failure_explaination.query(move |failure_explaination| {
                            InnerRuntimeCriterion::MatchesRegex { pattern, sup }
                                .build(is_match, failure_explaination)
                                .into()
                        })
                    })
                })
            }
            Self::Catch {
                failure_explaination,
                criterion,
                error,
                only_if,
                default_value,
                action,
            } => eval_catch(
                criterion,
                *error,
                *only_if,
                *default_value,
                *action,
                failure_explaination.clone(),
            ),
        }
    }
}

fn eval_catch(
    value: &EntityId<CriterionEntity>,
    error: cerr,
    only_if: Option<EntityId<CriterionEntity>>,
    default_value: Option<EntityId<CriterionEntity>>,
    action: Option<EntityId<ActionEntity>>,
    failure_explaination: Option<ValueReference>,
) -> Machine<RuntimeCriterion> {
    value
        .query(move |value: RuntimeAny| {
            let found_error: cerr = match value {
                RuntimeAny::Criterion(happy) => return Machine::from_final(happy),
                RuntimeAny::Catchable(found_error) => found_error,
                RuntimeAny::Action(_) | RuntimeAny::Value(_) => cerr::typing_notCriterion,
            };
            if !error.contains(found_error) {
                return found_error.into();
            }

            only_if.query(move |only_if: Option<RuntimeCriterion>| {
                if only_if.is_some_and(|only_if| only_if.is_not_fulfilled()) {
                    return found_error.into();
                }
                let machine = if let Some(default_value) = default_value {
                    default_value.query(|default: RuntimeCriterion| default.into())
                } else {
                    Machine::from(found_error)
                };

                if let Some(action) = action {
                    action.query(|action: RuntimeAction| machine.with_action(action))
                } else {
                    machine
                }
            })
        })
        .and_then(|crit: RuntimeCriterion| {
            crit.set_failure_explaination_if_failed(failure_explaination, true)
        })
}

pub fn find_numbers(text: &str) -> impl Iterator<Item = Numeric> {
    text.split(|c: char| !(c.is_ascii_digit() || c == '.' || c == ',' || c == '-' || c == '+'))
        .map(|x| x.trim_end_matches(['+', '-', ',', '.']))
        .filter(|x| !x.is_empty())
        .flat_map(to_number)
}

fn to_number(text: &str) -> Option<Numeric> {
    if let Some((prefix, suffix)) = text.split_once(['.', ','])
        && suffix.trim_end_matches('0').is_empty()
    {
        return prefix.parse().map(Numeric::Int).ok();
    }

    if let Ok(int) = text.parse() {
        Some(Numeric::Int(int))
    } else {
        text.replace(",", ".").parse().map(Numeric::Float).ok()
    }
}

fn eval_contain_with_gaps(
    mut rev_parts: Vec<ValueReference>,
    mut eval_parts: Vec<PrimitiveValue>,
    text: Text,
    // = the number of characters to skip
    start_index: usize,
    failure_explaination: Option<ValueReference>,
) -> Machine<RuntimeCriterion> {
    if let Some(part) = rev_parts.pop() {
        part.query(move |part: PrimitiveValue| {
            if let Some((_start_index_many_chars, rest_to_search)) =
                text.split_at_checked(start_index)
            {
                eval_parts.push(part.clone());
                let part = part.into_text();
                if let Some((before, _after)) = rest_to_search.split_once(part.as_ref()) {
                    // compute a new start index beginning at after
                    let start_index = start_index + before.len() + part.len();
                    debug_assert!(
                        rest_to_search
                            .split_at(start_index)
                            .0
                            .ends_with(part.as_ref())
                    );
                    return eval_contain_with_gaps(
                        rev_parts,
                        eval_parts,
                        text,
                        start_index,
                        failure_explaination,
                    );
                }
            } else {
                eval_parts.push(part);
            }
            failure_explaination.query(move |failure_explaination| {
                InnerRuntimeCriterion::ContainWithGaps {
                    parts: eval_parts,
                    unevaluated_parts: rev_parts.len(),
                    sup: text,
                }
                .build_unfulfilled(failure_explaination)
                .into()
            })
        })
    } else {
        InnerRuntimeCriterion::ContainWithGaps {
            parts: eval_parts,
            unevaluated_parts: rev_parts.len(),
            sup: text,
        }
        .build_fulfilled()
        .into()
    }
}

fn eval_containtext(
    sub: &ValueReference,
    sup: &ValueReference,
    failure_explaination: Option<ValueReference>,
) -> Machine<RuntimeCriterion> {
    (sub, sup).query_n(move |sub: PrimitiveValue, sup: Text| {
        let sub_text = sub.clone().into_text();
        let contains = sup.contains(sub_text.as_ref());

        InnerRuntimeCriterion::ContainText { sub, sup }
            .build(contains, None)
            .set_failure_explaination_if_failed(failure_explaination, true)
    })
}

fn eval_comparison(
    left: &ValueReference,
    right: &ValueReference,
    mut failure_explaination: Option<ValueReference>,
    satisfied_cons: impl FnOnce(PrimitiveValue, PrimitiveValue) -> (bool, InnerRuntimeCriterion)
    + 'static,
) -> Machine<RuntimeCriterion> {
    (left, right).query_n(move |left: PrimitiveValue, right: PrimitiveValue| {
        let (is_fulfilled, inner) = satisfied_cons(left, right);
        inner
            .build(is_fulfilled, None)
            .set_failure_explaination_if_failed(failure_explaination, true)
    })
}

pub(super) fn eval_negated(
    clause: EntityId<CriterionEntity>,
    mut failure_explaination: Option<ValueReference>,
) -> Machine<RuntimeCriterion> {
    clause.query(move |clause: RuntimeCriterion| {
        let this_is_fulfilled = clause.is_not_fulfilled();
        if this_is_fulfilled {
            // If the inner clause is not fulfilled, the outer one (the negated one)
            // will be. So we clear the explaination so that it isn't displayed
            // even if set for failures.
            failure_explaination = None;
        }
        failure_explaination.query(move |outer_failure_explaination: Option<Text>| {
            let failure_explaination =
                outer_failure_explaination.or_else(|| clause.failure_explaination().cloned());

            InnerRuntimeCriterion::Negated {
                clause: Box::new(clause),
            }
            .build(this_is_fulfilled, failure_explaination)
            .into()
        })
    })
}

pub(super) fn eval_if_then_else(
    if_: &EntityId<CriterionEntity>,
    then_: EntityId<CriterionEntity>,
    else_: EntityId<CriterionEntity>,
    mut failure_explaination: Option<ValueReference>,
) -> Machine<RuntimeCriterion> {
    if_.query(move |if_: RuntimeCriterion| {
        let selected_branch = if if_.is_fulfilled() { then_ } else { else_ };
        selected_branch.query(move |selected_branch: RuntimeCriterion| {
            let is_fulfilled = selected_branch.is_fulfilled();
            if is_fulfilled {
                failure_explaination = None;
            }
            failure_explaination.query(move |outer_failure_explaination: Option<Text>| {
                let failure_explaination = outer_failure_explaination
                    .or_else(|| selected_branch.failure_explaination().cloned());

                InnerRuntimeCriterion::IfThenElse {
                    if_: Box::new(if_),
                    selected_branch: Box::new(selected_branch),
                    other_branch: Box::new(MaybeEval::Unevaluated),
                }
                .build(is_fulfilled, failure_explaination)
                .into()
            })
        })
    })
}
fn eval_any_of(
    mut reversed_entities: Vec<EntityId<CriterionEntity>>,
    mut computed_conditions: Vec<MaybeEval<RuntimeCriterion>>,
    failure_explaination: Option<ValueReference>,
) -> Machine<RuntimeCriterion> {
    if let Some(x) = reversed_entities.pop() {
        x.query(|crit: RuntimeCriterion| {
            if crit.is_fulfilled() {
                computed_conditions.push(crit.into());
                computed_conditions.extend(std::iter::repeat_n(
                    MaybeEval::Unevaluated,
                    reversed_entities.len(),
                ));
                InnerRuntimeCriterion::AnyOf {
                    clauses: computed_conditions,
                }
                .build_fulfilled()
                .into()
            } else {
                computed_conditions.push(crit.into());
                eval_any_of(reversed_entities, computed_conditions, failure_explaination)
            }
        })
    } else {
        failure_explaination.query(move |failure_explaination: Option<Text>| {
            InnerRuntimeCriterion::AnyOf {
                clauses: computed_conditions,
            }
            .build_unfulfilled(failure_explaination)
            .into()
        })
    }
}
fn eval_all_of(
    mut reversed_entities: Vec<EntityId<CriterionEntity>>,
    mut computed_conditions: Vec<MaybeEval<RuntimeCriterion>>,
    failure_explaination: Option<ValueReference>,
) -> Machine<RuntimeCriterion> {
    if let Some(x) = reversed_entities.pop() {
        x.query(|crit: RuntimeCriterion| {
            if crit.is_not_fulfilled() {
                computed_conditions.push(crit.into());
                computed_conditions.extend(std::iter::repeat_n(
                    MaybeEval::Unevaluated,
                    reversed_entities.len(),
                ));
                failure_explaination.query(move |failure_explaination: Option<Text>| {
                    InnerRuntimeCriterion::AllOf {
                        clauses: computed_conditions,
                    }
                    .build_unfulfilled(failure_explaination)
                    .into()
                })
            } else {
                computed_conditions.push(crit.into());
                eval_all_of(reversed_entities, computed_conditions, failure_explaination)
            }
        })
    } else {
        InnerRuntimeCriterion::AllOf {
            clauses: computed_conditions,
        }
        .build_fulfilled()
        .into()
    }
}
