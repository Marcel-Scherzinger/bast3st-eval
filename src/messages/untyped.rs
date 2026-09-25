use derive_more::Deref;
use either::Either;
use itertools::Itertools;

use crate::{
    Messages,
    messages::Message,
    spec::{AlternativeTest, Bast3StSpec, Category, MainTest, MessageSendingLevel, SendMsgAction},
};

#[derive(Debug, Default, PartialEq, PartialOrd, Clone, Deref)]
pub struct AnyMessages(Vec<SendMsgAction>);

pub trait MsgType {
    const LEVEL: MessageSendingLevel;
}
impl MsgType for Bast3StSpec {
    const LEVEL: MessageSendingLevel = MessageSendingLevel::Spec;
}
impl MsgType for Category {
    const LEVEL: MessageSendingLevel = MessageSendingLevel::Category;
}
impl MsgType for MainTest {
    const LEVEL: MessageSendingLevel = MessageSendingLevel::Maintest;
}
impl MsgType for AlternativeTest {
    const LEVEL: MessageSendingLevel = MessageSendingLevel::Current;
}

impl AnyMessages {
    pub fn drain_msg_of<T: MsgType>(&mut self) -> Messages<T> {
        let (a, b) = std::mem::take(&mut self.0)
            .into_iter()
            .partition_map(|msg| {
                if msg.level() == &MessageSendingLevel::Current || msg.level() == &T::LEVEL {
                    Either::Right(Message::<T>::new(*msg.severity(), msg.text().clone()))
                } else {
                    Either::Left(msg)
                }
            });
        self.0 = a;
        b
    }
}
impl Extend<SendMsgAction> for AnyMessages {
    fn extend<T: IntoIterator<Item = SendMsgAction>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

impl IntoIterator for AnyMessages {
    type Item = SendMsgAction;
    type IntoIter = std::vec::IntoIter<SendMsgAction>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
