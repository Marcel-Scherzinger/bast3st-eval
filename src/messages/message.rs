use crate::spec::{MessageSeverity, Text};

#[derive(Debug, Clone)]
pub struct Message<Level> {
    pub(super) severity: MessageSeverity,
    pub(super) text: Text,
    pub(super) _phantom: std::marker::PhantomData<Level>,
}

impl<L> Message<L> {
    pub fn change_level<O>(self) -> Message<O>
    where
        O: From<L>,
    {
        Message {
            severity: self.severity,
            text: self.text,
            _phantom: Default::default(),
        }
    }
    pub fn new(severity: MessageSeverity, text: Text) -> Self {
        Self {
            severity,
            text,
            _phantom: Default::default(),
        }
    }
    pub fn severity(&self) -> MessageSeverity {
        self.severity
    }
    pub fn text(&self) -> &Text {
        &self.text
    }
}
