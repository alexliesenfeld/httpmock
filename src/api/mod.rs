// TODO: Remove this at some point
#![allow(clippy::needless_lifetimes)]
pub use adapter::{
    local::LocalMockServerAdapter, standalone::RemoteMockServerAdapter, Method, MockServerAdapter,
    Regex,
};
pub use mock::{Mock, MockRef};

mod adapter;
mod mock;
