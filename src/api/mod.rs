// TODO: Remove this
#![allow(clippy::needless_lifetimes)]

pub(crate) use adapter::{LocalMockServerAdapter, MockServerAdapter, RemoteMockServerAdapter};
pub use adapter::{Method, Regex};
pub use {mock::Mock, mock::MockRef};

mod adapter;
mod mock;
