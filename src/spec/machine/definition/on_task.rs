use crate::spec::{
    machine::definition::{TaskPushable, normal::Machine},
    runtime::{ClarifiedCerrMerging, CompiledRegex, NetworkRequest},
};

#[derive(derive_more::From)]
pub enum OnTask<R> {
    NetworkRequest((NetworkRequest, TaskPushable<NetworkRequest, R>)),
    CompiledRegex((CompiledRegex, TaskPushable<CompiledRegex, R>)),
}

impl<R: ClarifiedCerrMerging + 'static> OnTask<R> {
    pub fn map<U>(self, closure: impl FnOnce(R) -> U + 'static) -> OnTask<U> {
        match self {
            Self::CompiledRegex((req, inner)) => {
                OnTask::CompiledRegex((req, Box::new(move |val| inner(val).map(closure))))
            }
            Self::NetworkRequest((req, inner)) => {
                OnTask::NetworkRequest((req, Box::new(move |val| inner(val).map(closure))))
            }
        }
    }
    pub fn and_then<U>(self, closure: impl FnOnce(R) -> Machine<U> + 'static) -> OnTask<U> {
        match self {
            Self::CompiledRegex((req, inner)) => {
                OnTask::CompiledRegex((req, Box::new(move |val| inner(val).and_then(closure))))
            }
            Self::NetworkRequest((req, inner)) => {
                OnTask::NetworkRequest((req, Box::new(move |val| inner(val).and_then(closure))))
            }
        }
    }
}
