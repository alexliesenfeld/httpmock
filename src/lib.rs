//! A lightweight, simple and efficient HTTP mock server that can be used for local tests as
//! well as tests that span multiple systems. It provides a local (or remote) mock server and
//! a library to create, verify and remove HTTP mocks.
//!
//! # Usage
//! If used without a dedicated (standalone) mock server instance, an HTTP mock server will
//! automatically be created in the background of your tests. The local mock server is created
//! in a separate thread that will be started when a test needs a mock server for the first time.
//! It will be shut down at the end of the test run.
//!
//! Should you need to extend or change your tests to span multiple systems later (such as in
//! system integration tests), you can switch the tests to use a standalone mock server by simply
//! setting the address of the remote server using an environment variable. This way the remote
//! server will be used for mocking and your mocks will be available to all participating systems.
//! A standalone version of the HTTP mock server is available as an executable binary.
//!
//! ## Getting Started
//! You can use a local mock server in your tests like shown in the following:
//! ```
//! extern crate mocha;
//!
//! use mocha::mock;
//! use mocha::Method::GET;
//!
//! #[test]
//! fn simple_test() {
//!    // Arrange the test by creating a mock
//!    let health_mock = mock(GET, "/health")
//!       .return_status(200)
//!       .return_header("Content-Type", "application/text")
//!       .return_header("X-Version", "0.0.1")
//!       .return_body("OK")
//!       .create();
//!
//!    // Act (simulates your code)
//!    let response = reqwest::get("http://localhost:5000/health").unwrap();
//!
//!    // Make some assertions
//!    assert_eq!(response.status(), 200);
//!    assert_eq!(health_mock.times_called().unwrap(), 1);
//! }
//! ```
//! As shown in the code snippet, a mock server is automatically created when the [`mock`] function
//! is called. You can provide expected request attributes (such as headers, body content, etc.)
//! and values that will be returned by the mock to the calling application using the
//! [`expect_xxx`] and [`return_xxx`] methods, respectively. The [`create`] method will eventually
//! make a request to the mock server (either local or remote) to create the mock at the server.
//!
//! You can use the mock object returned by the [`create`] method to fetch information about
//! the mock from the mock server, such as the number of times this mock has been called.
//! This object is useful for test assertions.
//!
//! A request is only considered to match a mock if the request contains all attributes required
//! by the mock. If a request does not match any mock previously created, the mock server will
//! respond with an empty response body and a status code 500 (Internal Server Error).
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate typed_builder;

mod server;

pub use server::{start_server, HttpMockConfig};
use server::{HttpMockRequest, HttpMockResponse, SetMockRequest, StoredSetMockRequest};
use std::cell::RefCell;
use std::collections::BTreeMap;

use crate::server::MockCreatedResponse;
use std::io::Read;

use std::sync::{LockResult, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};

lazy_static! {
    static ref SERVER_HOST: &'static str = {
        let host: Option<&str> = option_env!("MOCHA_SERVER_HOST");
        return match host {
            None => "localhost",
            Some(h) => h,
        };
    };
    static ref SERVER_PORT: u16 = {
        let port: Option<&str> = option_env!("MOCHA_SERVER_PORT");
        return match port {
            None => 5000 as u16,
            Some(port_string) => port_string.parse::<u16>().expect(&format!(
                "Cannot parse port from environment variable value '{}'",
                port_string
            )),
        };
    };
    static ref SERVER: Mutex<JoinHandle<()>> = {
        let server_thread = thread::spawn(move || {
            let config = HttpMockConfig::builder()
                .port(*SERVER_PORT as u16)
                .workers(3 as usize)
                .build();

            start_server(config);
        });
        return Mutex::new(server_thread);
    };

    // TODO: Rework how client is being used to allow more parallelism
    static ref CLIENT: Mutex<reqwest::Client> = {
        return Mutex::new(reqwest::Client::new());
    };
}

thread_local!(
    static SERVER_GUARD: RefCell<LockResult<MutexGuard<'static, JoinHandle<()>>>> =
        RefCell::new(SERVER.lock());
);

#[derive(Debug)]
struct ServerAdapter;
impl ServerAdapter {
    pub fn new() -> ServerAdapter {
        SERVER_GUARD.with(|_| {}); // Prevents tests run in parallel
        ServerAdapter {}
    }

    pub fn server_port(&self) -> u16 {
        *SERVER_PORT as u16
    }

    pub fn server_host(&self) -> &str {
        *SERVER_HOST as &str
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_host(), self.server_port())
    }

    pub fn create_mock(&self, mock: &SetMockRequest) -> Result<MockCreatedResponse, String> {
        // Serialize to JSON
        let json = serde_json::to_string(mock);
        if let Err(err) = json {
            return Err(format!("cannot serialize mock object to JSON: {}", err));
        }
        let json = json.unwrap();

        // Send the request to the mock server
        let request_url = format!("http://{}/__mocks", &self.server_address());
        let response;
        {
            let client = CLIENT.lock().unwrap();
            response = client
                .post(request_url.as_str())
                .header("Content-Type", "application/json")
                .body(json)
                .send();
        }
        if let Err(err) = response {
            return Err(format!("cannot send request to mock server: {}", err));
        }

        let mut response = response.unwrap();

        // Extract the response body
        let mut body_contents = String::new();
        let result = response.read_to_string(&mut body_contents);
        if let Err(err) = result {
            return Err(format!("cannot read response body: {}", err));
        }

        // Evaluate the response status
        if response.status() != 201 {
            return Err(format!(
                "could not create mock. Mock server response: status = {}, message = {}",
                response.status(),
                body_contents
            ));
        }

        // Create response object
        let response: serde_json::Result<MockCreatedResponse> =
            serde_json::from_str(body_contents.as_str());
        if let Err(err) = response {
            return Err(format!("cannot deserialize mock server response: {}", err));
        }

        return Ok(response.unwrap());
    }

    pub fn fetch_mock(&self, mock_id: usize) -> Result<StoredSetMockRequest, String> {
        // Send the request to the mock server
        let request_url = format!("http://{}/__mocks/{}", &self.server_address(), mock_id);
        let response;
        {
            let client = CLIENT.lock().unwrap();
            response = client.get(request_url.as_str()).send();
        }
        if let Err(err) = response {
            return Err(format!("cannot send request to mock server: {}", err));
        }
        let mut response = response.unwrap();

        // Extract the response body
        let mut body_contents = String::new();
        let result = response.read_to_string(&mut body_contents);
        if let Err(err) = result {
            return Err(format!("cannot read response body: {}", err));
        }

        // Evaluate response status code
        if response.status() != 200 {
            return Err(format!(
                "could not create mock. Mock server response: status = {}, message = {}",
                response.status(),
                body_contents
            ));
        }

        // Create response object
        let response: serde_json::Result<StoredSetMockRequest> =
            serde_json::from_str(body_contents.as_str());
        if let Err(err) = response {
            return Err(format!("cannot deserialize mock server response: {}", err));
        }

        return Ok(response.unwrap());
    }

    pub fn delete_mock(&self, mock_id: usize) -> Result<(), String> {
        // Send the request to the mock server
        let request_url = format!("http://{}/__mocks/{}", &self.server_address(), mock_id);
        let response;
        {
            let client = CLIENT.lock().unwrap();
            response = client.delete(request_url.as_str()).send();
        }
        if let Err(err) = response {
            return Err(format!("cannot send request to mock server: {}", err));
        }
        let mut response = response.unwrap();

        // Extract the response body
        let mut body_contents = String::new();
        let result = response.read_to_string(&mut body_contents);
        if let Err(err) = result {
            return Err(format!("cannot read response body: {}", err));
        }

        // Evaluate response status code
        if response.status() != 202 {
            return Err(format!(
                "Could not delete mocks from server (status = {}, message = {})",
                response.status(),
                body_contents
            ));
        }

        return Ok(());
    }
}

#[derive(Debug)]
pub struct Mock {
    mock: SetMockRequest,
    server_adapter: ServerAdapter,
    id: Option<usize>,
}

impl Mock {
    pub fn path(mut self, path: &str) -> Self {
        self.mock.request.path = Some(path.to_string());
        self
    }

    pub fn method(mut self, method: Method) -> Self {
        self.mock.request.method = Some(method.as_str().to_string());
        self
    }

    pub fn return_status(mut self, status: usize) -> Self {
        self.mock.response.status = status as u16;
        self
    }

    pub fn return_body(mut self, contents: &str) -> Self {
        self.mock.response.body = Some(contents.to_string());
        self
    }

    pub fn expect_body(mut self, contents: &str) -> Self {
        self.mock.request.body = Some(contents.to_string());
        self
    }

    pub fn return_header(mut self, key: &str, value: &str) -> Self {
        if self.mock.response.headers.is_none() {
            self.mock.response.headers = Some(BTreeMap::new());
        }

        self.mock
            .response
            .headers
            .as_mut()
            .unwrap()
            .insert(key.to_string(), value.to_string());
        self
    }

    pub fn expect_header(mut self, key: &str, value: &str) -> Self {
        if self.mock.request.headers.is_none() {
            self.mock.request.headers = Some(BTreeMap::new());
        }

        self.mock
            .request
            .headers
            .as_mut()
            .unwrap()
            .insert(key.to_string(), value.to_string());

        self
    }

    pub fn create(mut self) -> Self {
        let response = self
            .server_adapter
            .create_mock(&self.mock)
            .expect("Cannot deserialize mock server response");

        self.id = Some(response.mock_id);
        self
    }

    pub fn times_called(&self) -> usize {
        if self.id.is_none() {
            panic!("you cannot fetch the number of calls for a mock that has not yet been created")
        }

        let response = self
            .server_adapter
            .fetch_mock(self.id.unwrap())
            .expect("cannot deserialize mock server response");

        return response.call_counter;
    }
}

impl Drop for Mock {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self
                .server_adapter
                .delete_mock(id)
                .expect("could not delete mock from server");
        }
    }
}

pub enum Method {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}

impl Method {
    pub fn as_str(&self) -> &str {
        match self {
            Method::GET => "GET",
            Method::HEAD => "HEAD",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::DELETE => "DELETE",
            Method::CONNECT => "CONNECT",
            Method::OPTIONS => "OPTIONS",
            Method::TRACE => "TRACE",
            Method::PATCH => "PATCH",
        }
    }
}

pub fn mock(method: Method, path: &str) -> Mock {
    Mock {
        id: None,
        server_adapter: ServerAdapter::new(),
        mock: SetMockRequest {
            request: HttpMockRequest {
                method: Some(method.as_str().to_string()),
                path: Some(String::from(path)),
                headers: None,
                body: None,
            },
            response: HttpMockResponse {
                status: 200,
                status_message: None,
                headers: None,
                body: None,
            },
        },
    }
}
