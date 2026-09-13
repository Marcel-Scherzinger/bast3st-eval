use scratch_test_value::SNumber;
use serde::{Deserialize, Serialize};

use crate::{
    catchable::cerr,
    spec::{
        ActionEntity, CriterionEntity, EntityId, Numeric, PrimitiveValue, Text, ValueReference,
        entities::eval_if_then_else,
        machine::{Machine, MachineConstruction, MachineConstructionN},
        runtime::{Array, Mapping, RuntimeCriterion, RuntimeValue, Selector},
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

            _ => todo!(),
        }
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
