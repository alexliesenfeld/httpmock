// TODO: Remove this at some point
#![allow(clippy::needless_lifetimes)]

pub use adapter::{
    local::LocalMockServerAdapter, standalone::RemoteMockServerAdapter, Method, MockServerAdapter,
    Regex,
};

pub use mock::{MockRef, MockRefExt};
mod adapter;
mod mock;
