// TODO: Remove this at some point
#![allow(clippy::needless_lifetimes)]

pub use adapter::{local::LocalMockServerAdapter, Method, MockServerAdapter, Regex};

#[cfg(feature = "remote")]
pub use adapter::standalone::RemoteMockServerAdapter;

pub use mock::{Mock, MockExt};
pub use server::MockServer;
pub use spec::{Then, When};

mod adapter;
mod mock;
mod server;
pub mod spec;
