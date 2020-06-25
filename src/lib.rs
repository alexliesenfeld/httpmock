//! `httpmock` is a Rust crate that allows you to mock HTTP responses in your tests. It contains
//! two major components:
//!
//! * a **mock server** that is automatically started in the background of your tests, and
//! * a **test library** to create HTTP mocks on the server.
//!
//! All interaction with the mock server happens through the provided library. Therefore, you do
//! not need to interact with the mock server directly.
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
//! httpmock = "0.3.5"
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
//! and cleaning up old mock server state, so that each test can start with
//! a clean server. The annotation also sequentializes tests, so
//! they do not conflict with each other when using the mock server.
//!
//! If you try to create a mock without having annotated your test function
//! with the [with_mock_server](../httpmock_macros/attr.with_mock_server.html) annotation,
//! you will receive a panic at runtime pointing you to this problem.
//!
//! # Usage
//! Interaction with the mock server happens via the [Mock](struct.Mock.html) structure.
//! It provides you all mocking functionality that is supported by the mock server.
//!
//! The expected style of usage is as follows:
//! * Create a [Mock](struct.Mock.html) object using the
//! [Mock::create](struct.Mock.html#method.create) method
//! (or [Mock::new](struct.Mock.html#method.new) for slightly more control).
//! * Set your mock requirements using the provided `expect`-methods, such as
//! [expect_header](struct.Mock.html#method.expect_header),
//! [expect_body](struct.Mock.html#method.expect_body), etc.
//! These methods describe what attributes an HTTP request needs to have
//! to be considered a "match" for the mock you are creating.
//!
//! * use the provided `return`-methods to describe what the mock server should return when it
//! receives an HTTP request that matches all mock requirements. Some example `return`-methods
//! are [return_status](struct.Mock.html#method.return_status) and
//! [return_body](struct.Mock.html#method.return_body). If the server does not find any matching
//! mocks for an incoming HTTP request, it will return a response with an empty body and HTTP
//! status code 500.
//! * create the mock using the [Mock::create](struct.Mock.html#method.create) method. If you do
//! not call this method when you are finished configuring it, it will not be created at the mock
//! server and your test will not receive the expected response.
//! * using the mock object returned by the [Mock::create](struct.Mock.html#method.create) method
//! to assert that a mock has been called by your code under test (please refer to any example).
//!
//! # Responses
//! For any HTTP request sent to the mock server by your application, the request is only
//! considered to match a mock if it fulfills all of the mocks request requirements.
//! If a request does not match any mock, the server will respond with an empty response body
//! and an HTTP status code 500 (Internal Server Error).
//!
//! # Examples
//! Fore more examples, please refer to
//! [this crates test directory](https://github.com/alexliesenfeld/httpmock/blob/master/tests/integration_tests.rs ).
//!
//! # Debugging
//! `httpmock` logs against the `log` crate. For example, if you use the `env_logger` logging
//! backend, you can activate debug logging by setting `RUST_LOG` environment variable to `debug`
//! and then calling `env_logger::try_init()`:
//! ```rust
//! #[test]
//! #[with_mock_server]
//! fn your_test() {
//!     let _ = env_logger::try_init();
//!     // ...
//! }
//! ```
//! # Standalone Mode
//! You can use `httpmock` to provide a standalone mock server that is available to multiple
//! applications. This can be useful if you are running integration tests that involve
//! multiple applications and you want to mock only a subset of them.
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

use std::any::Any;
use std::borrow::BorrowMut;
use std::net::{SocketAddr, ToSocketAddrs};
use std::panic::catch_unwind;
use std::rc::Rc;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread::{sleep, JoinHandle};
use std::time::Duration;

use hyper::Body;
use std::thread;

use crate::api::{LocalMockServerAdapter, MockServerAdapter, RemoteMockServerAdapter};
pub use crate::api::{Method, Mock, Regex};
use crate::pool::ItemPool;
use crate::server::data::{MockServerHttpRequest, MockServerState};
use crate::server::{start_server, HttpMockConfig};
use crate::util::{read_env, with_retry, MaxPassLatch};
use futures::executor::block_on;
use util::Join;

use crossbeam_utils::sync::{Parker, Unparker};
use futures_util::{pin_mut, task::ArcWake};
use std::{
    future::Future,
    net::{UdpSocket},

    task::{Context, Poll, Waker},
};


mod api;
pub mod pool;
mod server;
mod util;
use tokio::task::LocalSet;
use isahc::prelude::Configurable;

pub type MockServerRequest = Rc<MockServerHttpRequest>;

pub struct MockServer {
    server_adapter: Arc<Arc<dyn MockServerAdapter + Send + Sync>>,
}

impl MockServer {
    fn from(server_adapter: Arc<Arc<dyn MockServerAdapter + Send + Sync>>) -> Self {
        let client : Arc<Arc<isahc::HttpClient>>= LOCAL_CLIENT_POOL.get_or_create_from(|| {
            return Arc::new(isahc::HttpClientBuilder::new()
                .tcp_keepalive(Duration::from_secs(60 * 60 * 24 * 256))
                .build().expect("Cannot build client"));
        }).join();

        // TODO: No pool but exactly one fixed client per local server
        // TODO: for remote servers, it should be a pool of clients for one remote server
        client.delete(&format!("http://{}/__mocks", server_adapter.address()))
            .expect("Cannot contact HTTP server");

        MockServer { server_adapter }
    }

    pub fn new_remote_from_address(addr: SocketAddr) -> Self {
        return MockServer::from(Arc::new(Arc::new(RemoteMockServerAdapter::new(addr))));
    }

    pub fn new_remote() -> Self {
        let host = read_env("HTTPMOCK_HOST", "127.0.0.1");
        let port = read_env("HTTPMOCK_PORT", "5000")
            .parse::<u16>()
            .expect("Cannot parse port from environment variable HTTPMOCK_PORT");

        let addr = format!("{}:{}", host, port)
            .to_socket_addrs()
            .expect("Cannot parse mock server address")
            .next()
            .expect("Cannot find mock server address in user input");

        return MockServer::from(Arc::new(Arc::new(RemoteMockServerAdapter::new(addr))));
    }

    pub fn new() -> Self {
        let adapter = LOCAL_SERVER_POOL.get_or_create_from(LOCAL_SERVER_ADAPTER_GENERATOR).join();
        MockServer::from(adapter)
    }

    pub fn new_mock(&self) -> Mock {
        Mock::new(self.server_adapter.clone())
    }

    pub fn mock(&self, method: Method, path: &str) -> Mock {
        Mock::new(self.server_adapter.clone())
            .expect_method(method)
            .expect_path(path)
    }

    pub fn host(&self) -> String {
        self.server_adapter.host()
    }

    pub fn port(&self) -> u16 {
        self.server_adapter.port()
    }

    pub fn address(&self) -> &SocketAddr {
        self.server_adapter.address()
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        LOCAL_SERVER_POOL.put_back(self.server_adapter.clone()).join();
    }
}

const LOCAL_SERVER_ADAPTER_GENERATOR: fn() -> Arc<dyn MockServerAdapter + Send + Sync> = || {
    let (addr_sender, addr_receiver) = tokio::sync::oneshot::channel::<SocketAddr>();
    let state = Arc::new(MockServerState::new());
    let server_state = state.clone();

    thread::spawn(move || {
        let config = HttpMockConfig::new(0, 1, false);
        let server_state = server_state.clone();

        let srv = start_server(config, &server_state, None, Some(addr_sender));

        let mut runtime = tokio::runtime::Builder::new()
            .enable_all()
            .basic_scheduler()
            .build()
            .expect("Cannot build local tokio runtime");

        return LocalSet::new().block_on(&mut runtime, srv);
    });

    let addr = block_on(addr_receiver).expect("Cannot get server address");
    return Arc::new(LocalMockServerAdapter::new(addr, state));
};

lazy_static! {
    static ref LOCAL_SERVER_POOL: Arc<ItemPool<Arc<dyn MockServerAdapter + Send + Sync>>> = {
        let max_servers = read_env("HTTPMOCK_MAX_SERVERS", "30")
            .parse::<usize>()
            .expect("Cannot parse environment variable HTTPMOCK_MAX_SERVERS to an integer");
        return Arc::new(ItemPool::<Arc<dyn MockServerAdapter + Send + Sync>>::new(
            max_servers,
        ));
    };
    static ref LOCAL_CLIENT_POOL: Arc<ItemPool<Arc<isahc::HttpClient>>> = {
        let max_clients = read_env("HTTPMOCK_MAX_LOCAL_CLIENTS", "30")
            .parse::<usize>()
            .expect("Cannot parse environment variable HTTPMOCK_MAX_LOCAL_CLIENTS to an integer");
        return Arc::new(ItemPool::<Arc<isahc::HttpClient>>::new(max_clients));
    };
}

