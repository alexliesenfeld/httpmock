use crate::handlers::mocks::SetMockRequest;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::RwLock;

pub mod mocks;

/// The shared state accessible to all handlers
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

/// A protocol scheme.
#[derive(Serialize, Deserialize, Debug)]
pub enum Scheme {
    HTTP,
    HTTPS,
}

impl Scheme {
    pub fn as_str(&self) -> &'static str {
        match self {
            Scheme::HTTP => "http",
            Scheme::HTTPS => "https",
        }
    }
}

/// A general abstraction of an HTTP request for all handlers.
#[derive(Serialize, Deserialize, TypedBuilder, Debug)]
pub struct HttpMockRequest {
    #[builder(default=Option::None)]
    pub scheme: Option<Scheme>,

    #[builder(default=Option::None)]
    pub path: Option<String>,

    #[builder(default=Option::None)]
    pub method: Option<String>,

    #[builder(default=Option::None)]
    pub headers: Option<BTreeMap<String, String>>,

    #[builder(default=Option::None)]
    pub body: Option<String>,
}

/// A general abstraction of an HTTP response for all handlers.
#[derive(Serialize, Deserialize, TypedBuilder, Debug, Clone)]
pub struct HttpMockResponse {
    pub status: u16,

    #[builder(default=Option::None)]
    pub status_message: Option<String>,

    #[builder(default=Option::None)]
    pub headers: Option<BTreeMap<String, String>>,

    #[builder(default=Option::None)]
    pub body: Option<String>,
}
