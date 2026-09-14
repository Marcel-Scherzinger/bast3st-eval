use scratch_test_value::SNumber;
use serde::{Deserialize, Serialize};

use crate::{
    catchable::cerr,
    spec::{
        ActionEntity, CriterionEntity, EntityId, MapKey, NetworkMethod, Numeric, PrimitiveValue,
        Text, ValueReference,
        entities::eval_if_then_else,
        machine::{Machine, MachineConstruction, MachineConstructionN},
        runtime::{
            Array, Mapping, NetworkRequest, NetworkResponse, RuntimeAction, RuntimeAny,
            RuntimeCriterion, RuntimeValue, Selector,
        },
    },
};
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
        method: NetworkMethod,
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
fn val(x: impl Into<RuntimeValue>) -> Machine<RuntimeValue> {
    x.into().into()
}
macro_rules! map_simple {
    ($first: ident: &$ty: ty ; $func: ident) => {
        $first.query_ref(|$first: &$ty| val($first.$func()))
    };
    ($first: ident: $ty: ty ; $func: ident) => {
        $first.query(|$first: $ty| val($first.$func()))
    };
}

impl ValueEntity {
    pub fn machine(&self) -> Machine<RuntimeValue> {
        match self {
            Self::Lit { value } => Machine::from_final(RuntimeValue::Prim(value.clone())),
            Self::Add { left, right } => (left, right)
                .query_n(|left: Numeric, right: Numeric| val(left.q_add_numbers(&right, &mut ()))),
            Self::Sub { left, right } => (left, right)
                .query_n(|left: Numeric, right: Numeric| val(left.q_sub_numbers(&right, &mut ()))),
            Self::Mul { left, right } => (left, right)
                .query_n(|left: Numeric, right: Numeric| val(left.q_mul_numbers(&right, &mut ()))),
            Self::ReadInput {} => Selector::Input.std_machine::<Array>(),
            Self::ReadRandoms {} => Selector::Randoms.std_machine::<Array>(),
            Self::ReadOutput {} => Selector::Output.std_machine::<Array>(),
            Self::ReadParam {} => Selector::Param.std_machine::<Mapping>(),
            Self::ReadLists {} => Selector::Lists.std_machine::<Mapping>(),
            Self::ReadBlockcount {} => Selector::Blockcount.std_machine::<Mapping>(),
            Self::ReadVariables {} => Selector::Variables.std_machine::<Mapping>(),
            Self::IfThenElse { if_, then_, else_ } => {
                eval_if_then_else(if_, then_.clone(), else_.clone())
            }
            Self::Concat { a } => eval_concat(a.iter().rev().cloned().collect(), String::default()),
            Self::View {
                perspective,
                mapping,
            } => mapping.query({
                let perspective = perspective.clone();
                move |mapping: Mapping| match perspective.as_str() {
                    "keys" => val(mapping.keys().cloned().collect::<Array>()),
                    "values" => val(mapping.values().cloned().collect::<Array>()),
                    _ => cerr::exotic_unknownPerspective.into(),
                }
            }),
            Self::Trim { value } => map_simple!(value: &Text; trim),
            Self::TrimStart { value } => map_simple!(value: &Text; trim_start),
            Self::TrimEnd { value } => map_simple!(value: &Text; trim_end),
            Self::ToUpper { value } => map_simple!(value: &Text; to_uppercase),
            Self::ToLower { value } => map_simple!(value: &Text; to_lowercase),
            Self::Abs { value } => map_simple!(value: &Numeric; abs),
            Self::Floor { value } => map_simple!(value: &Numeric; floor),
            Self::Ceil { value } => map_simple!(value: &Numeric; ceil),
            Self::Round { value } => map_simple!(value: &Numeric; round),
            Self::Neg { value } => value
                .query_ref(|value: &Numeric| val(SNumber::Int(0).q_sub_numbers(value, &mut ()))),
            Self::Truediv { left, right } => (left, right)
                .query_n(|left: Numeric, right: Numeric| val(left.q_div_numbers(&right, &mut ()))),
            Self::Floordiv { left, right } => {
                (left, right).query_n(|left: Numeric, right: Numeric| {
                    val(left.q_div_numbers(&right, &mut ()).floor())
                })
            }
            Self::Mod { left, right } => (left, right)
                .query_n(|left: Numeric, right: Numeric| val(left.q_modulo(&right, &mut ()))),
            Self::Pow { left, right } => (left, right).query_n(|left: Numeric, right: Numeric| {
                let int_try = match (left, right) {
                    (Numeric::Int(left), Numeric::Int(r)) if let Ok(right) = r.try_into() => {
                        left.checked_pow(right).map(Numeric::Int).map(val)
                    }
                    _ => None,
                };
                if let Some(int_try) = int_try {
                    int_try
                } else {
                    val(Numeric::Float(
                        left.q_as_float(&mut ()).powf(right.q_as_float(&mut ())),
                    ))
                }
            }),
            Self::Mapitem { mapping, keys } => {
                let reversed_keys = keys.iter().rev().cloned().collect();
                mapping.query(|mapping| eval_mapitem(mapping, reversed_keys))
            }
            Self::Catch {
                value,
                error,
                only_if,
                default_value,
                action,
            } => {
                let error = *error;
                let default_value = default_value.clone();
                let action = *action;
                let only_if = *only_if;
                let on_only_if_not_false = move |found_error: cerr| {
                    let machine = if let Some(default_value) = default_value {
                        default_value.query(|default: RuntimeValue| default.into())
                    } else {
                        Machine::from(found_error)
                    };

                    if let Some(action) = action {
                        action.query(|action: RuntimeAction| machine.with_action(action))
                    } else {
                        machine
                    }
                };

                value.query(move |value: RuntimeAny| {
                    let found_error: cerr = match value {
                        RuntimeAny::Value(happy) => return Machine::from_final(happy),
                        RuntimeAny::Catchable(found_error) => found_error,
                        RuntimeAny::Action(_) | RuntimeAny::Criterion(_) => cerr::typing_notValue,
                    };
                    if !error.contains(found_error) {
                        return found_error.into();
                    }

                    if let Some(only_if) = only_if {
                        only_if.query(move |only_if: RuntimeCriterion| {
                            if !only_if.0 {
                                return found_error.into();
                            }
                            on_only_if_not_false(found_error)
                        })
                    } else {
                        on_only_if_not_false(found_error)
                    }
                })
            }
            Self::NetworkRequest {
                server,
                route,
                method,
                allowed_status,
                json,
            } => {
                let reversed_json = json
                    .as_ref()
                    .map(|json| json.iter().cloned().rev().collect());
                let server: Text = server.into();
                let method = *method;
                let allowed_status = allowed_status.clone();
                route.query(move |route: Text| {
                    let req = match method {
                        NetworkMethod::Get => {
                            // TODO: check if json is set even if it shouldn't
                            NetworkRequest::new_get(server, route).query(Machine::from_final)
                        }
                        NetworkMethod::Post => {
                            if let Some(reversed_json) = reversed_json {
                                eval_post_request(server, route, reversed_json, vec![])
                            } else {
                                NetworkRequest::new_post(server, route, None)
                                    .query(Machine::from_final)
                            }
                        }
                    };
                    req.and_then(|resp: NetworkResponse| {
                        if allowed_status.is_some_and(|allowed| !allowed.contains(resp.status())) {
                            return cerr::network_statusDisallowed.into();
                        }
                        RuntimeValue::Mapping(resp.into()).into()
                    })
                })
            }
            _ => todo!(),
        }
    }
}

fn eval_post_request(
    server: Text,
    route: Text,
    mut reversed_json: Vec<(ValueReference, ValueReference)>,
    mut final_json: Vec<(Text, RuntimeValue)>,
) -> Machine<NetworkResponse> {
    if let Some((k, v)) = reversed_json.pop() {
        (&k, &v).query_n(|k: Text, v: RuntimeValue| {
            final_json.push((k, v));
            eval_post_request(server, route, reversed_json, final_json)
        })
    } else {
        NetworkRequest::new_post(server, route, Some(final_json)).query(Machine::from_final)
    }
}

fn eval_mapitem(
    current: RuntimeValue,
    mut reversed_keys: Vec<ValueReference>,
) -> Machine<RuntimeValue> {
    if let Some(key) = reversed_keys.pop() {
        key.query(|key: MapKey| match current {
            RuntimeValue::Mapping(mapping) => {
                if let Some(next) = mapping.get(&key).cloned() {
                    eval_mapitem(next, reversed_keys)
                } else {
                    cerr::mapping_missingKey.into()
                }
            }
            _ => cerr::typing_notMapping.into(),
        })
    } else {
        Machine::from_final(current)
    }
}

fn eval_concat(
    mut reversed_exprs: Vec<ValueReference>,
    mut finished: String,
) -> Machine<RuntimeValue> {
    if let Some(x) = reversed_exprs.pop() {
        x.query_ref(|next: &Text| {
            finished.push_str(next);
            eval_concat(reversed_exprs, finished)
        })
    } else {
        Machine::from_final(RuntimeValue::Prim(PrimitiveValue::Str(finished.into())))
    }
}
