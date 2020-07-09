//! `httpmock` is a Rust library that allows you to mock HTTP services in your tests.
//!
//!  # Features
//!
//! * Provides a full-blown HTTP mock server with HTTP/1 and HTTP/2 support.
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
//! httpmock = "0.4.0"
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
//!     // The mock_server variable tis being used to generate a mock server URL for path "/hello".
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
//! `httpmock` logs against the `log` crate. For example, if you use the `env_logger` backend,
//! you can activate debug logging by setting the `RUST_LOG` environment variable to `httpmock=debug`.
//!
//! # Standalone Mode
//! You can use `httpmock` to run a standalone mock server that is available to multiple
//! applications. This can be useful if you are running integration tests that involve both,
//! real and mocked applications.
//!
//! ## Docker
//! Altough you can build the mock server in standalone mode yourself, it is easiest to use
//! the Docker image from the accompanying [Docker image](https://hub.docker.com/r/alexliesenfeld/httpmock).
//! Please refer to the documentation on Docker repository.
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
//!     let search_mock = Mock::new()
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
//!     assert_eq!(search_mock.times_called(), 1);
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
use crate::server::data::MockServerHttpRequest;
pub use crate::server::HttpMockConfig;
use crate::server::{start_server, MockServerState};
use crate::util::{read_env, with_retry};

mod api;
mod server;
mod util;

pub mod standalone {
    use std::sync::Arc;

    use crate::server::HttpMockConfig;
    use crate::server::{start_server, MockServerState};

    pub async fn start_standalone_server(config: HttpMockConfig) -> Result<(), String> {
        let state = Arc::new(MockServerState::new());
        println!("test1");
        start_server(config, &state, None).await
    }
}

pub type MockServerRequest = Rc<MockServerHttpRequest>;

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

    pub fn connect(address: &str) -> Self {
        Self::connect_async(address).join()
    }

    pub async fn connect_from_env_async() -> Self {
        let host = read_env("HTTPMOCK_HOST", "127.0.0.1");
        let port = read_env("HTTPMOCK_PORT", "5000")
            .parse::<u16>()
            .expect("Cannot parse environment variable HTTPMOCK_PORT to an integer");
        Self::connect_async(&format!("{}:{}", host, port)).await
    }

    pub fn connect_from_env() -> Self {
        Self::connect_from_env_async().join()
    }

    pub async fn start_async() -> Self {
        let adapter = LOCAL_SERVER_POOL_REF
            .take(LOCAL_SERVER_ADAPTER_GENERATOR)
            .await;
        Self::from(adapter, LOCAL_SERVER_POOL_REF.clone()).await
    }

    pub fn start() -> MockServer {
        Self::start_async().join()
    }

    pub fn host(&self) -> String {
        self.server_adapter.as_ref().unwrap().host()
    }

    pub fn port(&self) -> u16 {
        self.server_adapter.as_ref().unwrap().port()
    }

    pub fn address(&self) -> &SocketAddr {
        self.server_adapter.as_ref().unwrap().address()
    }

    pub fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.address(), path)
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
