extern crate serde_regex;

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::RwLock;

/// A general abstraction of an HTTP request for all handlers.
#[derive(Serialize, Deserialize, TypedBuilder, Clone, Debug)]
pub struct MockServerHttpRequest {
    pub path: String,
    pub method: String,

    #[builder(default=Option::None)]
    pub headers: Option<BTreeMap<String, String>>,

    #[builder(default=Option::None)]
    pub query_params: Option<BTreeMap<String, String>>,

    #[builder(default=Option::None)]
    pub body: Option<String>,
}

/// A general abstraction of an HTTP response for all handlers.
#[derive(Serialize, Deserialize, TypedBuilder, Clone, Debug)]
pub struct MockServerHttpResponse {
    pub status: u16,

    #[builder(default=Option::None)]
    pub headers: Option<BTreeMap<String, String>>,

    #[builder(default=Option::None)]
    pub body: Option<String>,
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

/// A general abstraction of an HTTP request for all handlers.
#[derive(Serialize, Deserialize, TypedBuilder, Clone, Debug)]
pub struct RequestRequirements {
    #[builder(default=Option::None)]
    pub path: Option<String>,

    #[builder(default=Option::None)]
    pub path_contains: Option<Vec<String>>,

    #[builder(default=Option::None)]
    pub path_matches: Option<Vec<Pattern>>,

    #[builder(default=Option::None)]
    pub method: Option<String>,

    #[builder(default=Option::None)]
    pub headers: Option<BTreeMap<String, String>>,

    #[builder(default=Option::None)]
    pub header_exists: Option<Vec<String>>,

    #[builder(default=Option::None)]
    pub body: Option<String>,

    #[builder(default=Option::None)]
    pub body_contains: Option<Vec<String>>,

    #[builder(default=Option::None)]
    pub body_matches: Option<Vec<Pattern>>,

    #[builder(default=Option::None)]
    pub query_param_exists: Option<Vec<String>>,

    #[builder(default=Option::None)]
    pub query_param: Option<BTreeMap<String, String>>,
}

/// A Request that is made to set a new mock.
#[derive(Serialize, Deserialize, TypedBuilder, Clone, Debug)]
pub struct MockDefinition {
    pub request: RequestRequirements,
    pub response: MockServerHttpResponse,
}

#[derive(Serialize, Deserialize, TypedBuilder, Clone, Debug)]
pub struct MockIdentification {
    pub mock_id: usize,
}

/// The shared state accessible to all handlers
pub struct ApplicationState {
    pub mocks: RwLock<BTreeMap<usize, ActiveMock>>,
    id_counter: AtomicUsize,
}

impl ApplicationState {
    pub fn create_new_id(&self) -> usize {
        self.id_counter.fetch_add(1, Relaxed)
    }

    pub fn new() -> ApplicationState {
        ApplicationState {
            mocks: RwLock::new(BTreeMap::new()),
            id_counter: AtomicUsize::new(0),
        }
    }
}

#[derive(Serialize, Deserialize, TypedBuilder, Clone, Debug)]
pub struct ActiveMock {
    pub id: usize,
    pub call_counter: usize,
    pub definition: MockDefinition,
}

impl ActiveMock {
    pub fn new(id: usize, mock_definition: MockDefinition) -> ActiveMock {
        ActiveMock {
            id,
            definition: mock_definition,
            call_counter: 0,
        }
    }
}
