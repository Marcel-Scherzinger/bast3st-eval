#[cfg(not(feature = "no-log-pfx"))]
#[derive(derive_more::Debug, derive_more::Display, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LogPfx(Box<str>);

#[cfg(not(feature = "no-log-pfx"))]
impl LogPfx {
    pub fn new(t: impl Into<Box<str>>) -> Self {
        Self(format!("{:?}", t.into()).into())
    }
    pub fn join(&self, text: impl std::fmt::Debug) -> LogPfx {
        LogPfx(format!("{}/{:?}", self.0, text).into())
    }
}
#[cfg(not(feature = "no-log-pfx"))]
impl From<&str> for LogPfx {
    fn from(value: &str) -> Self {
        Self(format!("{value:?}").into())
    }
}

#[cfg(feature = "no-log-pfx")]
#[derive(derive_more::Debug, derive_more::Display, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[display("")]
pub struct LogPfx(());

#[cfg(feature = "no-log-pfx")]
impl LogPfx {
    pub fn new(t: impl Into<Box<str>>) -> Self {
        Self(())
    }
    pub fn join(&self, text: impl std::fmt::Debug) -> LogPfx {
        LogPfx(())
    }
}
#[cfg(feature = "no-log-pfx")]
impl From<&str> for LogPfx {
    fn from(value: &str) -> Self {
        Self(())
    }
}
