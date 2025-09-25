#![allow(clippy::trivial_regex)]
use std::{borrow::Borrow, str::FromStr};

use crate::server::matchers::Matcher;
use bytes::Bytes;
use futures_util::task::Spawn;
use hyper::body::{Body, Buf};
use std::{future::Future, net::SocketAddr};

use futures_util::{FutureExt, TryStreamExt};
use http_body_util::BodyExt;

mod builder;
mod handler;
pub mod matchers;
mod server;
pub mod state;

#[cfg(feature = "record")]
mod persistence;

#[cfg(feature = "https")]
mod tls;

use crate::server::{handler::HttpMockHandler, server::MockServer, state::HttpMockStateManager};

pub use builder::HttpMockServerBuilder;
pub use server::Error;

// We want to expose this error to the user
pub type HttpMockServer = MockServer<HttpMockHandler<HttpMockStateManager>>;

/// Per-request metadata propagated through Hyper services.
#[derive(Clone)]
pub struct RequestMetadata {
    /// The scheme ("http" or "https") associated with this request, used by the
    /// upstream client to reconstruct the absolute target when needed.
    pub scheme: &'static str,
}

impl RequestMetadata {
    /// Create new RequestMetadata for a request with the given scheme.
    pub fn new(scheme: &'static str) -> Self {
        Self { scheme }
    }
}
