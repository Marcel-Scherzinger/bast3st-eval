use serde::{Deserialize, Serialize};

use crate::{
    catchable::cerr,
    spec::{
        ActionEntity, CriterionEntity, EntityId, Numeric, PrimitiveValue, ValueReference,
        machine::{Machine, MachineConstruction, MachineConstructionN},
        runtime::RuntimeValue,
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

impl ValueEntity {
    pub fn machine(&self) -> Machine<RuntimeValue> {
        match self {
            Self::Lit { value } => Machine::from_final(RuntimeValue::Prim(value.clone())),
            Self::Add { left, right } => {
                (left, right).and_then_n(|left: Numeric, right: Numeric| {
                    val(left.q_add_numbers(&right, &mut ()))
                })
            }
            Self::Sub { left, right } => {
                (left, right).and_then_n(|left: Numeric, right: Numeric| {
                    val(left.q_sub_numbers(&right, &mut ()))
                })
            }
            Self::Mul { left, right } => {
                (left, right).and_then_n(|left: Numeric, right: Numeric| {
                    val(left.q_mul_numbers(&right, &mut ()))
                })
            }
            _ => todo!(),
        }
    }
}
