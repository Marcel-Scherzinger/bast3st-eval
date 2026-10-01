use std::{collections::BTreeMap, sync::Arc};

use derive_getters::Getters;
use serde::{Deserialize, Serialize};

use crate::{
    catchable::cerr,
    spec::{MapKey, RealMapping, Text, runtime::RuntimeValue},
};

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
    url: Text,
    text: Text,
    json: Result<RuntimeValue, cerr>,
}
impl NetworkResponse {
    pub fn status(&self) -> u16 {
        self.status
    }
    pub async fn new(value: reqwest::Response) -> Result<Self, cerr> {
        let status = value.status().as_u16();
        let url = value.url().to_string().into();
        let text = value.text().await.map_err(|_| cerr::network_external)?;
        let json: Result<RuntimeValue, cerr> =
            serde_json::from_str::<serde_json::Value>(text.as_str())
                .map_err(|_| cerr::network_respInvalid_notJson)
                .map(RuntimeValue::from);
        Ok(NetworkResponse {
            status,
            url,
            json,
            text: text.into(),
        })
    }
}

impl From<NetworkResponse> for RealMapping {
    fn from(value: NetworkResponse) -> Self {
        let mut mapping: BTreeMap<MapKey, Result<RuntimeValue, cerr>> = BTreeMap::new();
        mapping.insert("status".into(), Ok(value.status.into()));
        mapping.insert("url".into(), Ok(value.url.into()));
        mapping.insert("text".into(), Ok(value.text.into()));
        mapping.insert("json".into(), value.json);
        mapping.into()
    }
}
