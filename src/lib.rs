#![allow(warnings)]
//! HTTP mocking library that allows you to simulate responses from HTTP based services.
//!
//! # Features
//! * Simple, expressive, fluent API.
//! * Many built-in helpers for easy request matching ([Regex](https://docs.rs/regex/), JSON, [serde](https://crates.io/crates/serde), cookies, and more).
//! * Parallel test execution.
//! * Custom request matchers.
//! * Record and Playback
//! * Forward and Proxy Mode
//! * HTTPS support
//! * Fault and network delay simulation.
//! * Standalone mode with an accompanying [Docker image](https://hub.docker.com/r/alexliesenfeld/httpmock).
//! * Helpful error messages
//! * [Advanced verification and debugging support](https://alexliesenfeld.github.io/posts/mocking-http--services-in-rust/#creating-mocks) (including diff generation between actual and expected HTTP request values)
//! * Fully asynchronous core with synchronous and asynchronous APIs.
//! * Support for [Regex](https://docs.rs/regex/) matching, JSON, [serde](https://crates.io/crates/serde), cookies, and more.
//! * Support for [mock configuration using YAML files](https://github.com/alexliesenfeld/httpmock/tree/master#file-based-mock-specification).
//!
//! # Getting Started
//! Add `httpmock` to `Cargo.toml`:
//!
//! ```toml
//! [dev-dependencies]
//! httpmock = "0.8.0-alpha.1"
//! ```
//!
//! You can then use `httpmock` as follows:
//! ```
//! use httpmock::prelude::*;
//! use reqwest::blocking::get;
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
//! let response = get(&server.url("/translate?word=hello")).unwrap();
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
//! # Online Documentation
//! Please find the official `httpmock` documentation and website at: http://alexliesenfeld.github.io/httpmock
//!
//! # License
//! `httpmock` is free software: you can redistribute it and/or modify it under the terms
//! of the MIT Public License.
//!
//! This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
//! without even the implied
//! warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the MIT Public
//! License for more details.
extern crate lazy_static;

use std::{borrow::BorrowMut, net::ToSocketAddrs};

use std::str::FromStr;

use serde::{Deserialize, Serialize};

use api::MockServerAdapter;
use common::util::Join;

pub use api::{Method, Mock, MockExt, MockServer, Regex, Then, When};

mod api;
pub mod common;
pub mod server;

#[cfg(feature = "record")]
pub use common::data::RecordingRuleConfig;

#[cfg(feature = "proxy")]
pub use common::data::{ForwardingRuleConfig, ProxyRuleConfig};

pub mod prelude {
    #[doc(no_inline)]
    pub use crate::{
        api::MockServer, common::data::HttpMockRequest, Method, Method::DELETE, Method::GET,
        Method::OPTIONS, Method::PATCH, Method::POST, Method::PUT, Regex,
    };
}
