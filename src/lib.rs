//! HTTP mocking library that allows you to simulate responses from HTTP based services.
//!
//!  # Features
//! * Simple, expressive, fluent API.
//! * Many built-in helpers for easy request matching.
//! * Parallel test execution.
//! * Extensible request matching.
//! * Two interchangeable API DSLs for mock definition.
//! * Fully asynchronous core with synchronous and asynchronous APIs.
//! * Debugging support
//! * Standalone mode with an accompanying [Docker image](https://hub.docker.com/r/alexliesenfeld/httpmock).
//! * Network delay simulation
//! * Support for [Regex](type.Regex.html) matching, JSON, [serde](https://crates.io/crates/serde), cookies, and more.

//! # Getting Started
//! Add `httpmock` to `Cargo.toml`:
//!
//! ```toml
//! [dev-dependencies]
//! httpmock = "0.5.0"
//! ```
//!
//! You can then use `httpmock` as follows:
//! ```
//! use httpmock::MockServer;
//!
//! // Start a lightweight mock server.
//! let mock_server = MockServer::start();
//!
//! // Create a mock on the server.
//! let hello_mock = mock_server.mock(|when, then| {
//!     when.method(GET)
//!         .path("/translate")
//!         .query_param("word", "hello");
//!     then.status(200)
//!         .header("Content-Type", "text/html")
//!         .body("ohi");
//! });
//!
//! // Send an HTTP request to the mock server. This simulates your code.
//! let response = isahc::get(mock_server.url("/translate?word=hello")).unwrap();
//!
//! // Ensure the mock server did respond as specified.
//! assert_eq!(response.status(), 200);
//! // Ensure the specified mock was called exactly one time.
//! assert_eq!(hello_mock.times_called(), 1);
//! ```
//! # Usage
//! To be able to configure mocks, you first need to start a mock server by calling
//! [MockServer::start](struct.MockServer.html#method.start).
//! This will spin up a lightweight HTTP
//! mock server in the background and wait until the server is ready to accept requests.
//!
//! You can then create a [Mock](struct.Mock.html) object on the server by using the
//! [MockServer::mock](struct.MockServer.html#method.mock) method. This method expects a closure
//! with two parameters, that we will refer to as the `when` and `then` parameter:
//! - The `when` parameter is of type [When](struct.When.html) and holds all request characteristics.
//! The mock server will only respond to HTTP requests that meet all the criteria. Otherwise it
//! will respond with HTTP status code `404` and an error message.
//! - The `then` parameter is of type [Then](struct.Then.html) and holds all values that the mock
//! server will respond with.
//!
//! # Sync / Async
//! The internal implementation of `httpmock` is completely asynchronous. It provides you a
//! synchronous and an asynchronous API though. If you want to schedule awaiting operations manually, then
//! you can use the `async` variants that exist for every potentially blocking operation. For
//! example, there is [MockServer::start_async](struct.MockServer.html#method.start_async) as an
//! asynchronous counterpart to [MockServer::start](struct.MockServer.html#method.start). You can
//! find similar methods throughout the entire library.
//!
//! # Parallelism
//! To balance execution speed and resource consumption, [MockServer](struct.MockServer.html)s
//! are kept in a server pool internally. This allows to run tests in parallel without overwhelming
//! the executing machine by creating too many HTTP servers. A test will be blocked if it tries to
//! use a [MockServer](struct.MockServer.html) (e.g. by calling
//! [MockServer::start](struct.MockServer.html#method.start)) while the server pool is empty
//! (i.e. all servers are occupied by other tests).
//!
//! [MockServer](struct.MockServer.html)s are never recreated but recycled/resetted.
//! The pool is filled on demand up to a maximum number of 25 servers.
//! You can override this number by using the environment variable `HTTPMOCK_MAX_SERVERS`.
//!
//! # Debugging
//! `httpmock` logs against the [log](https://crates.io/crates/log) crate. This allows you to
//! see detailed log output that contains information about `httpmock`s behaviour.
//! You can use this log output to investigate
//! issues, such as to find out why a request does not match a mock definition.
//!
//! The most useful log level is `debug`, but you can also go down to `trace` to see even more
//! information.
//!
//! **Attention**: To be able to see the log output, you need to add the `--nocapture` argument
//! when starting test execution!
//!
//! *Hint*: If you use the `env_logger` backend, you need to set the `RUST_LOG` environment variable to
//! `httpmock=debug`.
//!
//! # API Alternatives
//! This library provides two functionally interchangeable DSL APIs that allow you to create
//! mocks on the server. You can choose the one you like best or use both side-by-side. For a
//! consistent look, it is recommended to stick to one of them, though.
//!
//! ## When/Then API
//! This is the default API of `httpmock`. It is concise and easy to read. The main goal
//! is to reduce overhead emposed by this library to a bare minimum. It works well with
//! formatting tools, such as [rustfmt](https://crates.io/crates/rustfmt) (i.e. `cargo fmt`),
//! and can fully benefit from IDE support.
//!
//! ### Example
//! ```
//! let mock_server = httpmock::MockServer::start();
//!
//! let greeting_mock = mock_server.mock(|when, then| {
//!     when.path("/hi");
//!     then.status(200);
//! });
//!
//! let response = isahc::get(mock_server.url("/hi")).unwrap();
//! assert_eq!(hello_mock.times_called(), 1);
//! ```
//! Note that `when` and `then` are variables. This allows you to rename them to something you
//! like better (such as `expect`/`respond_with`).
//!
//! Relevant elements for this API are [MockServer::mock](struct.MockServer.html#method.mock), [When](struct.When.html) and [Then](struct.Then.html).
//!
//! ## Expect/Return API
//! This is the historical API of `httpmock`. It is verbous but explicit and provides
//! much control over mock definition reuse. You should almost never need to use it though.
//!
//! ### Example
//! ```
//! use httpmock::{MockServer, Mock};
//! let mock_server = MockServer::start();
//!
//! let greeting_mock = Mock::new()
//!     .expect_path("/hi")
//!     .return_status(200)
//!     .create_on(&mock_server);
//!
//! let response = isahc::get(mock_server.url("/hi")).unwrap();
//! assert_eq!(hello_mock.times_called(), 1);
//! ```
//! Please observe the following method naming scheme:
//! - All [Mock](struct.Mock.html) methods that start with `expect` in their name set a requirement
//! for HTTP requests (e.g. [Mock::expect_method](struct.Mock.html#method.expect_method),
//! [Mock::expect_path](struct.Mock.html#method.expect_path),
//! [Mock::expect_body](struct.Mock.html#method.expect_body), etc.).
//! - All [Mock](struct.Mock.html) methods that start with `return` in their name define what the
//! mock server will return in response to an HTTP request that matched all mock requirements (e.g.
//! [Mock::return_status](struct.Mock.html#method.return_status),
//! [Mock::return_body](struct.Mock.html#method.return_body), etc.).
//!
//! With this naming scheme users can benefit from IDE autocompletion to find request matchers and
//! response attributes mostly without even looking into documentation.
//!
//! # Examples
//! You can find examples in the test directory in this crates Git repository:
//! [this crates test directory](https://github.com/alexliesenfeld/httpmock/blob/master/tests ).
//!
//! # Standalone Mode
//! You can use `httpmock` to run a standalone mock server that runs in a separate process.
//! This allows it to be available to multiple applications, not only inside your unit and integration
//! tests. This is useful if you want to use `httpmock` in system (or even end-to-end) tests, that
//! require mocked services. With this feature, `httpmock` is a universal HTTP mocking tool that is
//! useful in all stages of the development lifecycle.
//!
//! ## Using a Standalone Mock Server
//! Although you can build the mock server in standalone mode yourself, it is easiest to use the
//! accompanying [Docker image](https://hub.docker.com/r/alexliesenfeld/httpmock).
//!
//! To be able to use the standalone server from within your tests, you need to change how an
//! instance of the [MockServer](struct.MockServer.html) instance is created.
//! Instead of using [MockServer::start](struct.MockServer.html#method.start),
//! you need to connect to a remote server by using one of the `connect` methods (such as
//! [MockServer::connect](struct.MockServer.html#method.connect) or
//! [MockServer::connect_from_env](struct.MockServer.html#method.connect_from_env)).
//!
//! ```
//! use httpmock::{MockServer, Mock};
//! use isahc::get;
//!
//! #[test]
//! fn simple_test() {
//!     // Arrange
//!     let mock_server = MockServer::connect("some-host:5000");
//!
//!     let hello_mock = mock_server.mock(|when, then|{
//!         when.path("/hello/standalone");
//!         then.status(200);
//!     });
//!
//!     // Act
//!     let response = get(mock_server.url("/hello/standalone")).unwrap();
//!
//!     // Assert
//!     assert_eq!(response.status(), 200);
//!     assert_eq!(hello_mock.times_called(), 1);
//! }
//! ```
//!
//! ## Standalone Parallelism
//! Tests that use a remote mock server are executed sequentially by default. This is in
//! contrast to tests that use a local mock server. Sequential execution is achieved by
//! blocking all tests from further execution whenever a test requires to connect to a
//! busy mock server.
//!
//! ## Limitations of the Standalone Mode
//! At this time, it is not possible to use custom request matchers in combination with standalone
//! mock servers (see [When::matches](struct.When.html#method.matches) or
//! [Mock::expect_match](struct.Mock.html#method.expect_match)).
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
use crate::server::{start_server, MockServerState};
use crate::util::{read_env, with_retry};
use futures_util::core_reexport::time::Duration;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cell::Cell;

mod api;
mod server;
mod util;

pub mod standalone {
    use std::sync::Arc;

    use crate::server::{start_server, MockServerState};

    pub async fn start_standalone_server(port: u16, expose: bool) -> Result<(), String> {
        let state = Arc::new(MockServerState::new());
        start_server(port, expose, &state, None).await
    }
}

/// A general abstraction of an HTTP request of `httpmock`.
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

    /// The hostname of the `MockServer`. By default, this is `127.0.0.1`.
    /// In standalone mode, the hostname will be the host where the standalone mock server is
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
    /// ```
    /// // Start a local mock server for exclusive use by this test function.
    /// let mock_server = httpmock::MockServer::start();
    ///
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
    /// ```
    /// // Start a local mock server for exclusive use by this test function.
    /// let mock_server = httpmock::MockServer::start();
    ///
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
    /// ```
    /// // Start a local mock server for exclusive use by this test function.
    /// let mock_server = httpmock::MockServer::start();
    ///
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

    /// Creates a [Mock](struct.Mock.html) object on the mock server.
    ///
    /// **Example**:
    /// ```
    /// let mock_server = httpmock::MockServer::start();
    ///
    /// let mock = mock_server.mock(|when, then| {
    ///     when.path("/hello");
    ///     then.status(200);
    /// });
    ///
    /// assert_eq!(mock.times_called(), 0);
    /// ```
    pub fn mock<F>(&self, config_fn: F) -> MockRef
    where
        F: FnOnce(When, Then),
    {
        self.mock_async(config_fn).join()
    }

    /// Creates a [Mock](struct.Mock.html) object on the mock server.
    ///
    /// **Example**:
    /// ```
    /// async_std::task::block_on(async {
    ///     let mock_server = httpmock::MockServer::start();
    ///
    ///     let mock = mock_server
    ///         .mock_async(|when, then| {
    ///             when.path("/hello");
    ///             then.status(200);
    ///         })
    ///         .await;
    ///
    ///     assert_eq!(mock.times_called(), 0);
    /// });
    /// ```
    pub async fn mock_async<'a, F>(&'a self, config_fn: F) -> MockRef<'a>
    where
        F: FnOnce(When, Then),
    {
        let mock = Rc::new(Cell::new(Mock::new()));
        config_fn(When { mock: mock.clone() }, Then { mock: mock.clone() });
        mock.take().create_on_async(self).await
    }
}

/// A builder that allows concise specification of HTTP request values.
pub struct When {
    pub(crate) mock: Rc<Cell<Mock>>,
}

impl When {
    /// Sets the expected HTTP method.
    ///
    /// * `method` - The HTTP method.
    ///
    /// # Example
    /// ```
    /// use httpmock::{Mock, MockServer};
    /// use httpmock::Method::GET;
    /// use regex::Regex;
    ///
    /// let mock_server = MockServer::start();
    ///
    /// let mock = mock_server.mock(|when, then|{
    ///     when.method(GET);
    ///     then.status(200);
    /// });
    ///
    /// isahc::get(mock_server.url("/")).unwrap();
    ///
    /// assert_eq!(mock.times_called(), 1);
    /// ```
    pub fn method(self, method: Method) -> Self {
        self.mock.set(self.mock.take().expect_method(method));
        self
    }

    /// Sets the expected URL path.
    /// * `path` - The URL path.
    ///
    /// # Example
    /// ```
    /// use httpmock::{Mock, MockServer};
    ///
    /// let mock_server = MockServer::start();
    ///
    /// let mock = mock_server.mock(|when, then|{
    ///     when.path_contains("/test");
    ///     then.status(200);
    /// });
    ///
    /// isahc::get(mock_server.url("/test")).unwrap();
    ///
    /// assert_eq!(mock.times_called(), 1);
    /// ```
    pub fn path(self, path: &str) -> Self {
        self.mock.set(self.mock.take().expect_path(path));
        self
    }

    /// Sets an substring that the URL path needs to contain.
    /// * `substring` - The substring to match against.
    ///
    /// # Example
    /// ```
    /// use httpmock::{Mock, MockServer};
    ///
    /// let mock_server = MockServer::start();
    ///
    /// let mock = mock_server.mock(|when, then|{
    ///     when.path_contains("es");
    ///     then.status(200);
    /// });
    ///
    /// isahc::get(mock_server.url("/test")).unwrap();
    ///
    /// assert_eq!(mock.times_called(), 1);
    /// ```
    pub fn path_contains(self, substring: &str) -> Self {
        self.mock
            .set(self.mock.take().expect_path_contains(substring));
        self
    }

    /// Sets a regex that the URL path needs to match.
    /// * `regex` - The regex to match against.
    ///
    /// # Example
    /// ```
    /// use httpmock::{Mock, MockServer};
    /// use regex::Regex;
    ///
    /// let mock_server = MockServer::start();
    ///
    /// let mock = mock_server.mock(|when, then|{
    ///     when.path_matches(Regex::new("le$").unwrap());
    ///     then.status(200);
    /// });
    ///
    /// isahc::get(mock_server.url("/example")).unwrap();
    ///
    /// assert_eq!(mock.times_called(), 1);
    /// ```
    pub fn path_matches(self, regex: Regex) -> Self {
        self.mock.set(self.mock.take().expect_path_matches(regex));
        self
    }

    /// Sets a query parameter that needs to be provided.
    /// * `name` - The query parameter name that will matched against.
    /// * `value` - The value parameter name that will matched against.
    ///
    /// ```
    /// // Arrange
    /// use isahc::get;
    /// use httpmock::{MockServer, Mock};
    ///
    /// let _ = env_logger::try_init();
    /// let mock_server = MockServer::start();
    ///
    /// let m = mock_server.mock(|when, then|{
    ///     when.query_param("query", "Metallica");
    ///     then.status(200);
    /// });
    ///
    /// // Act
    /// get(mock_server.url("/search?query=Metallica")).unwrap();
    ///
    /// // Assert
    /// assert_eq!(m.times_called(), 1);
    /// ```
    pub fn query_param(self, name: &str, value: &str) -> Self {
        self.mock
            .set(self.mock.take().expect_query_param(name, value));
        self
    }

    /// Sets a query parameter that needs to exist in an HTTP request.
    /// * `name` - The query parameter name that will matched against.
    ///
    /// ```
    /// // Arrange
    /// use isahc::get;
    /// use httpmock::{MockServer, Mock};
    ///
    /// let _ = env_logger::try_init();
    /// let mock_server = MockServer::start();
    ///
    /// let m = mock_server.mock(|when, then| {
    ///     when.query_param_exists("query");
    ///     then.status(200);
    /// });
    ///
    /// // Act
    /// get(mock_server.url("/search?query=Metallica")).unwrap();
    ///
    /// // Assert
    /// assert_eq!(m.times_called(), 1);
    /// ```
    pub fn query_param_exists(self, name: &str) -> Self {
        self.mock
            .set(self.mock.take().expect_query_param_exists(name));
        self
    }

    /// Sets the required HTTP request body content.
    ///
    /// * `body` - The required HTTP request body.
    ///
    /// # Example
    /// ```
    /// use httpmock::{Mock, MockServer};
    /// use httpmock::Method::GET;
    /// use regex::Regex;
    /// use isahc::prelude::*;
    ///
    /// let mock_server = MockServer::start();
    ///
    /// let mock = mock_server.mock(|when, then|{
    ///     when.body("The Great Gatsby");
    ///     then.status(200);
    /// });
    ///
    /// Request::post(&format!("http://{}/test", mock_server.address()))
    ///     .body("The Great Gatsby")
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// assert_eq!(mock.times_called(), 1);
    /// ```
    pub fn body(self, body: &str) -> Self {
        self.mock.set(self.mock.take().expect_body(body));
        self
    }

    /// Sets a [Regex](type.Regex.html) for the expected HTTP body.
    ///
    /// * `regex` - The regex that the HTTP request body will matched against.
    ///
    /// ```
    /// use isahc::prelude::*;
    /// use httpmock::Method::POST;
    /// use httpmock::{MockServer, Mock, Regex};
    ///
    /// // Arrange
    /// let _ = env_logger::try_init();
    ///
    /// let mock_server = MockServer::start();
    ///
    /// let m = mock_server.mock(|when, then|{
    ///     when.method(POST)
    ///         .path("/books")
    ///         .body_matches(Regex::new("Fellowship").unwrap());
    ///     then.status(201);
    /// });
    ///
    /// // Act: Send the request
    /// let response = Request::post(mock_server.url("/books"))
    ///     .body("The Fellowship of the Ring")
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// // Assert
    /// assert_eq!(response.status(), 201);
    /// assert_eq!(m.times_called(), 1);
    /// ```
    pub fn body_matches(self, regex: Regex) -> Self {
        self.mock.set(self.mock.take().expect_body_matches(regex));
        self
    }

    /// Sets the expected HTTP body substring.
    ///
    /// * `substring` - The substring that will matched against.
    ///
    /// ```
    /// use httpmock::{MockServer, Mock, Regex};
    /// use httpmock::Method::POST;
    /// use isahc::prelude::*;
    ///
    /// // Arrange
    /// let _ = env_logger::try_init();
    ///
    /// let mock_server = MockServer::start();
    ///
    /// let m = mock_server.mock(|when, then|{
    ///     when.path("/books")
    ///         .body_contains("Ring");
    ///     then.status(201);
    /// });
    ///
    /// // Act: Send the request
    /// let response = Request::post(mock_server.url("/books"))
    ///     .body("The Fellowship of the Ring")
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// // Assert
    /// assert_eq!(response.status(), 201);
    /// assert_eq!(m.times_called(), 1);
    /// ```
    pub fn body_contains(self, substring: &str) -> Self {
        self.mock
            .set(self.mock.take().expect_body_contains(substring));
        self
    }

    /// Sets the expected JSON body. This method expects a [serde_json::Value](../serde_json/enum.Value.html)
    /// that will be serialized/deserialized to/from a JSON string.
    ///
    /// Note that this method does not set the `Content-Type` header automatically, so you
    /// need to provide one yourself!
    ///
    /// * `body` - The HTTP body object that will be serialized to JSON using serde.
    ///
    /// ```
    /// use httpmock::{Mock, MockServer};
    /// use httpmock::Method::POST;
    /// use serde_json::json;
    /// use isahc::prelude::*;
    ///
    /// // Arrange
    /// let _ = env_logger::try_init();
    /// let mock_server = MockServer::start();
    ///
    /// let m = mock_server.mock(|when, then|{
    ///     when.path("/user")
    ///         .header("Content-Type", "application/json")
    ///         .json_body(json!({ "name": "Hans" }));
    ///     then.status(201);
    /// });
    ///
    /// // Act: Send the request and deserialize the response to JSON
    /// let mut response = Request::post(&format!("http://{}/user", mock_server.address()))
    ///     .header("Content-Type", "application/json")
    ///     .body(json!({ "name": "Hans" }).to_string())
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// // Assert
    /// assert_eq!(response.status(), 201);
    /// assert_eq!(m.times_called(), 1);
    /// ```
    pub fn json_body(self, value: serde_json::Value) -> Self {
        self.mock.set(self.mock.take().expect_json_body(value));
        self
    }

    /// Sets the expected JSON body. This method expects a serializable serde object
    /// that will be serialized/deserialized to/from a JSON string.
    ///
    /// Note that this method does not set the "Content-Type" header automatically, so you
    /// need to provide one yourself!
    ///
    /// * `body` - The HTTP body object that will be serialized to JSON using serde.
    ///
    /// ```
    /// use httpmock::{MockServer, Mock};
    /// use httpmock::Method::POST;
    /// use serde_json::json;
    /// use isahc::prelude::*;
    ///
    /// // This is a temporary type that we will use for this test
    /// #[derive(serde::Serialize, serde::Deserialize)]
    /// struct TestUser {
    ///     name: String,
    /// }
    ///
    /// // Arrange
    /// let _ = env_logger::try_init();
    /// let mock_server = MockServer::start();
    ///
    /// let m = mock_server.mock(|when, then|{
    ///     when.path("/user")
    ///         .header("Content-Type", "application/json")
    ///         .json_body_obj(&TestUser {
    ///             name: String::from("Fred"),
    ///         });
    ///     then.status(200);
    /// });
    ///
    /// // Act: Send the request and deserialize the response to JSON
    /// let mut response = Request::post(&format!("http://{}/user", mock_server.address()))
    ///     .header("Content-Type", "application/json")
    ///     .body(json!(&TestUser {
    ///         name: "Fred".to_string()
    ///     }).to_string())
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// // Assert
    /// assert_eq!(response.status(), 200);
    /// assert_eq!(m.times_called(), 1);
    /// ```
    pub fn json_body_obj<'a, T>(self, body: &T) -> Self
    where
        T: Serialize + Deserialize<'a>,
    {
        self.mock.set(self.mock.take().expect_json_body_obj(body));
        self
    }

    /// Sets the expected partial JSON body.
    ///
    /// **Attention: The partial string needs to be a valid JSON string. It must contain
    /// the full object hierarchy from the original JSON object but can leave out irrelevant
    /// attributes (see example).**
    ///
    /// Note that this method does not set the `Content-Type` header automatically, so you
    /// need to provide one yourself!
    ///
    /// String format and attribute order are irrelevant.
    ///
    /// * `partial_body` - The HTTP body object that will be serialized to JSON using serde.
    ///
    /// ## Example
    /// Suppose your application sends the following JSON request body:
    /// ```json
    /// {
    ///     "parent_attribute" : "Some parent data goes here",
    ///     "child" : {
    ///         "target_attribute" : "Example",
    ///         "other_attribute" : "Another value"
    ///     }
    /// }
    /// ```
    /// If we only want to verify that `target_attribute` has value `Example` without the need
    /// to provive a full JSON object, we can use this method as follows:
    /// ```
    /// use httpmock::{MockServer, Mock};
    ///
    /// let mock_server = MockServer::start();
    ///
    /// let mut mock = mock_server.mock(|when, then|{
    ///     when.json_body_partial(r#"
    ///         {
    ///             "child" : {
    ///                 "target_attribute" : "Example"
    ///             }
    ///          }
    ///     "#);
    ///     then.status(200);
    /// });
    /// ```
    /// Please note that the JSON partial contains the full object hierachy, i.e. it needs to start
    /// from the root! It leaves out irrelevant attributes, however (`parent_attribute`
    /// and `child.other_attribute`).
    pub fn json_body_partial(self, partial: &str) -> Self {
        self.mock
            .set(self.mock.take().expect_json_body_partial(partial));
        self
    }

    /// Sets the expected HTTP header.
    /// * `name` - The HTTP header name (header names are case-insensitive by RFC 2616).
    /// * `value` - The header value.
    ///
    /// # Example
    /// ```
    /// use httpmock::{Mock, MockServer};
    /// use httpmock::Method::GET;
    /// use regex::Regex;
    /// use isahc::prelude::*;
    ///
    /// let mock_server = MockServer::start();
    ///
    /// let mock = mock_server.mock(|when, then|{
    ///     when.header("Authorization", "token 1234567890");
    ///     then.status(200);
    /// });
    ///
    /// Request::post(&format!("http://{}/test", mock_server.address()))
    ///     .header("Authorization", "token 1234567890")
    ///     .body(())
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// assert_eq!(mock.times_called(), 1);
    /// ```
    pub fn header(self, name: &str, value: &str) -> Self {
        self.mock.set(self.mock.take().expect_header(name, value));
        self
    }

    /// Sets the requirement that the HTTP request needs to contain a specific header
    /// (value is unchecked, refer to [Mock::expect_header](struct.Mock.html#method.expect_header)).
    ///
    /// * `name` - The HTTP header name (header names are case-insensitive by RFC 2616).
    ///
    /// # Example
    /// ```
    /// use httpmock::{Mock, MockServer};
    /// use httpmock::Method::GET;
    /// use regex::Regex;
    /// use isahc::prelude::*;
    ///
    /// let mock_server = MockServer::start();
    ///
    /// let mock = mock_server.mock(|when, then|{
    ///     when.header_exists("Authorization");
    ///     then.status(200);
    /// });
    ///
    /// Request::post(&format!("http://{}/test", mock_server.address()))
    ///     .header("Authorization", "token 1234567890")
    ///     .body(())
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// assert_eq!(mock.times_called(), 1);
    /// ```
    pub fn header_exists(self, name: &str) -> Self {
        self.mock.set(self.mock.take().expect_header_exists(name));
        self
    }

    /// Sets the cookie that needs to exist in the HTTP request.
    /// Cookie parsing follows [RFC-6265](https://tools.ietf.org/html/rfc6265.html).
    ///
    /// * `name` - The cookie name.
    /// * `value` - The expected cookie value.
    ///
    /// # Example
    /// ```
    /// use httpmock::{Mock, MockServer};
    /// use httpmock::Method::GET;
    /// use regex::Regex;
    /// use isahc::prelude::*;
    ///
    /// let mock_server = MockServer::start();
    ///
    /// let mock = mock_server.mock(|when, then|{
    ///     when.cookie("SESSIONID", "1234567890");
    ///     then.status(200);
    /// });
    ///
    /// Request::post(&format!("http://{}/test", mock_server.address()))
    ///     .header("Cookie", "TRACK=12345; SESSIONID=1234567890; CONSENT=1")
    ///     .body(())
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// assert_eq!(mock.times_called(), 1);
    /// ```
    pub fn cookie(self, name: &str, value: &str) -> Self {
        self.mock.set(self.mock.take().expect_cookie(name, value));
        self
    }

    /// Sets the cookie that needs to exist in the HTTP request.
    /// Cookie parsing follows [RFC-6265](https://tools.ietf.org/html/rfc6265.html).
    ///
    /// * `name` - The cookie name
    ///
    /// # Example
    /// ```
    /// use httpmock::{Mock, MockServer};
    /// use httpmock::Method::GET;
    /// use regex::Regex;
    /// use isahc::prelude::*;
    ///
    /// let mock_server = MockServer::start();
    ///
    /// let mock = mock_server.mock(|when, then|{
    ///     when.cookie_exists("SESSIONID");
    ///     then.status(200);
    /// });
    ///
    /// Request::post(&format!("http://{}/test", mock_server.address()))
    ///     .header("Cookie", "TRACK=12345; SESSIONID=1234567890; CONSENT=1")
    ///     .body(())
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// assert_eq!(mock.times_called(), 1);
    /// ```
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
    /// ```
    /// use httpmock::{MockServer, Mock, MockServerRequest};
    ///
    /// // Arrange
    /// let mock_server = MockServer::start();
    ///
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

/// A builder that allows concise specification of HTTP response values.
pub struct Then {
    pub(crate) mock: Rc<Cell<Mock>>,
}

impl Then {
    /// Sets the HTTP response code that will be returned by the mock server.
    ///
    /// * `status` - The status code.
    ///
    /// ## Example:
    /// ```
    /// use httpmock::{MockServer, Mock, MockServerRequest};
    ///
    /// // Arrange
    /// let mock_server = MockServer::start();
    ///
    /// let m = mock_server.mock(|when, then|{
    ///     when.path("/hello");
    ///     then.status(200);
    /// });
    ///
    /// // Act
    /// let response = isahc::get(mock_server.url("/hello")).unwrap();
    ///
    /// // Assert
    /// assert_eq!(response.status(), 200);
    /// assert_eq!(m.times_called(), 1);
    /// ```
    pub fn status(self, status: usize) -> Self {
        self.mock.set(self.mock.take().return_status(status));
        self
    }

    /// Sets the HTTP response body that will be returned by the mock server.
    ///
    /// * `body` - The response body content.
    ///
    /// ## Example:
    /// ```
    /// use httpmock::{MockServer, Mock, MockServerRequest};
    /// use isahc::ResponseExt;
    ///
    /// // Arrange
    /// let mock_server = MockServer::start();
    ///
    /// let m = mock_server.mock(|when, then| {
    ///     when.path("/hello");
    ///     then.status(200)
    ///         .body("ohi!");
    /// });
    ///
    /// // Act
    /// let mut response = isahc::get(mock_server.url("/hello")).unwrap();
    ///
    /// // Assert
    /// assert_eq!(response.status(), 200);
    /// assert_eq!(response.text().unwrap(), "ohi!");
    /// assert_eq!(m.times_called(), 1);
    /// ```
    pub fn body(self, body: &str) -> Self {
        self.mock.set(self.mock.take().return_body(body));
        self
    }

    /// Sets the JSON body for the HTTP response that will be returned by the mock server.
    ///
    /// The provided JSON object needs to be both, a deserializable and serializable serde object.
    ///
    /// Note that this method does not set the "Content-Type" header automatically, so you need
    /// to provide one yourself!
    ///
    /// * `body` -  The HTTP response body the mock server will return in the form of a
    ///             serde_json::Value object.
    ///
    /// ## Example
    /// You can use this method conveniently as follows:
    /// ```
    /// use httpmock::{MockServer, Mock};
    /// use serde_json::{Value, json};
    /// use isahc::ResponseExt;
    /// use isahc::prelude::*;
    ///
    /// // Arrange
    /// let _ = env_logger::try_init();
    /// let mock_server = MockServer::start();
    ///
    /// let m = mock_server.mock(|when, then|{
    ///     when.path("/user");
    ///     then.status(200)
    ///         .header("Content-Type", "application/json")
    ///         .json_body(json!({ "name": "Hans" }));
    /// });
    ///
    /// // Act
    /// let mut response = isahc::get(mock_server.url("/user")).unwrap();
    ///
    /// let user: Value =
    ///     serde_json::from_str(&response.text().unwrap()).expect("cannot deserialize JSON");
    ///
    /// // Assert
    /// assert_eq!(response.status(), 200);
    /// assert_eq!(m.times_called(), 1);
    /// assert_eq!(user.as_object().unwrap().get("name").unwrap(), "Hans");
    /// ```
    pub fn json_body(self, value: Value) -> Self {
        self.mock.set(self.mock.take().return_json_body(value));
        self
    }

    /// Sets the JSON body that will be returned by the mock server.
    /// This method expects a serializable serde object that will be serialized/deserialized
    /// to/from a JSON string.
    ///
    /// Note that this method does not set the "Content-Type" header automatically, so you
    /// need to provide one yourself!
    ///
    /// * `body` - The HTTP body object that will be serialized to JSON using serde.
    ///
    /// ```
    /// use httpmock::{MockServer, Mock};
    /// use isahc::ResponseExt;
    ///
    /// // This is a temporary type that we will use for this example
    /// #[derive(serde::Serialize, serde::Deserialize)]
    /// struct TestUser {
    ///     name: String,
    /// }
    ///
    /// // Arrange
    /// let _ = env_logger::try_init();
    /// let mock_server = MockServer::start();
    ///
    /// let m = mock_server.mock(|when, then| {
    ///     when.path("/user");
    ///     then.status(200)
    ///         .header("Content-Type", "application/json")
    ///         .json_body_obj(&TestUser {
    ///             name: String::from("Hans"),
    ///         });
    /// });
    ///
    /// // Act
    /// let mut response = isahc::get(mock_server.url("/user")).unwrap();
    ///
    /// let user: TestUser =
    ///     serde_json::from_str(&response.text().unwrap()).unwrap();
    ///
    /// // Assert
    /// assert_eq!(response.status(), 200);
    /// assert_eq!(user.name, "Hans");
    /// assert_eq!(m.times_called(), 1);
    /// ```
    pub fn json_body_obj<T>(self, body: &T) -> Self
    where
        T: Serialize,
    {
        self.mock.set(self.mock.take().return_json_body_obj(body));
        self
    }

    /// Sets an HTTP header that the mock server will return.
    ///
    /// * `name` - The name of the header.
    /// * `value` - The value of the header.
    ///
    /// ## Example
    /// You can use this method conveniently as follows:
    /// ```
    /// // Arrange
    /// use httpmock::{MockServer, Mock};
    /// use serde_json::Value;
    /// use isahc::ResponseExt;
    ///
    /// let _ = env_logger::try_init();
    /// let mock_server = MockServer::start();
    ///
    /// let m = mock_server.mock(|when, then|{
    ///     when.path("/");
    ///     then.status(200)
    ///         .header("Expires", "Wed, 21 Oct 2050 07:28:00 GMT");
    /// });
    ///
    /// // Act
    /// let mut response = isahc::get(mock_server.url("/")).unwrap();
    ///
    /// // Assert
    /// assert_eq!(response.status(), 200);
    /// assert_eq!(m.times_called(), 1);
    /// ```
    pub fn header(self, name: &str, value: &str) -> Self {
        self.mock.set(self.mock.take().return_header(name, value));
        self
    }

    /// Sets the HTTP response up to return a temporary redirect.
    ///
    /// In detail, this method will add the following information to the HTTP response:
    /// - A "Location" header with the provided URL as its value.
    /// - Status code will be set to 302 (if no other status code was set before).
    /// - The response body will be set to "Found" (if no other body was set before).
    ///
    /// Further information: https://developer.mozilla.org/en-US/docs/Web/HTTP/Redirections
    /// and https://tools.ietf.org/html/rfc2616#section-10.3.8.
    ///
    /// * `redirect_url` - THe URL to redirect to.
    ///
    /// ## Example
    /// ```
    /// // Arrange
    /// use httpmock::MockServer;
    /// use isahc::ResponseExt;
    /// let _ = env_logger::try_init();
    ///
    /// let mock_server = MockServer::start();
    ///
    /// let redirect_mock = mock_server.mock(|when, then|{
    ///     when.path("/redirectPath");
    ///     then.temporary_redirect("http://www.google.com");
    /// });
    ///
    /// // Act: Send the HTTP request with an HTTP client that DOES NOT FOLLOW redirects automatically!
    /// let mut response = isahc::get(mock_server.url("/redirectPath")).unwrap();
    /// let body = response.text().unwrap();
    ///
    /// // Assert
    /// assert_eq!(redirect_mock.times_called(), 1);
    ///
    /// // Attention!: Note that all of these values are automatically added to the response
    /// // (see details in mock builder method documentation).
    /// assert_eq!(response.status(), 302);
    /// assert_eq!(body, "Found");
    /// assert_eq!(response.headers().get("Location").unwrap().to_str().unwrap(), target_url);
    /// ```
    pub fn temporary_redirect(mut self, redirect_url: &str) -> Self {
        self.mock.set(self.mock.take().return_temporary_redirect(redirect_url));
        self
    }

    /// Sets a duration that will delay the mock server response.
    ///
    /// * `duration` - The delay.
    ///
    /// ```
    /// // Arrange
    /// use std::time::{SystemTime, Duration};
    /// use httpmock::{MockServer, Mock};
    ///
    /// let _ = env_logger::try_init();
    /// let start_time = SystemTime::now();
    /// let three_seconds = Duration::from_secs(3);
    /// let mock_server = MockServer::start();
    ///
    /// let mock = mock_server.mock(|when, then| {
    ///     when.path("/delay");
    ///     then.status(200)
    ///         .delay(three_seconds);
    /// });
    ///
    /// // Act
    /// let response = isahc::get(mock_server.url("/delay")).unwrap();
    ///
    /// // Assert
    /// assert_eq!(mock.times_called(), 1);
    /// assert_eq!(start_time.elapsed().unwrap() > three_seconds, true);
    /// ```
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
        let server_state = server_state.clone();
        let srv = start_server(0, false, &server_state, Some(addr_sender));

        let mut runtime = tokio::runtime::Builder::new()
            .enable_all()
            .basic_scheduler()
            .build()
            .expect("Cannot build local tokio runtime");

        LocalSet::new().block_on(&mut runtime, srv)
    });

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
