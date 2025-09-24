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
///
/// When acting as an HTTPS MITM proxy, the client→proxy leg speaks origin-form
/// (e.g. "/" with a Host header). Internally we normalize to absolute-form for
/// matching/recording, but before sending upstream we convert back to origin-form.
/// The upstream Hyper client still needs to know where to dial. We therefore store
/// the original scheme ("http" or "https") here so the Http client can reconstruct
/// an absolute URI from Host + path when the request URI is origin-form.
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
