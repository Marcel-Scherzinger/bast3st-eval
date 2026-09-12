use serde::{Deserialize, Serialize};

use crate::spec::{
    CriterionEntity, EndThisTestMode, EntityId, MessageSendingLevel, MessageSeverity, SetFlagMode,
    ValueReference,
};

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
