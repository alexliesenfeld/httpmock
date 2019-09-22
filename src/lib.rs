//! A simple-to-use HTTP mock server that can be used for local tests as
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
//! ```rust
//! extern crate mocha;
//!
//! use mocha::mock;
//! use mocha::Method::GET;
//!
//! #[test]
//! fn simple_test() {
//!     let m = mock(GET, "/health")
//!        .expect_header("User-Agent", "rust-test")
//!        .return_status(200)
//!        .return_header("X-Version", "0.0.1")
//!        .return_body("OK")
//!        .create();
//!
//!     let response = reqwest::Client::new()
//!        .get("http://localhost:5000/health")
//!         .header("User-Agent", "rust-test")
//!         .send()
//!         .unwrap();
//!
//!     assert_eq!(response.status(), 200);
//!     assert_eq!(m.times_called(), 1);
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
//!
//! If a server port is not provided using an environment variable (MOCHA_SERVER_PORT), the
//! internal mock server port will be set to 5000. If another server address is explicitely set
//! using an environment variable (MOCHA_SERVER_HOST), then this API will use the remote server
//! for creating and managing mocks.
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate typed_builder;

mod server;

pub use server::{start_server, HttpMockConfig};
use std::cell::RefCell;
use std::collections::BTreeMap;

use std::io::Read;

use crate::server::data::{
    ActiveMock, MockDefinition, MockIdentification, MockServerHttpResponse, Pattern,
    RequestRequirements,
};
use serde::Serialize;
use std::sync::{LockResult, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};

pub type Regex = regex::Regex;

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

    static ref CLIENT: Mutex<reqwest::Client> = {
        return Mutex::new(reqwest::Client::new());
    };
}

thread_local!(
    pub static SERVER_GUARD: RefCell<LockResult<MutexGuard<'static, JoinHandle<()>>>> =
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

    pub fn create_mock(&self, mock: &MockDefinition) -> Result<MockIdentification, String> {
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
        let response: serde_json::Result<MockIdentification> =
            serde_json::from_str(body_contents.as_str());
        if let Err(err) = response {
            return Err(format!("cannot deserialize mock server response: {}", err));
        }

        return Ok(response.unwrap());
    }

    pub fn fetch_mock(&self, mock_id: usize) -> Result<ActiveMock, String> {
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
        let response: serde_json::Result<ActiveMock> = serde_json::from_str(body_contents.as_str());
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

/// This struct presents the mocking interface to the user. It is the main struct to use in tests.
/// If there is no mock server running at the time when the mock is created, a mock server will be
/// created automatically in a background thread. The mock server will then be used for every
/// following test when needed by other mock objects and will be shut down at the end of the test
/// run. You can configure a mock like in the following example:
/// ```rust
/// extern crate mocha;
///
/// use mocha::mock;
/// use mocha::Method::GET;
///
/// #[test]
/// fn simple_test() {
///    let health_mock = mock(GET, "/health")
///       .return_status(200)
///       .create();
///
///    // Act (simulates your code)
///    let response = reqwest::get("http://localhost:5000/health").unwrap();
///
///    // Make some assertions
///    assert_eq!(response.status(), 200);
///    assert_eq!(health_mock.times_called().unwrap(), 1);
/// }
/// ```
/// Remember to call the [create](Mock::create) method when you are finished configuring the mock. This
/// will craete the mock object at the mock server and return you a mock object that represents
/// the mock. It will be used to fetch mock related information from the server or to delete the mock
/// once it gets out of scope. The mock will be deleted from the mock once it gets out of scope,
/// so you need to make sure to store the result of the [create](Mock::create) method.
/// The following will result in a mock that will be created and deleted from the server because it
/// got out of scope and hence instantly being removed from the server again:
/// ```rust
/// extern crate mocha;
///
/// use mocha::mock;
/// use mocha::Method::GET;
///
/// #[test]
/// fn simple_test() {
///    mock(GET, "/health") // WRONG! Created mock is not being held in a variable!
///       .return_status(200)
///       .create();
/// }
/// ```
/// Node that the [mock](mocha::mock) function is only there for convenience. If you want to have
/// more control over matching the path, you can use the [create](Mock::new) method like this:
/// ```rust
/// extern crate mocha;
///
/// use mocha::{Mock, Regex};
/// use mocha::Method::POST;
///
/// #[test]
/// fn simple_test() {
///     let m = Mock::new()
///     .expect_path("/test")
///     .expect_path_contains("test")
///     .expect_path_matches(Regex::new(r#"test"#).unwrap())
///     .expect_method(POST)
///     .return_status(200)
///     .create();
/// }
/// ```
/// If a server port is not provided using an environment variable (MOCHA_SERVER_PORT), the
/// internal mock server port will be set to 5000. If another server address is explicitely set
/// using an environment variable (MOCHA_SERVER_HOST), then this API will use the remote server
/// for creating and managing mocks.
///
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
            server_adapter: ServerAdapter::new(),
            mock: MockDefinition {
                request: RequestRequirements {
                    method: None,
                    path: None,
                    path_contains: None,
                    headers: None,
                    header_exists: None,
                    body: None,
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
        self.mock.request.body = Some(serialized_body);
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

    /// This method creates the mock at the server side and returns a Mock object representing
    /// the mock.
    ///
    /// This method will also block a test thread to prevent that multiple tests are being executed
    /// against the same mock server in parallel.
    pub fn create(mut self) -> Self {
        let response = self
            .server_adapter
            .create_mock(&self.mock)
            .expect("Cannot deserialize mock server response");

        self.id = Some(response.mock_id);
        self
    }

    /// This method returns the number of times a mock has been called at the mock server.
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
    /// not set otherwise by the environment variable MOCHA_SERVER_PORT.
    pub fn server_port(&self) -> u16 {
        self.server_adapter.server_port()
    }

    /// Returns the host of the mock server this mock is using. By default this is localhost if
    /// not set otherwise by the environment variable MOCHA_SERVER_HOST.
    pub fn server_host(&self) -> &str {
        self.server_adapter.server_host()
    }

    /// Returns the address of the mock server this mock is using. By default this is
    /// "localhost:5000" if not set otherwise by the environment variables  MOCHA_SERVER_HOST and
    /// MOCHA_SERVER_PORT.
    pub fn server_address(&self) -> String {
        self.server_adapter.server_address()
    }
}

impl Drop for Mock {
    /// Drops a mock and deletes it from the (remote) mock server.
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.server_adapter
                .delete_mock(id)
                .expect("could not delete mock from server");
        }
    }
}

/// The HTTP Method a mock should respond to.
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

// Enables enum to_string conversion
impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

/// This is a convenience method to create an HTTP mock. It automatically calls
/// [Mock::new](Mock::new) and already sets a path and an HTTP method for it.
/// Please refer to [Mock](Mock).
pub fn mock(method: Method, path: &str) -> Mock {
    Mock::new().expect_method(method).expect_path(path)
}
