//! An a library that allows you to mock HTTP responses in your tests.
//!
//! This crate contains two major lib:
//!
//! * a **mock server** that is automatically started in the background of your tests, and
//! * a **test library** to create HTTP mocks on the server.
//!
//! All interaction with the mock server happens through the provided library. Therefore, you do
//! not need to interact with the mock server directly (but you certainly can!).
//!
//! By default, an HTTP mock server instance will be started in the background of
//! your tests. It will be created when your tests need the mock server for the first
//! time and will be shut down at the end of the test run. The mock server is executed in a
//! separate thread, so it does not conflict with your tests.
//!
//! The mock server can also be started in **standalone mode** (more information below).
//!
//! # Getting Started
//! Add `httpmock` in your `Cargo.toml`:
//!
//! ```toml
//! [dev-dependencies]
//! httpmock = "0.3.2"
//! ```
//!
//! You can then use `httpmock` in your tests like shown in the following example:
//! ```rust
//! extern crate httpmock;
//!
//! use httpmock::Method::GET;
//! use httpmock::{mock, with_mock_server};
//!
//! #[test]
//! #[with_mock_server]
//! fn simple_test() {
//!    let search_mock = mock(GET, "/search")
//!        .expect_query_param("query", "metallica")
//!        .return_status(204)
//!        .create();
//!
//!    let response = reqwest::get("http://localhost:5000/search?query=metallica").unwrap();
//!
//!    assert_eq!(response.status(), 204);
//!    assert_eq!(search_mock.times_called(), 1);
//! }
//! ```
//! In the above example, a mock server is automatically created when the test launches.
//! This is ensured by the [with_mock_server](../httpmock_macros/attr.with_mock_server.html)
//! annotation. It wraps the test with an initializer function that is performing several important
//! preparation steps, such as starting the mock server if none yet exists
//! and cleaning up the mock server state, so that each test can start with
//! a clean mock server. The annotation also sequentializes tests that are marked with it, so
//! they do not conflict with each other when using the mock server.
//!
//! If you try to create a mock without having annotated your test function
//! with the [with_mock_server](../httpmock_macros/attr.with_mock_server.html) annotation,
//! you will receive a panic at runtime pointing you to this problem.
//!
//! # Usage
//! The main point of interaction with the mock server happens via [Mock](struct.Mock.html).
//! It provides you all mocking functionality that is supported by the mock server.
//!
//! The expected style of usage is to
//! * create a [Mock](struct.Mock.html) object using the
//! [Mock::create](struct.Mock.html#method.create) method
//! (or [Mock::new](struct.Mock.html#method.new) for slightly more control)
//! * Set all your mock requirements using `expect_xxx`-methods, such as headers, body content, etc.
//! These methods describe what attributes an HTTP request needs to have to be considered a
//! "match" for the mock you are creating.
//! * use `return_xxx`-methods to describe what the mock server should return when it receives
//! an HTTP request that matches the mock. If the server does not find any matching mocks for an
//! HTTP request, it will return a response with an empty body and an HTTP status code 500.
//! * create the mock using the [Mock::create](struct.Mock.html#method.create) method. If you do
//! not call this method when you complete configuring it, it will not be created at the mock
//! server and your test will not receive the expected response.
//! * using the mock object returned by by the [Mock::create](struct.Mock.html#method.create) method
//! to assert that a mock has been called by your code under test (please refer to any example).
//!
//! # Responses
//! An HTTP request made by your application is only considered to match a mock if the request
//! fulfills all specified mock requirements. If a request does not match any mock currently stored
//! on the mock server, it will respond with an empty response body and an HTTP status code 500
//! (Internal Server Error).
//!
//! # Examples
//! Fore more examples, please refer to
//! [this crates test directory](https://github.com/alexliesenfeld/httpmock/blob/master/tests/integration_tests.rs ).
//!
//! # Debugging
//! `httpmock` logs against the `log` crate. If you use the `env_logger` backend, you can activate
//! debug logging by setting `RUST_LOG` environment variable to `debug` and then calling
//! `env_logger::try_init()`:
//! ```rust
//! #[test]
//! #[with_mock_server]
//! fn your_test() {
//!     let _ = env_logger::try_init();
//!     // ...
//! }
//! ```
//! # Standalone Mode
//! You can use this crate to provide both, an HTTP mock server for your local tests,
//! but also a standalone mock server that is reachable for other applications as well. This can be
//! useful if you are running integration tests that span multiple applications.
//!
//! To activate standalone mode, you need to do the following steps:
//! * Start the mock server in standalone mode by running
//! `cargo run --features="standalone" --release` from the sources
//! (or by using a binary that you can build with `cargo build --features="standalone" --release`).
//! * On the host that is executing the tests, provide a host name by setting the environment variable
//! `HTTPMOCK_HOST`. If set, tests are assuming a mock server is being executed elsewhere,
//! so no local mock server will be started for your tests anymore. Instead, this library will be using
//! the remote server to create mocks.
//!
//! By default, if a server port is not provided by the environment variable
//! `HTTPMOCK_PORT`, port `5000` will be used.
//!
//! ## Exposing the mock server to the network
//! If you want to expose the server to machines other than localhost, you need to provide the
//! `--expose` parameter:
//! * using cargo: `cargo run --features="standalone" --release -- --expose`
//! * using the binary: `httpmock --expose`
//!
//! ## Docker container
//! As an alternative to building the mock server yourself, you can use the Docker image from
//! the sources to run a mock server in standalone mode:
//! ```shell
//! docker build -t httpmock .
//! docker run -it --rm -p 5000:5000 --name httpmock httpmock
//! ```
//! To enable extended logging, you can run the docker container with the `RUST_LOG` environment
//! variable set to the log level of your choice:
//! ```shell
//! docker run -it --rm -e RUST_LOG=httpmock=debug -p 5000:5000 --name httpmock httpmock
//! ```
//! Please refer to the [log](../log/index.html) and [env_logger](../env_logger/index.html) crates
//! for more information about logging.

#[macro_use]
extern crate lazy_static;

mod server;
#[doc(hidden)]
pub mod util;

pub use httpmock_macros::with_mock_server;
pub use server::{start_server, HttpMockConfig};

use std::collections::BTreeMap;

use crate::server::data::{
    ActiveMock, MockDefinition, MockIdentification, MockServerHttpResponse, Pattern,
    RequestRequirements,
};
use serde::Serialize;
use serde_json::Value;
use std::cell::RefCell;
use std::str::FromStr;
use std::sync::{Mutex, MutexGuard};
use std::thread;

use futures::future;
use futures::{Future, Stream};

use hyper::client::connect::dns::GaiResolver;
use hyper::client::HttpConnector;
use hyper::Request;
use hyper::{Body, Client, Error, Method as HyperMethod, StatusCode};
use std::net::TcpListener;

/// Refer to [regex::Regex](../regex/struct.Regex.html).
pub type Regex = regex::Regex;

/// Waits until a given port is available on a given address or panics if this takes too long.
fn wait_until_port_available(addr: &str, port: u16) {
    let result = util::with_retry(100, 100, || TcpListener::bind((addr, port)));

    if result.is_err() {
        panic!(format!(
            "Cannot create mock server (port {} seems busy)",
            port
        ))
    }
}

lazy_static! {
    static ref SERVER_MUTEX: Mutex<ServerAdapter> = {
        let server = ServerAdapter::from_env();

        // Start local server if necessary
        if !server.is_remote {
            wait_until_port_available(&server.host, server.port);

            let port = server.port;
            thread::spawn(move || {
                let number_of_workers : usize = 3;
                let expose_to_network = false;
                let config = HttpMockConfig::new(port, number_of_workers, expose_to_network);
                start_server(config);
            });
        }

        return Mutex::new(server);
    };
}

thread_local!(
    static TEST_INITIALIZED: RefCell<bool> = RefCell::new(false);

    static TOKIO_RUNTIME: RefCell<tokio::runtime::current_thread::Runtime> = {
        let runtime = tokio::runtime::current_thread::Runtime::new()
            .expect("Cannot build thread local tokio tuntime");
        RefCell::new(runtime)
    };

    static HYPER_CLIENT: Client<HttpConnector<GaiResolver>, Body> =
        { hyper::client::Client::new() };
);

/// For internal use only. Do not use it.
#[doc(hidden)]
pub fn internal_thread_local_test_init_status(status: bool) {
    TEST_INITIALIZED.with(|is_init| *is_init.borrow_mut() = status);
}

/// For internal use only. Do not use it.
#[doc(hidden)]
pub fn internal_server_management_lock() -> MutexGuard<'static, ServerAdapter> {
    return match SERVER_MUTEX.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
}

/// This adapter allows to access the servers management functionality.
///
/// You can create an adapter by calling `ServerAdapter::from_env` to create a new instance.
/// You should never actually need to use this adapter, but you certainly can, if you absolutely
/// need to.
#[derive(Debug)]
pub struct ServerAdapter {
    is_remote: bool,
    host: String,
    port: u16,
}
impl ServerAdapter {
    pub fn from_env() -> ServerAdapter {
        let host = option_env!("HTTPMOCK_HOST");
        let port = option_env!("HTTPMOCK_PORT");

        ServerAdapter {
            is_remote: host.is_some(),
            host: match host {
                None => "localhost".to_string(),
                Some(h) => h.to_string(),
            },
            port: match port {
                None => 5000 as u16,
                Some(port_string) => port_string.parse::<u16>().expect(&format!(
                    "Cannot parse port from environment variable value '{}'",
                    port_string
                )),
            },
        }
    }

    pub fn server_port(&self) -> u16 {
        self.port
    }

    pub fn server_host(&self) -> &str {
        &self.host
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_host(), self.server_port())
    }

    pub fn create_mock(&self, mock: &MockDefinition) -> Result<MockIdentification, String> {
        // Serialize to JSON
        let json = serde_json::to_string(mock);
        if let Err(err) = json {
            return Err(format!("cannot serialize mock object to JSON: {}", err));
        }
        let json = json.unwrap();

        // Send the request to the mock server
        let request_url = format!("http://{}/__mocks", &self.server_address());

        let request = Request::builder()
            .method(HyperMethod::POST)
            .uri(request_url)
            .header("Content-Type", "application/json")
            .body(Body::from(json))
            .expect("Cannot build request");

        let response = execute_request(request);
        if let Err(err) = response {
            return Err(format!("cannot send request to mock server: {}", err));
        }

        let (status, body) = response.unwrap();

        // Evaluate the response status
        if status != 201 {
            return Err(format!(
                "could not create mock. Mock server response: status = {}, message = {}",
                status, body
            ));
        }

        // Create response object
        let response: serde_json::Result<MockIdentification> = serde_json::from_str(&body);
        if let Err(err) = response {
            return Err(format!("cannot deserialize mock server response: {}", err));
        }

        return Ok(response.unwrap());
    }

    pub fn fetch_mock(&self, mock_id: usize) -> Result<ActiveMock, String> {
        // Send the request to the mock server
        let request_url = format!("http://{}/__mocks/{}", &self.server_address(), mock_id);
        let request = Request::builder()
            .method(HyperMethod::GET)
            .uri(request_url)
            .body(Body::empty())
            .expect("Cannot build request");

        let response = execute_request(request);
        if let Err(err) = response {
            return Err(format!("cannot send request to mock server: {}", err));
        }

        let (status, body) = response.unwrap();

        // Evaluate response status code
        if status != 200 {
            return Err(format!(
                "could not create mock. Mock server response: status = {}, message = {}",
                status, body
            ));
        }

        // Create response object
        let response: serde_json::Result<ActiveMock> = serde_json::from_str(&body);
        if let Err(err) = response {
            return Err(format!("cannot deserialize mock server response: {}", err));
        }

        return Ok(response.unwrap());
    }

    pub fn delete_mock(&self, mock_id: usize) -> Result<(), String> {
        // Send the request to the mock server
        let request_url = format!("http://{}/__mocks/{}", &self.server_address(), mock_id);
        let request = Request::builder()
            .method(HyperMethod::DELETE)
            .uri(request_url)
            .body(Body::empty())
            .expect("Cannot build request");

        let response = execute_request(request);
        if let Err(err) = response {
            return Err(format!("cannot send request to mock server: {}", err));
        }
        let (status, body) = response.unwrap();

        // Evaluate response status code
        if status != 202 {
            return Err(format!(
                "Could not delete mocks from server (status = {}, message = {})",
                status, body
            ));
        }

        return Ok(());
    }

    pub fn delete_all_mocks(&self) -> Result<(), String> {
        // Send the request to the mock server
        let request_url = format!("http://{}/__mocks", &self.server_address());
        let request = Request::builder()
            .method(HyperMethod::DELETE)
            .uri(request_url)
            .body(Body::empty())
            .expect("Cannot build request");

        let response = execute_request(request);
        if let Err(err) = response {
            return Err(format!("cannot send request to mock server: {}", err));
        }

        let (status, body) = response.unwrap();

        // Evaluate response status code
        if status != 202 {
            return Err(format!(
                "Could not delete mocks from server (status = {}, message = {})",
                status, body
            ));
        }

        return Ok(());
    }
}

/// Represents the primary interface to the mock server.
///
/// # Example
/// ```rust
/// extern crate httpmock;
///
/// use httpmock::{mock, with_mock_server};
/// use httpmock::Method::GET;
///
/// #[test]
/// #[with_mock_server]
/// fn simple_test() {
///    let search_mock = mock(GET, "/health")
///       .return_status(200)
///       .create();
///
///    // Act (simulates your code)
///    let response = reqwest::get("http://localhost:5000/health").unwrap();
///
///    // Make some assertions
///    assert_eq!(response.status(), 200);
///    assert_eq!(search_mock.times_called().unwrap(), 1);
/// }
/// ```
/// To be able to create a mock, you need to mark your test function with the
/// [httpmock::with_mock_server](../httpmock/attr.with_mock_server.html) attribute. If you try to
/// create a mock by calling [Mock::create](struct.Mock.html#method.create) without marking your
/// test function with [httpmock::with_mock_server](../httpmock/attr.with_mock_server.html),
/// you will receive a panic during runtime telling you about this fact.
///
/// Note that you need to call the [Mock::create](struct.Mock.html#method.create) method once you
/// are finished configuring your mock. This will create the mock on the server. Thereafter, the
/// mock will be served whenever clients send HTTP requests that match all requirements of your mock.
///
/// The [Mock::create](struct.Mock.html#method.create) method returns a reference that
/// identifies the mock at the server side. The reference can be used to fetch
/// mock related information from the server, such as the number of times it has been called or to
/// explicitly delete the mock from the server.
///
/// While [httpmock::mock](struct.Mock.html#method.create) is a convenience function, you can
/// have more control over matching the path by directly creating a new [Mock](struct.Mock.html)
/// object yourself using the [Mock::new](struct.Mock.html#method.new) method.
/// # Example
/// ```rust
/// extern crate httpmock;
///
/// use httpmock::Method::POST;
/// use httpmock::{Mock, Regex, with_mock_server};
///
/// #[test]
/// #[with_mock_server]
/// fn simple_test() {
///     Mock::new()
///       .expect_path("/test")
///       .expect_path_contains("test")
///       .expect_path_matches(Regex::new(r#"test"#).unwrap())
///       .expect_method(POST)
///       .return_status(200)
///       .create();
/// }
/// ```
/// Fore more examples, please refer to
/// [this crates test directory](https://github.com/alexliesenfeld/httpmock/blob/master/tests/integration_tests.rs ).
#[derive(Debug)]
pub struct Mock {
    mock: MockDefinition,
    server_adapter: ServerAdapter,
    id: Option<usize>,
}

impl Mock {
    /// Creates a new mock that automatically returns HTTP status code 200 if hit by an HTTP call.
    pub fn new() -> Mock {
        Mock {
            id: None,
            server_adapter: ServerAdapter::from_env(),
            mock: MockDefinition {
                request: RequestRequirements {
                    method: None,
                    path: None,
                    path_contains: None,
                    headers: None,
                    header_exists: None,
                    body: None,
                    json_body: None,
                    json_body_includes: None,
                    body_contains: None,
                    path_matches: None,
                    body_matches: None,
                    query_param_exists: None,
                    query_param: None,
                },
                response: MockServerHttpResponse {
                    status: 200,
                    headers: None,
                    body: None,
                },
            },
        }
    }

    /// Sets the expected path. If the path of an HTTP request at the server is equal to the
    /// provided path, the request will be considered a match for this mock to respond (given all
    /// other criteria are met).
    /// * `path` - The exact path to match against.
    pub fn expect_path(mut self, path: &str) -> Self {
        self.mock.request.path = Some(path.to_string());
        self
    }

    /// Sets an expected path substring. If the path of an HTTP request at the server contains t,
    /// his substring the request will be considered a match for this mock to respond (given all
    /// other criteria are met).
    /// * `substring` - The substring to match against.
    pub fn expect_path_contains(mut self, substring: &str) -> Self {
        if self.mock.request.path_contains.is_none() {
            self.mock.request.path_contains = Some(Vec::new());
        }

        self.mock
            .request
            .path_contains
            .as_mut()
            .unwrap()
            .push(substring.to_string());

        self
    }

    /// Sets an expected path regex. If the path of an HTTP request at the server matches this,
    /// regex the request will be considered a match for this mock to respond (given all other
    /// criteria are met).
    /// * `regex` - The regex to match against.
    pub fn expect_path_matches(mut self, regex: Regex) -> Self {
        if self.mock.request.path_matches.is_none() {
            self.mock.request.path_matches = Some(Vec::new());
        }

        self.mock
            .request
            .path_matches
            .as_mut()
            .unwrap()
            .push(Pattern::from_regex(regex));
        self
    }

    /// Sets the expected HTTP method. If the path of an HTTP request at the server matches this regex,
    /// the request will be considered a match for this mock to respond (given all other
    /// criteria are met).
    /// * `method` - The HTTP method to match against.
    pub fn expect_method(mut self, method: Method) -> Self {
        self.mock.request.method = Some(method.to_string());
        self
    }

    /// Sets an expected HTTP header. If one of the headers of an HTTP request at the server matches
    /// the provided header key and value, the request will be considered a match for this mock to
    /// respond (given all other criteria are met).
    ///
    /// * `name` - The HTTP header name (header names are case-insensitive by RFC 2616).
    /// * `value` - The HTTP header value.
    pub fn expect_header(mut self, name: &str, value: &str) -> Self {
        if self.mock.request.headers.is_none() {
            self.mock.request.headers = Some(BTreeMap::new());
        }

        self.mock
            .request
            .headers
            .as_mut()
            .unwrap()
            .insert(name.to_string(), value.to_string());

        self
    }

    /// Sets an expected HTTP header to exists. If one of the headers of an HTTP request at the
    /// server matches the provided header name, the request will be considered a match for this
    /// mock to respond (given all other criteria are met).
    ///
    /// * `name` - The HTTP header name (header names are case-insensitive by RFC 2616).
    pub fn expect_header_exists(mut self, name: &str) -> Self {
        if self.mock.request.header_exists.is_none() {
            self.mock.request.header_exists = Some(Vec::new());
        }

        self.mock
            .request
            .header_exists
            .as_mut()
            .unwrap()
            .push(name.to_string());
        self
    }

    /// Sets the expected HTTP body. If the body of an HTTP request at the server matches the
    /// provided body, the request will be considered a match for this mock to respond
    /// (given all other criteria are met). This is an exact match, so all characters are taken
    /// into account, such as whitespace, tabs, etc.
    ///  * `contents` - The HTTP body to match against.
    pub fn expect_body(mut self, contents: &str) -> Self {
        self.mock.request.body = Some(contents.to_string());
        self
    }

    /// Sets the expected HTTP body JSON string. This method expects a serializable serde object
    /// that will be parsed into JSON. If the body of an HTTP request at the server matches the
    /// body according to the provided JSON object, the request will be considered a match for
    /// this mock to respond (given all other criteria are met).
    ///
    /// This is an exact match, so all characters are taken into account at the server side.
    ///
    /// The provided JSON object needs to be both, a deserializable and
    /// serializable serde object. Note that this method does not set the "Content-Type" header
    /// automatically, so you need to provide one yourself!
    ///
    /// * `body` - The HTTP body object that will be serialized to JSON using serde.
    pub fn expect_json_body<T>(mut self, body: &T) -> Self
    where
        T: Serialize,
    {
        let serialized_body =
            serde_json::to_string(body).expect("cannot serialize json body to JSON string ");

        let value =
            Value::from_str(&serialized_body).expect("cannot convert JSON string to serde value");

        self.mock.request.json_body = Some(value);
        self
    }

    /// Sets an expected partial HTTP body JSON string.
    ///
    /// If the body of an HTTP request at the server matches the
    /// partial, the request will be considered a match for
    /// this mock to respond (given all other criteria are met).
    ///
    /// # Important Notice
    /// The partial string needs to contain the full JSON object path from the root.
    ///
    /// ## Example
    /// If your application sends the following JSON request data to the mock server
    /// ```json
    /// {
    ///     "parent_attribute" : "Some parent data goes here",
    ///     "child" : {
    ///         "target_attribute" : "Target value",
    ///         "other_attribute" : "Another value"
    ///     }
    /// }
    /// ```
    /// and you only want to make sure that `target_attribute` has the value
    /// `Target value`, you need to provide a partial JSON string to this method, that starts from
    /// the root of the JSON object, but may leave out unimportant values:
    /// ```
    /// use httpmock::Method::POST;
    /// use httpmock::mock;
    ///
    /// #[test]
    /// #[with_mock_server]
    /// fn partial_json_test() {
    ///     mock(POST, "/path")
    ///         .expect_json_body_partial(r#"
    ///             {
    ///                 "child" : {
    ///                     "target_attribute" : "Target value"
    ///                 }
    ///             }
    ///         "#)
    ///         .return_status(200)
    ///         .create();
    /// }
    ///
    /// ```
    /// String format and attribute order will be ignored.
    ///
    /// * `partial` - The JSON partial.
    pub fn expect_json_body_partial(mut self, partial: &str) -> Self {
        if self.mock.request.json_body_includes.is_none() {
            self.mock.request.json_body_includes = Some(Vec::new());
        }

        let value = Value::from_str(partial).expect("cannot convert JSON string to serde value");

        self.mock
            .request
            .json_body_includes
            .as_mut()
            .unwrap()
            .push(value);
        self
    }

    /// Sets an expected HTTP body substring. If the body of an HTTP request at the server contains
    /// the provided substring, the request will be considered a match for this mock to respond
    /// (given all other criteria are met).
    /// * `substring` - The substring that will matched against.
    pub fn expect_body_contains(mut self, substring: &str) -> Self {
        if self.mock.request.body_contains.is_none() {
            self.mock.request.body_contains = Some(Vec::new());
        }

        self.mock
            .request
            .body_contains
            .as_mut()
            .unwrap()
            .push(substring.to_string());
        self
    }

    /// Sets an expected HTTP body regex. If the body of an HTTP request at the server matches
    /// the provided regex, the request will be considered a match for this mock to respond
    /// (given all other criteria are met).
    /// * `regex` - The regex that will matched against.
    pub fn expect_body_matches(mut self, regex: Regex) -> Self {
        if self.mock.request.body_matches.is_none() {
            self.mock.request.body_matches = Some(Vec::new());
        }

        self.mock
            .request
            .body_matches
            .as_mut()
            .unwrap()
            .push(Pattern::from_regex(regex));
        self
    }

    /// Sets an expected query parameter. If the query parameters of an HTTP request at the server
    /// contains the provided query parameter name and value, the request will be considered a
    /// match for this mock to respond (given all other criteria are met).
    /// * `name` - The query parameter name that will matched against.
    /// * `value` - The value parameter name that will matched against.
    pub fn expect_query_param(mut self, name: &str, value: &str) -> Self {
        if self.mock.request.query_param.is_none() {
            self.mock.request.query_param = Some(BTreeMap::new());
        }

        self.mock
            .request
            .query_param
            .as_mut()
            .unwrap()
            .insert(name.to_string(), value.to_string());

        self
    }

    /// Sets an expected query parameter name. If the query parameters of an HTTP request at the server
    /// contains the provided query parameter name (not considering the value), the request will be
    /// considered a match for this mock to respond (given all other criteria are met).
    /// * `name` - The query parameter name that will matched against.
    pub fn expect_query_param_exists(mut self, name: &str) -> Self {
        if self.mock.request.query_param_exists.is_none() {
            self.mock.request.query_param_exists = Some(Vec::new());
        }

        self.mock
            .request
            .query_param_exists
            .as_mut()
            .unwrap()
            .push(name.to_string());

        self
    }

    /// Sets the HTTP status that the mock will return, if an HTTP request fulfills all of
    /// the mocks requirements.
    /// * `status` - The HTTP status that the mock server will return.
    pub fn return_status(mut self, status: usize) -> Self {
        self.mock.response.status = status as u16;
        self
    }

    /// Sets the HTTP response body that the mock will return, if an HTTP request fulfills all of
    /// the mocks requirements.
    /// * `body` - The HTTP response body that the mock server will return.
    pub fn return_body(mut self, body: &str) -> Self {
        self.mock.response.body = Some(body.to_string());
        self
    }

    /// Sets the HTTP response JSON body that the mock will return, if an HTTP request fulfills all of
    /// the mocks requirements.
    ///
    /// The provided JSON object needs to be both, a deserializable and
    /// serializable serde object. Note that this method does not set the "Content-Type" header
    /// automatically, so you need to provide one yourself!
    ///
    /// * `body` - The HTTP response body the mock server will return in the form of a JSON string.
    pub fn return_json_body<T>(mut self, body: &T) -> Self
    where
        T: Serialize,
    {
        let serialized_body =
            serde_json::to_string(body).expect("cannot serialize json body to JSON string ");
        self.mock.response.body = Some(serialized_body);
        self
    }

    /// Sets an HTTP header that the mock will return, if an HTTP request fulfills all of
    /// the mocks requirements.
    /// * `name` - The name of the header.
    /// * `value` - The value of the header.
    pub fn return_header(mut self, name: &str, value: &str) -> Self {
        if self.mock.response.headers.is_none() {
            self.mock.response.headers = Some(BTreeMap::new());
        }

        self.mock
            .response
            .headers
            .as_mut()
            .unwrap()
            .insert(name.to_string(), value.to_string());

        self
    }

    /// This method creates the mock at the server side and returns a `Mock` object
    /// representing the reference of the created mock at the server.
    ///
    /// # Panics
    /// This method will panic if your test method was not marked using the the
    /// `httpmock::with_mock_server` annotation.
    pub fn create(mut self) -> Self {
        if !TEST_INITIALIZED.with(|is_init| *is_init.borrow()) {
            panic!("Mocking framework is not initialized (did you mark your test method with the #[with_mock_server] attribute?)")
        }

        let response = self
            .server_adapter
            .create_mock(&self.mock)
            .expect("Cannot deserialize mock server response");

        self.id = Some(response.mock_id);
        self
    }

    /// This method returns the number of times a mock has been called at the mock server.
    ///
    /// # Panics
    /// This method will panic if there is a problem to communicate with the server.
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

    /// Returns the port of the mock server this mock is using. By default this is port 5000 if
    /// not set otherwise by the environment variable HTTPMOCK_PORT.
    pub fn server_port(&self) -> u16 {
        self.server_adapter.server_port()
    }

    /// Returns the host of the mock server this mock is using. By default this is localhost if
    /// not set otherwise by the environment variable HTTPMOCK_HOST.
    pub fn server_host(&self) -> &str {
        self.server_adapter.server_host()
    }

    /// Returns the address of the mock server this mock is using. By default this is
    /// "localhost:5000" if not set otherwise by the environment variables  HTTPMOCK_HOST and
    /// HTTPMOCK_PORT.
    pub fn server_address(&self) -> String {
        self.server_adapter.server_address()
    }

    /// Deletes this mock from the mock server.
    ///
    /// # Panics
    /// This method will panic if there is a problem to communicate with the server.
    pub fn delete(&mut self) {
        if let Some(id) = self.id {
            self.server_adapter
                .delete_mock(id)
                .expect("could not delete mock from server");
        } else {
            panic!("Cannot delete mock, because it has not been created at the server yet.");
        }
    }
}

/// Represents an HTTP method.
#[derive(Debug)]
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

/// Enables enum to_string conversion
impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

/// A convenience function to create an HTTP mock. It automatically calls
/// [Mock::new](struct.Mock.html#method.new) and already sets a path and an HTTP method.
/// Please refer to [Mock](struct.Mock.html) struct for a more detailed description.
pub fn mock(method: Method, path: &str) -> Mock {
    Mock::new().expect_method(method).expect_path(path)
}

/// Executes an HTTP request synchronously
fn execute_request(req: Request<Body>) -> Result<(StatusCode, String), Error> {
    HYPER_CLIENT.with(move |client| {
        let fut = client.request(req).and_then(|res| {
            let status = res.status();

            res.into_body()
                .fold(Vec::new(), |mut v, chunk| {
                    v.extend(&chunk[..]);
                    future::ok::<_, Error>(v)
                })
                .and_then(move |chunks| {
                    let s = String::from_utf8_lossy(&chunks).to_string();
                    future::ok::<_, Error>((status, s))
                })
        });
        TOKIO_RUNTIME.with(|runtime| (*runtime.borrow_mut()).block_on(fut))
    })
}
