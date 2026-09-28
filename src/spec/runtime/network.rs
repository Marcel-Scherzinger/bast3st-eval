use std::{collections::BTreeMap, sync::Arc};

use derive_getters::Getters;
use serde::{Deserialize, Serialize};

use crate::spec::{RealMapping, Text, runtime::RuntimeValue};

#[derive(Debug, PartialEq, PartialOrd, Clone, Getters)]
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
    pub fn into_specific(self) -> InnerNetworkRequest {
        self.specific
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InnerNetworkRequest {
    Get,
    #[serde(rename_all = "kebab-case")]
    Post {
        json: Option<Arc<[(Text, RuntimeValue)]>>,
    },
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
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

impl From<reqwest::Response> for NetworkResponse {
    fn from(value: reqwest::Response) -> Self {
        todo!()
    }
}

impl From<NetworkResponse> for RealMapping {
    fn from(value: NetworkResponse) -> Self {
        let mut mapping = BTreeMap::new();
        mapping.insert("status".into(), value.status.into());
        mapping.into()
    }
}
