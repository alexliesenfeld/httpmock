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
//! A standalone version of the HTTP mock server is available as an executable binary or a Docker
//! image.
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
//!    assert_eq!(health_mock.number_of_calls(), 1);
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
//! respond with an empty response body and a status code 404 (Not Found).
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate typed_builder;

mod server;

pub use server::{start_server, HttpMockConfig};
use server::{HttpMockRequest, HttpMockResponse, SetMockRequest};
use std::cell::RefCell;
use std::collections::BTreeMap;

use std::io::Read;
use std::sync::{LockResult, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};

lazy_static! {
    static ref SERVER: Mutex<JoinHandle<()>> = {
        let server_thread = thread::spawn(move || {
            let config = HttpMockConfig::builder()
                .port(5000 as u16)
                .workers(3 as usize)
                .build();

            start_server(config);
        });
        return Mutex::new(server_thread);
    };
}

thread_local!(
    static SERVER_GUARD: RefCell<LockResult<MutexGuard<'static, JoinHandle<()>>>> =
        RefCell::new(SERVER.lock());
);

#[derive(Debug)]
pub struct Mock {
    server_host: String,
    server_port: u16,
    client: reqwest::Client,
    mock: SetMockRequest,
}

impl Mock {
    pub fn server_port(&self) -> u16 {
        self.server_port
    }

    pub fn server_host(&self) -> &String {
        &self.server_host
    }

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

    pub fn create(self) -> Self {
        SERVER_GUARD.with(|_| {}); // Prevents tests run in parallel

        let json = serde_json::to_string(&self.mock).expect("cannot serialize request");
        let request_url = format!("http://{}/__mocks", &self.server_address());
        let mut res = self
            .client
            .post(request_url.as_str())
            .header("Content-Type", "application/json")
            .body(json)
            .send()
            .expect("Mock server error");

        if res.status() != 201 {
            let mut buf = String::new();
            res.read_to_string(&mut buf)
                .expect("Failed to read response");
            let err_msg = format!(
                "Could not create mock (status = {}, message = {})",
                res.status(),
                buf
            );
            panic!(err_msg);
        }

        self
    }

    pub fn number_of_calls(&self) -> Result<usize, String> {
        Ok(5 as usize)
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}

impl Drop for Mock {
    fn drop(&mut self) {
        let request_url = format!("http://{}/__mocks", &self.server_address());
        let mut res = self
            .client
            .delete(request_url.as_str())
            .send()
            .expect("Mock server error");

        if res.status() != 202 {
            let mut buf = String::new();
            res.read_to_string(&mut buf)
                .expect("Failed to read response");
            let err_msg = format!(
                "Could not delete mocks from server (status = {}, message = {})",
                res.status(),
                buf
            );
            panic!(err_msg);
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
    pub fn as_str(&self) -> &'static str {
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
        server_port: 5000,
        server_host: "localhost".to_string(),
        client: reqwest::Client::new(),
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
