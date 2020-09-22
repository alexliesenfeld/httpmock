//! `httpmock` is a Rust library that allows you to mock HTTP services in your tests.
//!
//!  # Features
//!
//! * Provides an HTTP mock server with HTTP/1 and HTTP/2 support.
//! * A fully asynchronous core with synchronous and asynchronous APIs.
//! * Compatible with all major asynchronous executors and runtimes.
//! * Built-in request matchers with support for custom request matchers.
//! * Parallel test execution by default.
//! * A standalone mode with an accompanying [Docker image](https://hub.docker.com/r/alexliesenfeld/httpmock).

//! # Getting Started
//! Add `httpmock` to `Cargo.toml`:
//!
//! ```toml
//! [dev-dependencies]
//! httpmock = "0.5.0"
//! ```
//!
//! You can then use `httpmock` in your tests like shown in the example below:
//! ```rust
//! extern crate httpmock;
//!
//! use httpmock::Method::{GET};
//! use httpmock::{Mock, MockServer, MockServerRequest, Regex};
//! use isahc::{get};
//!
//! #[test]
//! fn example_test() {
//!     // Start a local mock server for exclusive use by this test function.
//!     let mock_server = MockServer::start();
//!
//!     // Create a mock on the mock server. The mock will return HTTP status code 200 whenever
//!     // the mock server receives a GET-request with path "/hello".
//!     let search_mock = Mock::new()
//!         .expect_method(GET)
//!         .expect_path("/hello")
//!         .return_status(200)
//!         .create_on(&mock_server);
//!
//!     // Send an HTTP request to the mock server. This simulates your code.
//!     // The mock_server variable is being used to generate a mock server URL for path "/hello".
//!     let response = get(mock_server.url("/hello")).unwrap();
//!
//!     // Ensure the mock server did respond as specified above.
//!     assert_eq!(response.status(), 200);
//!     // Ensure the specified mock responded exactly one time.
//!     assert_eq!(search_mock.times_called(), 1);
//! }
//! ```
//!
//! # API Usage
//!
//! Each test usually creates its own local [MockServer](struct.MockServer.html) using
//! [MockServer::start](struct.MockServer.html#method.start). This creates a lightweight HTTP
//! server that runs on its own port. This way tests do not conflict with each other.
//!
//! You can use the [Mock](struct.Mock.html) structure to specify and create mocks on the
//! mock server. It provides you all supported mocking functionality.
//!
//! ## Request Matching and Responses
//! Other than many other libraries `httpmock` does not require you to learn a DSL-like API to
//! specify mock behaviour. Instead, `httpmock` provides you a fluent builder API that
//! clearly separates request matching and response attributes by using the following naming scheme:
//!
//! - All [Mock](struct.Mock.html) methods that start with `expect` in their name set a requirement
//! for HTTP requests (e.g. [Mock::expect_method](struct.Mock.html#method.expect_method),
//! [Mock::expect_path](struct.Mock.html#method.expect_path), or
//! [Mock::expect_body](struct.Mock.html#method.expect_body)).
//! - All [Mock](struct.Mock.html) methods that start with `return` in their name define what the
//! mock server will return in response to an HTTP request that matched all mock requirements (e.g.
//! [Mock::return_status](struct.Mock.html#method.return_status),
//! [Mock::return_body](struct.Mock.html#method.return_body), etc.).
//!
//! With this naming scheme users can benefit from IDE autocompletion to find request matchers and
//! response attributes mostly without even looking into documentation.
//!
//! If a request does not match at least one mock, the server will respond with
//! an error message and HTTP status code 404 (Not Found).
//!
//! ## Sync / Async
//!
//! The internal implementation of `httpmock` is fully asynchronous. It provides you a synchronous
//! and an asynchronous API though. If you want to schedule awaiting operations manually, then
//! you can use the `async` variants that exist for every potentially blocking operation. For
//! example, there is [MockServer::start_async](struct.MockServer.html#method.start_async) as an
//! asynchronous counterpart to [MockServer::start](struct.MockServer.html#method.start) and
//! [Mock::create_on_async](struct.Mock.html#method.create_on_async) for
//! [Mock::create_on](struct.Mock.html#method.create_on).
//!
//! # Parallelism
//! To balance execution speed and resource consumption, `MockServer`s are kept in a server pool
//! internally. This allows to run multiple tests in parallel without overwhelming the executing
//! machine by creating too many HTTP servers. A test will be blocked if it tries to use a
//! `MockServer` (e.g. by calling `MockServer::new()`) while the server pool is empty (i.e. all
//! servers are occupied by other tests). To avoid TCP port binding issues, `MockServers` are
//! never recreated but recycled/resetted. The pool is filled on demand up to a predefined
//! maximum number of 25 servers. You can change this number by setting the environment
//! variable `HTTPMOCK_MAX_SERVERS`.
//!
//! # Examples
//! Fore more examples, please refer to
//! [this crates test directory](https://github.com/alexliesenfeld/httpmock/blob/master/tests/integration_tests.rs ).
//!
//! # Debugging
//! `httpmock` logs against the `log` crate. This allows you to see detailed information about
//! `httpmock`s behaviour. For example, if you use the `env_logger` backend, you can activate debug
//! logging by setting the `RUST_LOG` environment variable to `httpmock=debug`.
//!
//! Attention: To be able to see the log output, you need to add the `--nocapture` argument
//! when starting test execution!
//!
//! # Standalone Mode
//! You can use `httpmock` to run a standalone mock server that is available to multiple applications.
//! This can be useful if you are running integration tests that involve both, real and mocked
//! applications.
//!
//! Although you can build the mock server in standalone mode yourself, it is easiest to use the
//! accompanying [Docker image](https://hub.docker.com/r/alexliesenfeld/httpmock).
//!
//! ## API Usage
//! To be able to use a standalone server from your tests, you need to change how an instance
//! of the `MockServer` structure is created. Instead of using `MockServer::new()`, you need
//! to connect to a remote server by using one of the `connect` methods (such as
//! `MockServer::connect("localhost:5000")` or `MockServer::connect_from_env()`).
//! Therefore, tests that use a local mock server do only differ in one line of code
//! from tests that use a remote server. Otherwise, both variants are identical.
//!
//! ```rust
//! use httpmock::{MockServer, Mock};
//! use isahc::get;
//!
//! #[test]
//! fn simple_test() {
//!     // Arrange: Create a mock on a test local mock server
//!     let mock_server = MockServer::connect("some-host:5000");
//!
//!     let hello_mock = Mock::new()
//!         .expect_method(GET)
//!         .expect_path("/hello")
//!         .return_status(200)
//!         .create_on(&mock_server);
//!
//!     // Act: Send an HTTP request to the mock server (simulates your software)
//!     let response = get(mock_server.url("/hello")).unwrap();
//!
//!     // Assert: Ensure there was a response from the mock server
//!     assert_eq!(response.status(), 200);
//!     assert_eq!(hello_mock.times_called(), 1);
//! }
//! ```
//!
//! ## Parallelism
//! Tests that use a remote mock server are executed sequentially by default. This is in
//! contrast to tests that use a local mock server. Sequential execution is achieved by
//! blocking all tests from further execution whenever a test requires to connect to a
//! busy mock server.
//!
//! ## Limitations
//! At this time, it is not possible to use custom request matchers in combination with remote
//! mock servers. It is planned to add this functionality in future though.
//!
//! ## Examples
//! Fore more examples on how to use a remote server, please refer to
//! [this crates test directory](https://github.com/alexliesenfeld/httpmock/blob/master/tests/standalone_tests.rs ).
//!
//! # License
//! `httpmock` is free software: you can redistribute it and/or modify it under the terms
//! of the MIT Public License.
//!
//! This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
//! without even the implied
//! warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the MIT Public
//! License for more details.
#[macro_use]
extern crate lazy_static;

use std::net::{SocketAddr, ToSocketAddrs};
use std::rc::Rc;
use std::sync::Arc;
use std::thread;

use puddle::Pool;
use tokio::task::LocalSet;

use util::Join;

use crate::api::{LocalMockServerAdapter, MockServerAdapter, RemoteMockServerAdapter};
pub use crate::api::{Method, Mock, MockRef, Regex};
use crate::server::data::{MockMatcherFunction, MockServerHttpRequest};
pub use crate::server::HttpMockConfig;
use crate::server::{start_server, MockServerState};
use crate::util::{read_env, with_retry};
use futures_util::core_reexport::time::Duration;
use serde::Serialize;
use serde_json::Value;
use std::cell::Cell;

mod api;
mod server;
mod util;

pub mod standalone {
    use std::sync::Arc;

    use crate::server::HttpMockConfig;
    use crate::server::{start_server, MockServerState};

    pub async fn start_standalone_server(config: HttpMockConfig) -> Result<(), String> {
        let state = Arc::new(MockServerState::new());
        start_server(config, &state, None).await
    }
}

pub type MockServerRequest = Rc<MockServerHttpRequest>;

/// A mock server that is able to receive and respond to HTTP requests.
pub struct MockServer {
    pub(crate) server_adapter: Option<Arc<dyn MockServerAdapter + Send + Sync>>,
    pool: Arc<Pool<Arc<dyn MockServerAdapter + Send + Sync>>>,
}

impl MockServer {
    async fn from(
        server_adapter: Arc<dyn MockServerAdapter + Send + Sync>,
        pool: Arc<Pool<Arc<dyn MockServerAdapter + Send + Sync>>>,
    ) -> Self {
        with_retry(5, || server_adapter.ping())
            .await
            .expect("Cannot ping mock server.");
        with_retry(5, || server_adapter.delete_all_mocks())
            .await
            .expect("Cannot reset mock server.");
        Self {
            server_adapter: Some(server_adapter),
            pool,
        }
    }

    /// Asynchronously connects to a remote mock server that is running in standalone mode using
    /// the provided address of the form <host>:<port> (e.g. "127.0.0.1:8080") to establish
    /// the connection.
    pub async fn connect_async(address: &str) -> Self {
        let addr = address
            .to_socket_addrs()
            .expect("Cannot parse address")
            .find(|addr| addr.is_ipv4())
            .expect("Not able to resolve the provided host name to an IPv4 address");

        let adapter = REMOTE_SERVER_POOL_REF
            .take(|| Arc::new(RemoteMockServerAdapter::new(addr)))
            .await;
        Self::from(adapter, REMOTE_SERVER_POOL_REF.clone()).await
    }

    /// Synchronously connects to a remote mock server that is running in standalone mode using
    /// the provided address of the form <host>:<port> (e.g. "127.0.0.1:8080") to establish
    /// the connection.
    pub fn connect(address: &str) -> Self {
        Self::connect_async(address).join()
    }

    /// Asynchronously connects to a remote mock server that is running in standalone mode using
    /// connection parameters stored in `HTTPMOCK_HOST` and `HTTPMOCK_PORT` environment variables.
    pub async fn connect_from_env_async() -> Self {
        let host = read_env("HTTPMOCK_HOST", "127.0.0.1");
        let port = read_env("HTTPMOCK_PORT", "5000")
            .parse::<u16>()
            .expect("Cannot parse environment variable HTTPMOCK_PORT to an integer");
        Self::connect_async(&format!("{}:{}", host, port)).await
    }

    /// Synchronously connects to a remote mock server that is running in standalone mode using
    /// connection parameters stored in `HTTPMOCK_HOST` and `HTTPMOCK_PORT` environment variables.
    pub fn connect_from_env() -> Self {
        Self::connect_from_env_async().join()
    }

    /// Starts a new `MockServer` asynchronously.
    ///
    /// Attention: This library manages a pool of `MockServer` instances in the background.
    /// Instead of always starting a new mock server, a `MockServer` instance is only created
    /// on demand if there is no free `MockServer` instance in the pool and the pool has not
    /// reached a maximum size yet. Otherwise, *THIS METHOD WILL BLOCK* the executing function
    /// until a free mock server is available.
    ///
    /// This allows to run many tests in parallel, but will prevent exhaust the executing
    /// machine by creating too many mock servers.
    ///
    /// A `MockServer` instance is automatically taken from the pool whenever this method is called.
    /// The instance is put back into the pool automatically when the corresponding
    /// 'MockServer' variable gets out of scope.
    pub async fn start_async() -> Self {
        let adapter = LOCAL_SERVER_POOL_REF
            .take(LOCAL_SERVER_ADAPTER_GENERATOR)
            .await;
        Self::from(adapter, LOCAL_SERVER_POOL_REF.clone()).await
    }

    /// Starts a new `MockServer` synchronously.
    ///
    /// Attention: This library manages a pool of `MockServer` instances in the background.
    /// Instead of always starting a new mock server, a `MockServer` instance is only created
    /// on demand if there is no free `MockServer` instance in the pool and the pool has not
    /// reached a maximum size yet. Otherwise, *THIS METHOD WILL BLOCK* the executing function
    /// until a free mock server is available.
    ///
    /// This allows to run many tests in parallel, but will prevent exhaust the executing
    /// machine by creating too many mock servers.
    ///
    /// A `MockServer` instance is automatically taken from the pool whenever this method is called.
    /// The instance is put back into the pool automatically when the corresponding
    /// 'MockServer' variable gets out of scope.
    pub fn start() -> MockServer {
        Self::start_async().join()
    }

    /// The hostname of the `MockServer`. By default, this is always `127.0.0.1`.
    /// In standalone mode, the hostname will be the one where the remote mock server is
    /// running.
    pub fn host(&self) -> String {
        self.server_adapter.as_ref().unwrap().host()
    }

    /// The TCP port that the mock server is listening on.
    pub fn port(&self) -> u16 {
        self.server_adapter.as_ref().unwrap().port()
    }

    /// Builds the address for a specific path on the mock server.
    ///
    /// **Example**:
    /// ```rust
    /// // Start a local mock server for exclusive use by this test function.
    /// let mock_server = httpmock::MockServer::start();
    /// let expected_addr_str = format!("127.0.0.1:{}", mock_server.port());
    ///
    /// // Get the address of the MockServer.
    /// let addr = mock_server.address();
    ///
    /// // Ensure the returned URL is as expected
    /// assert_eq!(expected_addr_str, addr.to_string());
    /// ```
    pub fn address(&self) -> &SocketAddr {
        self.server_adapter.as_ref().unwrap().address()
    }

    /// Builds the URL for a specific path on the mock server.
    ///
    /// **Example**:
    /// ```rust
    /// // Start a local mock server for exclusive use by this test function.
    /// let mock_server = httpmock::MockServer::start();
    /// let expected_url = format!("http://127.0.0.1:{}/hello", mock_server.port());
    ///
    /// // Get the URL for path "/hello".
    /// let url = mock_server.url("/hello");
    ///
    /// // Ensure the returned URL is as expected
    /// assert_eq!(expected_url, url);
    /// ```
    pub fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.address(), path)
    }

    /// Builds the base URL for the mock server.
    ///
    /// **Example**:
    /// ```rust
    /// // Start a local mock server for exclusive use by this test function.
    /// let mock_server = httpmock::MockServer::start();
    /// let expected_url = format!("http://127.0.0.1:{}", mock_server.port());
    ///
    /// // Get the URL for path "/hello".
    /// let url = mock_server.base_url();
    ///
    /// // Ensure the returned URL is as expected
    /// assert_eq!(expected_url, url);
    /// ```
    pub fn base_url(&self) -> String {
        self.url("")
    }

    /// Builds the base URL for the mock server.
    ///
    /// ```
    pub fn mock<F>(&self, config_fn: F) -> MockRef
    where
        F: FnOnce(Expectations, Responders),
    {
        self.mock_async(config_fn).join()
    }

    /// Builds the base URL for the mock server.
    ///
    /// ```
    pub async fn mock_async<'a, F>(&'a self, config_fn: F) -> MockRef<'a>
    where
        F: FnOnce(Expectations, Responders),
    {
        let mock = Rc::new(Cell::new(Mock::new()));
        config_fn(
            Expectations { mock: mock.clone() },
            Responders { mock: mock.clone() },
        );
        mock.take().create_on_async(self).await
    }
}

pub struct Expectations {
    pub(crate) mock: Rc<Cell<Mock>>,
}

impl Expectations {
    /// Sets the expected HTTP method. If the path of an HTTP request at the server matches this regex,
    /// the request will be considered a match for this mock to respond (given all other
    /// criteria are met).
    /// * `method` - The HTTP method to match against.
    pub fn method(self, method: Method) -> Self {
        self.mock.set(self.mock.take().expect_method(method));
        self
    }

    /// Sets the expected path. If the path of an HTTP request at the server is equal to the
    /// provided path, the request will be considered a match for this mock to respond (given all
    /// other criteria are met).
    /// * `path` - The exact path to match against.
    pub fn path(self, path: &str) -> Self {
        self.mock.set(self.mock.take().expect_path(path));
        self
    }

    /// Sets an expected path substring. If the path of an HTTP request at the server contains t,
    /// his substring the request will be considered a match for this mock to respond (given all
    /// other criteria are met).
    /// * `substring` - The substring to match against.
    pub fn path_contains(self, substring: &str) -> Self {
        self.mock
            .set(self.mock.take().expect_path_contains(substring));
        self
    }

    /// Sets an expected path regex. If the path of an HTTP request at the server matches this,
    /// regex the request will be considered a match for this mock to respond (given all other
    /// criteria are met).
    /// * `regex` - The regex to match against.
    pub fn path_matches(self, regex: Regex) -> Self {
        self.mock.set(self.mock.take().expect_path_matches(regex));
        self
    }

    /// Sets an expected query parameter. If the query parameters of an HTTP request at the server
    /// contains the provided query parameter name and value, the request will be considered a
    /// match for this mock to respond (given all other criteria are met).
    /// * `name` - The query parameter name that will matched against.
    /// * `value` - The value parameter name that will matched against.
    pub fn query_param(self, name: &str, value: &str) -> Self {
        self.mock
            .set(self.mock.take().expect_query_param(name, value));
        self
    }

    /// Sets an expected query parameter name. If the query parameters of an HTTP request at the server
    /// contains the provided query parameter name (not considering the value), the request will be
    /// considered a match for this mock to respond (given all other criteria are met).
    /// * `name` - The query parameter name that will matched against.
    pub fn query_param_exists(self, name: &str) -> Self {
        self.mock
            .set(self.mock.take().expect_query_param_exists(name));
        self
    }

    /// Sets the expected HTTP body. If the body of an HTTP request at the server matches the
    /// provided body, the request will be considered a match for this mock to respond
    /// (given all other criteria are met). This is an exact match, so all characters are taken
    /// into account, such as whitespace, tabs, etc.
    ///  * `contents` - The HTTP body to match against.
    pub fn body(self, body: &str) -> Self {
        self.mock.set(self.mock.take().expect_body(body));
        self
    }

    /// Sets an expected HTTP body regex. If the body of an HTTP request at the server matches
    /// the provided regex, the request will be considered a match for this mock to respond
    /// (given all other criteria are met).
    /// * `regex` - The regex that will matched against.
    pub fn body_matches(self, regex: Regex) -> Self {
        self.mock.set(self.mock.take().expect_body_matches(regex));
        self
    }
    /// Sets an expected HTTP body substring. If the body of an HTTP request at the server contains
    /// the provided substring, the request will be considered a match for this mock to respond
    /// (given all other criteria are met).
    /// * `substring` - The substring that will matched against.
    pub fn body_contains(self, substring: &str) -> Self {
        self.mock
            .set(self.mock.take().expect_body_contains(substring));
        self
    }
    /// Sets the expected JSON body. This method expects a serde_json::Value object.
    /// If the body of an HTTP request at the server matches the body according to the
    /// provided JSON value, the request will be considered a match for this mock to
    /// respond (given all other criteria are met).
    ///
    /// This is an exact match, so all elements are taken into account.
    ///
    /// Note that this method does not set the "Content-Type" header
    /// automatically, so you need to provide one yourself!
    ///
    /// * `body` - The HTTP body as a json value (serde_json::Value).
    pub fn json_body(self, value: serde_json::Value) -> Self {
        self.mock.set(self.mock.take().expect_json_body(value));
        self
    }
    /// Sets the expected JSON body. This method expects a serializable serde object
    /// that will be serialized/deserialized to/from a JSON string. If the body of an HTTP
    /// request at the server matches the body according to the provided JSON object,
    /// the request will be considered a match for this mock to respond (given all other
    /// criteria are met).
    ///
    /// This is an exact match, so all elements are taken into account.
    ///
    /// The provided JSON object needs to be both, a deserializable and
    /// serializable serde object. Note that this method does not set the "Content-Type" header
    /// automatically, so you need to provide one yourself!
    ///
    /// * `body` - The HTTP body object that will be serialized to JSON using serde.
    pub fn json_body_obj<T>(self, body: &T) -> Self
    where
        T: Serialize,
    {
        self.mock.set(self.mock.take().expect_json_body_obj(body));
        self
    }
    /// Sets an expected partial HTTP body JSON string.
    ///
    /// If the body of an HTTP request at the server matches the
    /// partial, the request will be considered a match for
    /// this mock to respond (given all other criteria are met).
    ///
    /// * `partial` - The JSON partial.
    ///
    /// # Important
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
    /// ```rust
    /// // Arrange: Create mock server and a mock
    /// use httpmock::{MockServer, Mock};
    ///
    /// let mock_server = MockServer::start();
    /// let mut mock = mock_server.mock(|when, then| {
    ///    when.json_body_partial(r#"
    ///         {
    ///             "child" : {
    ///                 "target_attribute" : "Target value"
    ///             }
    ///          }
    ///     "#);
    ///     then.status(202);
    /// });
    /// ```
    /// String format and attribute order will be ignored.
    pub fn json_body_partial(self, partial: &str) -> Self {
        self.mock
            .set(self.mock.take().expect_json_body_partial(partial));
        self
    }

    /// Sets an expected HTTP header. If one of the headers of an HTTP request at the server matches
    /// the provided header key and value, the request will be considered a match for this mock to
    /// respond (given all other criteria are met).
    ///
    /// * `name` - The HTTP header name (header names are case-insensitive by RFC 2616).
    /// * `value` - The HTTP header value.
    pub fn header(self, name: &str, value: &str) -> Self {
        self.mock.set(self.mock.take().expect_header(name, value));
        self
    }

    /// Sets an expected HTTP header to exists. If one of the headers of an HTTP request at the
    /// server matches the provided header name, the request will be considered a match for this
    /// mock to respond (given all other criteria are met).
    ///
    /// * `name` - The HTTP header name (header names are case-insensitive by RFC 2616).
    pub fn header_exists(self, name: &str) -> Self {
        self.mock.set(self.mock.take().expect_header_exists(name));
        self
    }
    /// Sets the cookie that needs to exist in the HTTP request.
    /// Cookie parsing follows RFC-6265 (https://tools.ietf.org/html/rfc6265.html).
    ///
    /// * `name` - The cookie name.
    /// * `value` - The expected cookie value.
    pub fn cookie(self, name: &str, value: &str) -> Self {
        self.mock.set(self.mock.take().expect_cookie(name, value));
        self
    }
    /// Sets the cookie that needs to exist in the HTTP request.
    /// Cookie parsing follows RFC-6265 (https://tools.ietf.org/html/rfc6265.html).
    ///
    /// * `name` - The cookie name
    pub fn cookie_exists(self, name: &str) -> Self {
        self.mock.set(self.mock.take().expect_cookie_exists(name));
        self
    }
    /// Sets a custom matcher for expected HTTP request. If this function returns true, the request
    /// is considered a match and the mock server will respond to the request
    /// (given all other criteria are also met).
    /// * `request_matcher` - The matcher function.
    ///
    /// ## Example:
    /// ```rust
    /// use httpmock::{MockServer, Mock, MockServerRequest};
    ///
    /// // Arrange
    /// let mock_server = MockServer::start();
    /// let m = mock_server.mock(|when, then|{
    ///    when.matches(|req: MockServerRequest| {
    ///         req.path.contains("es")
    ///    });
    ///    then.status(200);
    /// });
    ///
    /// // Act: Send the HTTP request
    /// let response = isahc::get(mock_server.url("/test")).unwrap();
    ///
    /// // Assert
    /// assert_eq!(response.status(), 200);
    /// assert_eq!(m.times_called(), 1);
    /// ```
    pub fn matches(self, matcher: MockMatcherFunction) -> Self {
        self.mock.set(self.mock.take().expect_match(matcher));
        self
    }
}

pub struct Responders {
    pub(crate) mock: Rc<Cell<Mock>>,
}

impl Responders {
    /// Sets the HTTP status that the mock will return, if an HTTP request fulfills all of
    /// the mocks requirements.
    /// * `status` - The HTTP status that the mock server will return.
    pub fn status(self, status: usize) -> Self {
        self.mock.set(self.mock.take().return_status(status));
        self
    }

    /// Sets the HTTP response body that the mock will return, if an HTTP request fulfills all of
    /// the mocks requirements.
    /// * `body` - The HTTP response body that the mock server will return.
    pub fn body(self, body: &str) -> Self {
        self.mock.set(self.mock.take().return_body(body));
        self
    }

    /// Sets the JSON body for the HTTP response that will be returned by the mock server.
    ///
    /// The provided JSON object needs to be both, a deserializable and
    /// serializable serde object. Note that this method does not set the "Content-Type" header
    /// automatically, so you need to provide one yourself!
    ///
    /// * `body` -  The HTTP response body the mock server will return in the form of a
    ///             serde_json::Value object.
    /// ```
    pub fn json_body(self, value: Value) -> Self {
        self.mock.set(self.mock.take().return_json_body(value));
        self
    }

    /// Sets the JSON body for the HTTP response that will be returned by the mock server.
    ///
    /// The provided JSON object needs to be both, a deserializable and
    /// serializable serde object. Note that this method does not set the "Content-Type" header
    /// automatically, so you need to provide one yourself!
    ///
    /// * `body` -  The HTTP response body the mock server will return in the form of a
    ///             serde_json::Value object.
    ///
    pub fn json_body_obj<T>(self, body: &T) -> Self
    where
        T: Serialize,
    {
        self.mock.set(self.mock.take().return_json_body_obj(body));
        self
    }

    /// Sets an HTTP header that the mock will return, if an HTTP request fulfills all of
    /// the mocks requirements.
    /// * `name` - The name of the header.
    /// * `value` - The value of the header.
    pub fn header(self, name: &str, value: &str) -> Self {
        self.mock.set(self.mock.take().return_header(name, value));
        self
    }

    /// Sets a duration that will delay the mock server response.
    /// * `duration` - The delay.
    pub fn delay(self, duration: Duration) -> Self {
        self.mock.set(self.mock.take().return_with_delay(duration));
        self
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        let adapter = self.server_adapter.take().unwrap();
        self.pool.put(adapter).join();
    }
}

const LOCAL_SERVER_ADAPTER_GENERATOR: fn() -> Arc<dyn MockServerAdapter + Send + Sync> = || {
    let (addr_sender, addr_receiver) = tokio::sync::oneshot::channel::<SocketAddr>();
    let state = Arc::new(MockServerState::new());
    let server_state = state.clone();

    thread::spawn(move || {
        let config = HttpMockConfig::new(0, false);
        let server_state = server_state.clone();

        let srv = start_server(config, &server_state, Some(addr_sender));

        let mut runtime = tokio::runtime::Builder::new()
            .enable_all()
            .basic_scheduler()
            .build()
            .expect("Cannot build local tokio runtime");

        LocalSet::new().block_on(&mut runtime, srv)
    });

    // TODO: replace this join by await
    let addr = addr_receiver.join().expect("Cannot get server address");
    Arc::new(LocalMockServerAdapter::new(addr, state))
};

lazy_static! {
    static ref LOCAL_SERVER_POOL_REF: Arc<Pool<Arc<dyn MockServerAdapter + Send + Sync>>> = {
        let max_servers = read_env("HTTPMOCK_MAX_SERVERS", "25")
            .parse::<usize>()
            .expect("Cannot parse environment variable HTTPMOCK_MAX_SERVERS to an integer");
        Arc::new(Pool::new(max_servers))
    };
    static ref REMOTE_SERVER_POOL_REF: Arc<Pool<Arc<dyn MockServerAdapter + Send + Sync>>> =
        Arc::new(Pool::new(1));
}
