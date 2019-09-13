use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::RwLock;

pub mod mocks;

pub struct HttpMockState {
    pub mocks: RwLock<Vec<SetMockRequest>>,
}

impl HttpMockState {
    pub fn new() -> HttpMockState {
        HttpMockState {
            mocks: RwLock::new(Vec::new()),
        }
    }
}

#[derive(Serialize, Deserialize, TypedBuilder, Debug)]
pub struct HttpMockRequest {
    pub method: Option<String>,
    pub path: Option<String>,
    pub headers: Option<BTreeMap<String, String>>,
    pub body: Option<String>,
}

#[derive(Serialize, Deserialize, TypedBuilder, Debug, Clone)]
pub struct HttpMockResponse {
    pub status: u16,
    pub status_message: Option<String>,
    pub headers: Option<BTreeMap<String, String>>,
    pub body: Option<String>,
}

#[derive(Serialize, Deserialize, TypedBuilder, Debug)]
pub struct SetMockRequest {
    pub request: HttpMockRequest,
    pub response: HttpMockResponse,
}
