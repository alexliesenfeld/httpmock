// TODO: Remove this at some point
#![allow(clippy::needless_lifetimes)]

pub use adapter::{
    local::LocalMockServerAdapter, standalone::RemoteMockServerAdapter, Method, MockServerAdapter,
    Regex,
};
pub use mock::{Mock, MockExt};
pub use server::MockServer;
pub use spec::{When, Then};

mod adapter;
mod mock;
mod server;
pub mod spec;
