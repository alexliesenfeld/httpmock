// TODO: Remove this
#![allow(clippy::needless_lifetimes)]

mod adapter;
mod mock;

pub(crate) use adapter::{LocalMockServerAdapter, MockServerAdapter, RemoteMockServerAdapter};
pub use adapter::{Method, Regex};
pub use {mock::Mock, mock::MockRef};
