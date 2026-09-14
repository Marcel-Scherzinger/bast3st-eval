use crate::spec::{
    Text,
    runtime::{NetworkRequest, NetworkResponse},
};

pub trait SpecificTaskRequest {
    type MainOutput: Clone;
}
impl SpecificTaskRequest for NetworkRequest {
    type MainOutput = NetworkResponse;
}
#[derive(Debug, PartialEq, PartialOrd, Clone, derive_more::From)]
pub struct CompiledRegex(Text);
impl SpecificTaskRequest for CompiledRegex {
    type MainOutput = regex::Regex;
}
