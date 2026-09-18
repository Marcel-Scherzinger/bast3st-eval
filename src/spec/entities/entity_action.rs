use serde::{Deserialize, Serialize};

use crate::{
    Features,
    evaluation::RequiredFeatures,
    spec::{
        CriterionEntity, EndThisTestMode, EntityId, MachineConstruction, MachineConstructionN,
        MapKey, MessageSendingLevel, MessageSeverity, PrimitiveValue, SetFlagMode, Text,
        ValueReference, machine::Machine, runtime::RuntimeCriterion,
    },
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case")]
pub enum ActionEntity {
    SendMsg {
        #[serde(rename = "t")]
        text: ValueReference,
        #[serde(rename = "s")]
        severity: MessageSeverity,
        #[serde(rename = "l", default)]
        level: MessageSendingLevel,
    },
    EndThisTest {
        #[serde(rename = "m")]
        mode: EndThisTestMode,
        #[serde(rename = "e")]
        explaination: ValueReference,
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
impl ActionEntity {
    pub fn machine(&self) -> Machine<RuntimeAction> {
        match self {
            Self::SendMsg {
                text,
                severity,
                level,
            } => {
                let severity = *severity;
                let level = *level;
                text.query(move |text: PrimitiveValue| {
                    RuntimeAction::SendMsg {
                        text: text.into_text(),
                        severity,
                        level,
                    }
                    .into()
                })
                .require_features(level.required_features())
            }
            Self::EndThisTest { mode, explaination } => {
                let mode = *mode;
                explaination
                    .query(move |explaination: PrimitiveValue| {
                        RuntimeAction::EndThisTest {
                            mode,
                            explaination: explaination.into_text(),
                        }
                        .into()
                    })
                    .require_features(Features::END_THIS_TEST)
            }
            Self::IfThenElse { if_, then_, else_ } => {
                let (then_, else_) = (*then_, *else_);
                if_.query(move |if_: RuntimeCriterion| {
                    if if_.is_fulfilled() { then_ } else { else_ }
                        .query(|branch: RuntimeAction| branch.into())
                })
            }
            Self::SetFlag { mode, key, value } => {
                let mode = mode.clone();
                (key, value).query_n(move |key: MapKey, value: PrimitiveValue| {
                    RuntimeAction::SetFlag {
                        mode,
                        key: key.into_text(),
                        value,
                    }
                    .into()
                })
            }
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum RuntimeAction {
    SendMsg {
        text: Text,
        severity: MessageSeverity,
        level: MessageSendingLevel,
    },
    EndThisTest {
        mode: EndThisTestMode,
        explaination: Text,
    },

    SetFlag {
        mode: SetFlagMode,
        key: Text,
        value: PrimitiveValue,
    },
}
