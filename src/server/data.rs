extern crate serde_regex;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, RwLock};

/// A general abstraction of an HTTP request for all handlers.
#[derive(Serialize, Deserialize, Debug)]
pub struct MockServerHttpRequest {
    pub path: String,
    pub method: String,
    pub headers: Option<BTreeMap<String, String>>,
    pub query_params: Option<BTreeMap<String, String>>,
    pub body: Option<String>,
}

impl MockServerHttpRequest {
    pub fn new(method: String, path: String) -> Self {
        Self {
            path,
            method,
            headers: None,
            query_params: None,
            body: None,
        }
    }

    pub fn with_headers(mut self, arg: BTreeMap<String, String>) -> Self {
        self.headers = Some(arg);
        self
    }

    pub fn with_query_params(mut self, arg: BTreeMap<String, String>) -> Self {
        self.query_params = Some(arg);
        self
    }

    pub fn with_body(mut self, arg: String) -> Self {
        self.body = Some(arg);
        self
    }
}

/// A general abstraction of an HTTP response for all handlers.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MockServerHttpResponse {
    pub status: u16,
    pub headers: Option<BTreeMap<String, String>>,
    pub body: Option<String>,
}

impl MockServerHttpResponse {
    pub fn new(status: u16) -> Self {
        Self {
            status,
            headers: None,
            body: None,
        }
    }

    pub fn with_headers(mut self, arg: BTreeMap<String, String>) -> Self {
        self.headers = Some(arg);
        self
    }

    pub fn with_body(mut self, arg: String) -> Self {
        self.body = Some(arg);
        self
    }
}

/// A general abstraction of an HTTP request for all handlers.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Pattern {
    #[serde(with = "serde_regex")]
    pub regex: Regex,
}

impl Pattern {
    pub fn from_regex(regex: Regex) -> Pattern {
        Pattern { regex }
    }
}

impl Ord for Pattern {
    fn cmp(&self, other: &Self) -> Ordering {
        self.regex.as_str().cmp(other.regex.as_str())
    }
}

impl PartialOrd for Pattern {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Pattern {
    fn eq(&self, other: &Self) -> bool {
        self.regex.as_str() == other.regex.as_str()
    }
}

impl Eq for Pattern {}

pub type MockMatcherClosure = fn(Rc<MockServerHttpRequest>) -> bool;

/// A general abstraction of an HTTP request for all handlers.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RequestRequirements {
    pub path: Option<String>,
    pub path_contains: Option<Vec<String>>,
    pub path_matches: Option<Vec<Pattern>>,
    pub method: Option<String>,
    pub headers: Option<BTreeMap<String, String>>,
    pub header_exists: Option<Vec<String>>,
    pub body: Option<String>,
    pub json_body: Option<Value>,
    pub json_body_includes: Option<Vec<Value>>,
    pub body_contains: Option<Vec<String>>,
    pub body_matches: Option<Vec<Pattern>>,
    pub query_param_exists: Option<Vec<String>>,
    pub query_param: Option<BTreeMap<String, String>>,

    #[serde(skip_serializing, skip_deserializing)]
    pub matchers: Option<Vec<MockMatcherClosure>>,
}

impl RequestRequirements {
    pub fn new() -> Self {
        Self {
            path: None,
            path_contains: None,
            path_matches: None,
            method: None,
            headers: None,
            header_exists: None,
            body: None,
            json_body: None,
            json_body_includes: None,
            body_contains: None,
            body_matches: None,
            query_param_exists: None,
            query_param: None,
            matchers: None,
        }
    }

    pub fn with_path(mut self, arg: String) -> Self {
        self.path = Some(arg);
        self
    }

    pub fn with_method(mut self, arg: String) -> Self {
        self.method = Some(arg);
        self
    }

    pub fn with_body(mut self, arg: String) -> Self {
        self.body = Some(arg);
        self
    }

    pub fn with_json_body(mut self, arg: Value) -> Self {
        self.json_body = Some(arg);
        self
    }

    pub fn with_path_contains(mut self, arg: Vec<String>) -> Self {
        self.path_contains = Some(arg);
        self
    }

    pub fn with_path_matches(mut self, arg: Vec<Pattern>) -> Self {
        self.path_matches = Some(arg);
        self
    }

    pub fn with_headers(mut self, arg: BTreeMap<String, String>) -> Self {
        self.headers = Some(arg);
        self
    }

    pub fn with_header_exists(mut self, arg: Vec<String>) -> Self {
        self.header_exists = Some(arg);
        self
    }

    pub fn with_json_body_includes(mut self, arg: Vec<Value>) -> Self {
        self.json_body_includes = Some(arg);
        self
    }

    pub fn with_body_contains(mut self, arg: Vec<String>) -> Self {
        self.body_contains = Some(arg);
        self
    }

    pub fn with_body_matches(mut self, arg: Vec<Pattern>) -> Self {
        self.body_matches = Some(arg);
        self
    }

    pub fn with_query_param_exists(mut self, arg: Vec<String>) -> Self {
        self.query_param_exists = Some(arg);
        self
    }

    pub fn with_query_param(mut self, arg: BTreeMap<String, String>) -> Self {
        self.query_param = Some(arg);
        self
    }
}

/// A Request that is made to set a new mock.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MockDefinition {
    pub request: RequestRequirements,
    pub response: MockServerHttpResponse,
}

#[derive(Serialize, Deserialize)]
pub struct MockIdentification {
    pub mock_id: usize,
}

impl MockIdentification {
    pub fn new(mock_id: usize) -> Self {
        Self { mock_id }
    }
}

/// The shared state accessible to all handlers
pub struct MockServerState {
    pub mocks: RwLock<BTreeMap<usize, ActiveMock>>,
    id_counter: AtomicUsize,
}

impl MockServerState {
    pub fn create_new_id(&self) -> usize {
        self.id_counter.fetch_add(1, Relaxed)
    }

    pub fn new() -> Self {
        MockServerState {
            mocks: RwLock::new(BTreeMap::new()),
            id_counter: AtomicUsize::new(0),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ActiveMock {
    pub id: usize,
    pub call_counter: usize,
    pub definition: MockDefinition,
}

impl ActiveMock {
    pub fn new(id: usize, mock_definition: MockDefinition) -> Self {
        ActiveMock {
            id,
            definition: mock_definition,
            call_counter: 0,
        }
    }
}
#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub message: String,
}

impl ErrorResponse {
    pub fn new<T>(message: &T) -> ErrorResponse
    where
        T: ToString,
    {
        ErrorResponse {
            message: message.to_string(),
        }
    }
}
