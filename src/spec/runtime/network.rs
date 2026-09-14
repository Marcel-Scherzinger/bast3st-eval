use std::sync::Arc;

use crate::{
    catchable::cerr,
    spec::{
        Text,
        runtime::{Mapping, RuntimeAny, RuntimeValue, SpecializeFrom},
    },
};

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct NetworkRequest {
    server: Text,
    route: Text,
    specific: InnerNetworkRequest,
}
impl NetworkRequest {
    pub fn new_get(server: Text, route: Text) -> Self {
        Self {
            server,
            route,
            specific: InnerNetworkRequest::Get,
        }
    }
    pub fn new_post(server: Text, route: Text, json: Option<Vec<(Text, RuntimeValue)>>) -> Self {
        Self {
            server,
            route,
            specific: InnerNetworkRequest::Post {
                json: json.map(|x| x.into()),
            },
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum InnerNetworkRequest {
    Get,
    Post {
        json: Option<Arc<[(Text, RuntimeValue)]>>,
    },
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct NetworkResponse {
    status: u16,
}
impl NetworkResponse {
    pub fn status(&self) -> &u16 {
        &self.status
    }
}

impl SpecializeFrom<RuntimeAny> for NetworkResponse {
    fn specialize_from<'a>(
        any: &'a RuntimeAny,
    ) -> Result<std::borrow::Cow<'a, Self>, crate::catchable::cerr>
    where
        Self: Sized,
    {
        match any {
            RuntimeAny::Value(RuntimeValue::Mapping(m)) => Self::specialize_from(m),
            RuntimeAny::Catchable(err) => Err(*err),
            _ => Err(cerr::typing_notNetworkResp),
        }
    }
}
impl SpecializeFrom<Mapping> for NetworkResponse {
    fn specialize_from<'a>(
        any: &'a Mapping,
    ) -> Result<std::borrow::Cow<'a, Self>, crate::catchable::cerr>
    where
        Self: Sized,
    {
        todo!()
    }
}
impl From<NetworkResponse> for Mapping {
    fn from(value: NetworkResponse) -> Self {
        todo!()
    }
}
