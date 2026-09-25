use derive_getters::Getters;
use serde::{Deserialize, Serialize};

use crate::{
    Features,
    evaluation::RequiredFeatures,
    spec::{
        CriterionEntity, EndThisTestMode, EntityId, MachineConstruction, MapKey,
        MessageSendingLevel, MessageSeverity, PrimitiveValue, SetFlagMode, Text, ValueReference,
        machine::Machine, runtime::RuntimeCriterion,
    },
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone)]
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
        #[serde(rename = "m", default)]
        mode: SetFlagMode,
        #[serde(rename = "k")]
        key: Vec<ValueReference>,
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
                    RuntimeAction::SendMsg(SendMsgAction {
                        text: text.into_text(),
                        severity,
                        level,
                    })
                    .into()
                })
            }
            Self::EndThisTest { mode, explaination } => {
                let mode = *mode;
                explaination.query(move |explaination: PrimitiveValue| {
                    RuntimeAction::EndThisTest(EndThisTestAction {
                        mode,
                        explaination: explaination.into_text(),
                    })
                    .into()
                })
            }
            Self::IfThenElse { if_, then_, else_ } => {
                let (then_, else_) = (*then_, *else_);
                if_.query(move |if_: RuntimeCriterion| {
                    if if_.is_fulfilled() { then_ } else { else_ }
                        .query(|branch: RuntimeAction| branch.into())
                })
            }
            Self::SetFlag { mode, key, value } => set_flag_act(
                mode.clone(),
                key.iter().rev().cloned().collect(),
                vec![],
                value.clone(),
            ),
        }
        .require_features(self.required_features())
    }
}

fn set_flag_act(
    mode: SetFlagMode,
    mut reversed_keys: Vec<ValueReference>,
    mut eval_keys: Vec<MapKey>,
    value: ValueReference,
) -> Machine<RuntimeAction> {
    if let Some(key) = reversed_keys.pop() {
        key.query(move |key: MapKey| {
            eval_keys.push(key);
            set_flag_act(mode, reversed_keys, eval_keys, value)
        })
    } else {
        value.query(move |value: PrimitiveValue| {
            RuntimeAction::SetFlag(SetFlagAction {
                mode,
                key: eval_keys,
                value,
            })
            .into()
        })
    }
}

impl RequiredFeatures for ActionEntity {
    fn required_features(&self) -> Features {
        match self {
            Self::SendMsg { level, .. } => level.required_features(),
            Self::EndThisTest { .. } => Features::END_THIS_TEST,
            Self::SetFlag { .. } | Self::IfThenElse { .. } => Features::empty(),
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum RuntimeAction {
    SendMsg(SendMsgAction),
    EndThisTest(EndThisTestAction),
    SetFlag(SetFlagAction),
}
#[derive(Debug, PartialEq, PartialOrd, Clone, Getters)]
pub struct SetFlagAction {
    mode: SetFlagMode,
    key: Vec<MapKey>,
    value: PrimitiveValue,
}
impl SetFlagAction {
    pub fn into_parts(self) -> (SetFlagMode, Vec<MapKey>, PrimitiveValue) {
        (self.mode, self.key, self.value)
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Getters)]
pub struct EndThisTestAction {
    mode: EndThisTestMode,
    explaination: Text,
}

impl EndThisTestMode {
    pub fn action_with(self, explaination: Text) -> EndThisTestAction {
        EndThisTestAction {
            mode: self,
            explaination,
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Getters)]
pub struct SendMsgAction {
    level: MessageSendingLevel,
    severity: MessageSeverity,
    text: Text,
}
impl MessageSeverity {
    pub fn action_with(self, level: MessageSendingLevel, text: Text) -> SendMsgAction {
        SendMsgAction {
            severity: self,
            level,
            text,
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum ProcessedAction {}
