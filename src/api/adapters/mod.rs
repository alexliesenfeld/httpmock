pub mod http;
pub mod local;

pub use http::MockServerHttpAdapter;
pub use local::LocalMockServerAdapter;

/// Refer to [regex::Regex](../regex/struct.Regex.html).
pub type Regex = regex::Regex;

/// Represents an HTTP method.
#[derive(Debug)]
pub enum Method {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}
