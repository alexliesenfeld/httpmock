// TODO: Remove this at some point
#![allow(clippy::needless_lifetimes)]

pub use adapter::{
    local::LocalMockServerAdapter, standalone::RemoteMockServerAdapter, Method, MockServerAdapter,
    Regex,
};
pub use encoding::{StringValue, URLEncodedExtension};
pub use mock::{Mock, MockRef, MockRefExt};

mod adapter;
mod encoding;
mod mock;
