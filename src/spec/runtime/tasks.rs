use crate::{
    catchable::cerr,
    spec::{
        SpecializeFrom, Text,
        runtime::{NetworkRequest, NetworkResponse},
    },
};

pub trait SpecificTaskRequest {
    type MainOutput: Clone;
}
impl SpecificTaskRequest for NetworkRequest {
    type MainOutput = Result<NetworkResponse, cerr>;
}
#[derive(Debug, PartialEq, PartialOrd, Clone, derive_more::From)]
pub struct CompiledRegex(Text);

impl CompiledRegex {
    pub fn requested_pattern(&self) -> &Text {
        &self.0
    }
}

impl SpecificTaskRequest for CompiledRegex {
    type MainOutput = Result<regex::Regex, cerr>;
}

impl SpecializeFrom<Result<Self, cerr>> for regex::Regex {
    fn specialize_from<'a>(
        any: std::borrow::Cow<'a, Result<Self, cerr>>,
    ) -> Result<std::borrow::Cow<'a, Self>, cerr>
    where
        Self: Sized,
    {
        use std::borrow::Cow;
        match any {
            Cow::Owned(Ok(re)) => Ok(Cow::Owned(re)),
            Cow::Owned(Err(err)) => Err(err),
            Cow::Borrowed(res) => match res.as_ref() {
                Ok(re) => Ok(Cow::Borrowed(re)),
                Err(err) => Err(*err),
            },
        }
    }
}

impl SpecializeFrom<Result<Self, cerr>> for NetworkResponse {
    fn specialize_from<'a>(
        any: std::borrow::Cow<'a, Result<Self, cerr>>,
    ) -> Result<std::borrow::Cow<'a, Self>, cerr>
    where
        Self: Sized,
    {
        use std::borrow::Cow;
        match any {
            Cow::Owned(Ok(re)) => Ok(Cow::Owned(re)),
            Cow::Owned(Err(err)) => Err(err),
            Cow::Borrowed(res) => match res.as_ref() {
                Ok(re) => Ok(Cow::Borrowed(re)),
                Err(err) => Err(*err),
            },
        }
    }
}
