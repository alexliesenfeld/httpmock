#![allow(warnings)]
//! HTTP mocking library that allows you to simulate responses from HTTP based services.
//!
//! # Features
//! * Simple, expressive, fluent API.
//! * Many built-in helpers for easy request matching.
//! * Parallel test execution.
//! * Extensible request matching.
//! * Fully asynchronous core with synchronous and asynchronous APIs.
//! * [Advanced verification and debugging support](https://web.archive.org/web/20201202160613/https://dev.to/alexliesenfeld/rust-http-testing-with-httpmock-2mi0#verification)
//! * [Network delay simulation](https://github.com/alexliesenfeld/httpmock/blob/master/tests/examples/delay_tests.rs).
//! * Support for [Regex](https://docs.rs/regex/) matching, JSON, [serde](https://crates.io/crates/serde), cookies, and more.
//! * Standalone mode with an accompanying [Docker image](https://hub.docker.com/r/alexliesenfeld/httpmock).
//! * Support for [mock specification based on YAML files](https://github.com/alexliesenfeld/httpmock/blob/master/src/lib.rs#L185-L201).
//!
//! # Getting Started
//! Add `httpmock` to `Cargo.toml`:
//!
//! ```toml
//! [dev-dependencies]
//! httpmock = "0.7.0"
//! ```
//!
//! You can then use `httpmock` as follows:
//! ```
//! use httpmock::prelude::*;
//!
//! // Start a lightweight mock server.
//! let server = MockServer::start();
//!
//! // Create a mock on the server.
//! let hello_mock = server.mock(|when, then| {
//!     when.method(GET)
//!         .path("/translate")
//!         .query_param("word", "hello");
//!     then.status(200)
//!         .header("content-type", "text/html")
//!         .body("ohi");
//! });
//!
//! // Send an HTTP request to the mock server. This simulates your code.
//! let response = isahc::get(server.url("/translate?word=hello")).unwrap();
//!
//! // Ensure the specified mock was called exactly one time (or fail with a detailed error description).
//! hello_mock.assert();
//! // Ensure the mock server did respond as specified.
//! assert_eq!(response.status(), 200);
//! ```
//!
//! In case the request fails, `httpmock` would show you a detailed error description including a diff between the
//! expected and the actual HTTP request:
//!
//! ![colored-diff.png](https://raw.githubusercontent.com/alexliesenfeld/httpmock/master/docs/diff.png)
//!
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
//! [MockServer](struct.MockServer.html)s are never recreated but recycled/reset.
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
//! let server = httpmock::MockServer::start();
//!
//! let greeting_mock = server.mock(|when, then| {
//!     when.path("/hi");
//!     then.status(200);
//! });
//!
//! let response = isahc::get(server.url("/hi")).unwrap();
//!
//! greeting_mock.assert();
//! ```
//! Note that `when` and `then` are variables. This allows you to rename them to something you
//! like better (such as `expect`/`respond_with`).
//!
//! Relevant elements for this API are [MockServer::mock](struct.MockServer.html#method.mock), [When](struct.When.html) and [Then](struct.Then.html).
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
//! [MockServer::connect_from_env](struct.MockServer.html#method.connect_from_env)). **Note**:
//! These are only available with the `remote` feature **enabled**.
//!
//! ```
//! use httpmock::prelude::*;
//! use isahc::get;
//!
//! #[test]
//! fn simple_test() {
//!     // Arrange
//!     let server = MockServer::connect("some-host:5000");
//!
//!     let hello_mock = server.mock(|when, then|{
//!         when.path("/hello/standalone");
//!         then.status(200);
//!     });
//!
//!     // Act
//!     let response = get(server.url("/hello/standalone")).unwrap();
//!
//!     // Assert
//!     hello_mock.assert();
//!     assert_eq!(response.status(), 200);
//! }
//! ```
//!
//! ## Standalone Parallelism
//! To prevent interference with other tests, test functions are forced to use the standalone
//! mock server sequentially.
//! This means that test functions may be blocked when connecting to the remote server until
//! it becomes free again.
//! This is in contrast to tests that use a local mock server.
//!
//! ## Limitations of the Standalone Mode
//! At this time, it is not possible to use custom request matchers in combination with standalone
//! mock servers (see [When::matches](struct.When.html#method.matches) or
//! [Mock::expect_match](struct.Mock.html#method.expect_match)).
//!
//! ## Standalone Mode with YAML Mock Definition Files
//! The standalone server can also be used to read mock definitions from YAML files on startup once
//! and serve the mocked endpoints until the server is shut down again. These `static` mocks
//! cannot be deleted at runtime (even by Rust-based tests that use the mock server) and exist
//! for the entire uptime of the mock server.
//!
//! The definition files follow the standard `httpmock` API that you would also use in regular
//! Rust tests. Please find an example mock definition file in the `httpmock` Github repository
//! [here in this crates test directory](https://github.com/alexliesenfeld/httpmock/blob/master/tests/resources/static_yaml_mock.yaml).
//!
//! You can start the mock server with static mock support as follows:
//! * If you use the [Docker image from this creates repository](https://github.com/alexliesenfeld/httpmock/blob/master/Dockerfile)
//! or from [Dockerhub](https://hub.docker.com/r/alexliesenfeld/httpmock), you just need to mount a
//! volume with all your mock specification files to the `/mocks` directory within the container.
//! * If you build `httpmock` from source and use the binary, then you can pass the path to
//! the directory containing all your mock specification files using the `--static-mock-dir`
//! parameter. Example: `httpmock --expose --static-mock-dir=/mocks`.
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

use std::borrow::BorrowMut;
use std::net::ToSocketAddrs;

use std::str::FromStr;

use serde::{Deserialize, Serialize};

use api::MockServerAdapter;
use common::util::Join;

pub use api::{Method, Mock, MockExt, MockServer, Regex, Then, When};

mod api;
mod common;
mod server;
pub mod standalone;

pub mod prelude {
    #[doc(no_inline)]
    pub use crate::{
        api::MockServer, common::data::HttpMockRequest, Method::DELETE, Method::GET,
        Method::OPTIONS, Method::POST, Method::PUT, Regex,
    };
}
