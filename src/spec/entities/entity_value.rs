use scratch_test_value::SNumber;
use serde::{Deserialize, Serialize};

use crate::{
    catchable::cerr,
    spec::{
        ActionEntity, Coremapping, CriterionEntity, EntityId, FatalError, MapKey, MappingOrArray,
        NetworkMethod, Numeric, PrimitiveValue, RealMapping, RuntimeAction, Text, ValueReference,
        machine::{Machine, MachineConstruction, MachineConstructionN},
        runtime::{
            Array, ClarifiedCerrMerging, CompiledRegex, NetworkRequest, NetworkResponse,
            PossibleRuntimeValue, RuntimeAny, RuntimeCriterion, RuntimeValue, Selector,
            SpecializeFrom,
        },
    },
};
pub type MappingReference = ValueReference;

#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize, Clone)]
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
    ReadFlags {},
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

    #[serde(rename = "cmapitem")]
    CoremapItem {
        #[serde(rename = "m")]
        mapping: Coremapping,
        #[serde(rename = "k")]
        keys: Vec<ValueReference>,
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
        #[serde(rename = "p")]
        pattern: ValueReference,
        sup: ValueReference,
        group: Option<ValueReference>,
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
    Property {
        #[serde(rename = "p")]
        perspective: PropertyPerspective,
        // can also be mapping/array
        #[serde(rename = "v")]
        value: ValueReference,
    },
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PropertyPerspective {
    Sum,
    Length,
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
            Self::ReadInput {} => Selector::INPUT.std_machine::<Array>(),
            Self::ReadRandoms {} => Selector::RANDOMS.std_machine::<Array>(),
            Self::ReadOutput {} => Selector::OUTPUT.std_machine::<Array>(),
            Self::ReadParam {} => Selector::PARAM.std_machine::<RealMapping>(),
            Self::ReadFlags {} => Selector::FLAGS.std_machine::<RealMapping>(),
            Self::ReadLists {} => Selector::LISTS.std_machine::<RealMapping>(),
            Self::ReadVariables {} => Selector::VARIABLES.std_machine::<RealMapping>(),
            Self::IfThenElse { if_, then_, else_ } => {
                eval_flat_if_then_else(if_, then_.clone(), else_.clone())
            }
            Self::Concat { a } => eval_concat(a.iter().rev().cloned().collect(), String::default()),
            Self::View {
                perspective,
                mapping,
            } => mapping.query({
                let perspective = perspective.clone();
                move |mapping: MappingOrArray| match (mapping, perspective.as_str()) {
                    (mapping, "keys") => {
                        val(mapping.keys().map(|x| x.to_prim()).collect::<Array>())
                    }
                    (mapping, "values") => val(mapping.value_array()),
                    (mapping, "sum") => val(mapping.sum()),
                    (_, _) => FatalError::UnknownViewPerspective(perspective.into()).into(),
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
            Self::CoremapItem { mapping, keys } => {
                let reversed_keys = keys.iter().rev().cloned().collect();
                eval_coremap_item(*mapping, reversed_keys, Default::default())
            }
            Self::Catch {
                value,
                error,
                only_if,
                default_value,
                action,
            } => eval_catch(value, *error, *only_if, default_value.clone(), *action),
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
                eval_network_request(
                    server.into(),
                    route.clone(),
                    *method,
                    allowed_status.clone(),
                    reversed_json,
                )
            }
            Self::Property { perspective, value } => {
                let perspective = *perspective;
                value.query(move |value: RuntimeValue| match perspective {
                    PropertyPerspective::Sum => {
                        let sum: Numeric = match value {
                            RuntimeValue::Comp(ma) => ma.sum(),
                            value @ RuntimeValue::Prim(_) => [value].iter().sum(),
                        };
                        RuntimeValue::Prim(PrimitiveValue::Number(sum)).into()
                    }
                    PropertyPerspective::Length => {
                        let length: usize = match value {
                            RuntimeValue::Comp(ma) => ma.len(),
                            RuntimeValue::Prim(PrimitiveValue::Str(text)) => text.chars().count(),
                            _ => return cerr::typing_notIterable.into(),
                        };
                        let length = length.try_into().unwrap_or(i64::MAX);
                        RuntimeValue::Prim(PrimitiveValue::Number(SNumber::Int(length))).into()
                    }
                })
            }
            Self::FirstCaptureOfRegex {
                pattern,
                sup,
                group,
            } => {
                let group = group.clone();

                let process_capture_group = move |capture: regex::Captures<'_>, group| {
                    match group {
                        PrimitiveValue::Number(SNumber::Int(group))
                            if let Ok(group) = group.try_into() =>
                        {
                            capture.get(group)
                        }
                        PrimitiveValue::Str(name) => capture.name(&name),
                        _ => return cerr::regex_noGroup.into(),
                    }
                    .map(|mat| RuntimeValue::from(Text::from(mat.as_str())))
                    .ok_or(cerr::regex_noGroup)
                    .into()
                };

                let do_for_group = move |pattern, sup, group: PrimitiveValue| {
                    CompiledRegex::from(pattern).query(move |pattern: regex::Regex| match sup {
                        RuntimeValue::Comp(MappingOrArray::Mapping(_))
                        | RuntimeValue::Prim(
                            PrimitiveValue::Number(_), /*| PrimitiveValue::Bool(_) */
                        ) => cerr::regex_invalidHaystack.into(),
                        RuntimeValue::Comp(MappingOrArray::Array(array)) => {
                            for item in array.iter() {
                                if let RuntimeValue::Prim(PrimitiveValue::Str(text)) = item
                                    && let Some(capture) = pattern.captures(text)
                                {
                                    return process_capture_group(capture, group);
                                }
                            }
                            Machine::from(cerr::regex_noMatch)
                        }
                        RuntimeValue::Prim(PrimitiveValue::Str(text)) => {
                            let capture = pattern.captures(&text).ok_or(cerr::regex_noMatch);
                            capture.map(|c| process_capture_group(c, group)).into()
                        }
                    })
                };

                (pattern, sup).query_n(move |pattern: Text, sup: RuntimeValue| {
                    if let Some(group) = group {
                        group.query(move |group: PrimitiveValue| do_for_group(pattern, sup, group))
                    } else {
                        do_for_group(pattern, sup, PrimitiveValue::Number(Numeric::Int(0)))
                    }
                })
            }
        }
    }
}

fn eval_catch(
    value: &ValueReference,
    error: cerr,
    only_if: Option<EntityId<CriterionEntity>>,
    default_value: Option<ValueReference>,
    action: Option<EntityId<ActionEntity>>,
) -> Machine<RuntimeValue> {
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
                if only_if.is_not_fulfilled() {
                    return found_error.into();
                }
                on_only_if_not_false(found_error)
            })
        } else {
            on_only_if_not_false(found_error)
        }
    })
}

fn eval_network_request(
    server: Text,
    route: ValueReference,
    method: NetworkMethod,
    allowed_status: Option<Vec<u16>>,
    reversed_json: Option<Vec<(ValueReference, ValueReference)>>,
) -> Machine<RuntimeValue> {
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
                    NetworkRequest::new_post(server, route, None).query(Machine::from_final)
                }
            }
        };
        req.and_then(|resp: NetworkResponse| {
            if allowed_status.is_some_and(|allowed| !allowed.contains(&resp.status())) {
                return cerr::network_statusDisallowed.into();
            }
            RuntimeValue::Comp(MappingOrArray::Mapping(resp.into())).into()
        })
    })
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
            RuntimeValue::Comp(mapping) => match mapping.get(&key) {
                Ok(next) => eval_mapitem(next.clone(), reversed_keys),
                Err(err) => err.into(),
            },
            _ => cerr::typing_notCollection.into(),
        })
    } else {
        Machine::from_final(current)
    }
}

fn eval_coremap_item(
    mapping: Coremapping,
    mut reversed_keys: Vec<ValueReference>,
    mut eval_keys: Vec<MapKey>,
) -> Machine<RuntimeValue> {
    if let Some(key) = reversed_keys.pop() {
        key.query(move |key: MapKey| {
            eval_keys.push(key);
            eval_coremap_item(mapping, reversed_keys, eval_keys)
        })
    } else {
        Selector::CoremapItem {
            mapping,
            key: eval_keys,
        }
        .std_machine::<RuntimeValue>()
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

pub(super) fn eval_flat_if_then_else<
    E,
    R: Clone + SpecializeFrom + ClarifiedCerrMerging + 'static + Send,
>(
    if_: &EntityId<CriterionEntity>,
    then_: E,
    else_: E,
) -> Machine<R>
where
    E: PossibleRuntimeValue<R> + MachineConstruction<E, R> + 'static + Send,
{
    if_.query_ref(move |if_: &RuntimeCriterion| {
        if if_.is_fulfilled() {
            then_.query(|then_| Machine::from_final(then_))
        } else {
            else_.query(|else_| Machine::from_final(else_))
        }
    })
}
