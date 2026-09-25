use std::borrow::Cow;

use derive_getters::Getters;
use either::Either;
use itertools::Itertools;

use crate::{
    Features, Messages,
    evaluation::{FlagData, SelectableSource},
    messages::{AnyMessages, MsgType},
    spec::{FatalError, RuntimeAction, RuntimeAny, SendMsgAction},
};

#[derive(Debug, Default, Getters, PartialEq, PartialOrd, Clone)]
pub struct Effects {
    flags: FlagData,
    messages: AnyMessages,
}

impl Effects {
    pub fn append(&mut self, other: Effects) {
        self.messages.extend(other.messages);
        let flags = std::mem::take(&mut self.flags);
        self.flags = flags.with_append(other.flags);
    }
    pub fn take_messages<L: MsgType>(&mut self) -> Messages<L> {
        self.messages.drain_msg_of()
    }
}

impl SelectableSource for Effects {
    async fn request<'a>(
        &'a self,
        selector: &crate::spec::Selector,
        allowed_features: Features,
    ) -> Result<Cow<'a, RuntimeAny>, FatalError> {
        self.flags.request(selector, allowed_features).await
    }
}

impl Extend<RuntimeAction> for Effects {
    fn extend<T: IntoIterator<Item = RuntimeAction>>(&mut self, iter: T) {
        let (other, messages): (Vec<RuntimeAction>, Vec<SendMsgAction>) = iter
            .into_iter()
            .flat_map(|item| match item {
                RuntimeAction::SendMsg(send) => Some(Either::Right(send)),
                RuntimeAction::EndThisTest(_) => None,
                act @ RuntimeAction::SetFlag { .. } => Some(Either::Left(act)),
            })
            .partition_map(|x| x);
        self.flags.extend(&other);
        self.messages.extend(messages);
    }
}
