use crate::server::handlers::mocks::SetMockRequest;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::RwLock;

pub mod mocks;

/// The shared state accessible to all handlers
pub struct HttpMockState {
    pub mocks: RwLock<Vec<StoredSetMockRequest>>,
    id_counter: AtomicUsize,
}

impl HttpMockState {
    pub fn create_new_id(&self) -> usize {
        self.id_counter.fetch_add(1, Relaxed)
    }
}

#[derive(Serialize, Deserialize, TypedBuilder, Clone, Debug)]
pub struct StoredSetMockRequest {
    id: usize,
    mock: SetMockRequest,
}

impl HttpMockState {
    pub fn new() -> HttpMockState {
        HttpMockState {
            mocks: RwLock::new(Vec::new()),
            id_counter: AtomicUsize::new(0),
        }
    }
}

/// A general abstraction of an HTTP request for all handlers.
#[derive(Serialize, Deserialize, TypedBuilder, Clone, Debug)]
pub struct HttpMockRequest {
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
#[derive(Serialize, Deserialize, TypedBuilder, Clone, Debug)]
pub struct HttpMockResponse {
    pub status: u16,

    #[builder(default=Option::None)]
    pub status_message: Option<String>,

    #[builder(default=Option::None)]
    pub headers: Option<BTreeMap<String, String>>,

    #[builder(default=Option::None)]
    pub body: Option<String>,
}
