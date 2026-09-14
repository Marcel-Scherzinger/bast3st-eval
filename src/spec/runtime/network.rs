use std::{borrow::Cow, collections::BTreeMap, sync::Arc};

use scratch_test_value::SNumber;

use crate::{
    catchable::cerr,
    spec::{
        MapKey, PrimitiveValue, Text,
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

/// - status: u16
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct NetworkResponse {
    status: u16,
}
impl NetworkResponse {
    pub fn new(status: u16) -> Self {
        Self { status }
    }

    pub fn status(&self) -> u16 {
        self.status
    }
}

impl From<NetworkResponse> for Mapping {
    fn from(value: NetworkResponse) -> Self {
        let mut mapping = BTreeMap::new();
        mapping.insert("status".into(), value.status.into());
        mapping.into()
    }
}
