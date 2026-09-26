use std::fmt::Debug;

use derive_more::{Deref, Display, From};

#[derive(derive_more::Debug, Display, Deref, Clone, PartialEq, Eq, PartialOrd, Ord, From)]
pub struct LogPfx(Box<str>);
impl LogPfx {
    pub fn new(t: impl Into<Box<str>>) -> Self {
        Self(format!("{:?}", t.into()).into())
    }
    pub fn join(&self, text: impl Debug) -> LogPfx {
        LogPfx(format!("{}/{:?}", self.0, text).into())
    }
}
impl From<&str> for LogPfx {
    fn from(value: &str) -> Self {
        Self(format!("{value:?}").into())
    }
}
