#![allow(clippy::trivial_regex)]

use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use hyper::body::Buf;
use hyper::header::HeaderValue;
use hyper::http::header::HeaderName;
use hyper::service::{make_service_fn, service_fn};
use hyper::{
    Body, HeaderMap, Request as HyperRequest, Response as HyperResponse, Result as HyperResult,
    Server, StatusCode,
};
use regex::Regex;

pub use crate::server::data::MockServerState;

pub(crate) mod data;
pub(crate) mod handlers;

mod routes;
mod util;

type GenericError = Box<dyn std::error::Error + Send + Sync>;

/// Holds server configuration properties.
#[derive(Debug)]
pub struct HttpMockConfig {
    pub port: u16,
    pub expose: bool,
}

impl HttpMockConfig {
    pub fn new(port: u16, expose: bool) -> Self {
        Self { port, expose }
    }
}

#[derive(Default, Debug)]
pub(crate) struct ServerRequestHeader {
    pub method: String,
    pub path: String,
    pub query: String,
    pub headers: BTreeMap<String, String>,
}

impl ServerRequestHeader {
    pub fn from(req: &HyperRequest<Body>) -> Result<ServerRequestHeader, String> {
        let headers = extract_headers(req.headers());
        if let Err(e) = headers {
            return Err(format!("error parsing headers: {}", e));
        }

        let method = req.method().as_str().to_string();
        let path = req.uri().path().to_string();
        let query = req.uri().query().unwrap_or("").to_string();
        let headers = headers.unwrap();

        let server_request = ServerRequestHeader::new(method, path, query, headers);

        Ok(server_request)
    }

    pub fn new(
        method: String,
        path: String,
        query: String,
        headers: BTreeMap<String, String>,
    ) -> Self {
        Self {
            method,
            path,
            query,
            headers,
        }
    }
}

#[derive(Default, Debug)]
pub(crate) struct ServerResponse {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body: String,
}

impl ServerResponse {
    pub fn new(status: u16, headers: BTreeMap<String, String>, body: String) -> Self {
        Self {
            status,
            headers,
            body,
        }
    }
}

/// Extracts all headers from the URI of the given request.
fn extract_headers(header_map: &HeaderMap) -> Result<BTreeMap<String, String>, String> {
    let mut headers = BTreeMap::new();
    for (hn, hv) in header_map {
        let hn = hn.as_str().to_string();
        let hv = hv.to_str();
        if let Err(e) = hv {
            return Err(format!("error parsing headers: {}", e));
        }
        headers.insert(hn, hv.unwrap().to_string());
    }
    Ok(headers)
}

async fn handle_server_request(
    req: HyperRequest<Body>,
    state: Arc<MockServerState>,
) -> HyperResult<HyperResponse<Body>> {
    let request_header = ServerRequestHeader::from(&req);

    if let Err(e) = request_header {
        return Ok(error_response(format!("Cannot parse request: {}", e)));
    }

    let entire_body = hyper::body::aggregate(req.into_body()).await;
    if let Err(e) = entire_body {
        return Ok(error_response(format!("Cannot read request body: {}", e)));
    }

    let the_body = entire_body.unwrap();
    let body_bytes = the_body.bytes();
    let body = String::from_utf8(body_bytes.to_vec());

    if let Err(e) = body {
        return Ok(error_response(format!("Cannot read body: {}", e)));
    }

    let routing_result = route_request(state.borrow(), &request_header.unwrap(), body.unwrap()).await;
    if let Err(e) = routing_result {
        return Ok(error_response(format!("Request handler error: {}", e)));
    }

    let response = map_response(routing_result.unwrap());
    if let Err(e) = response {
        return Ok(error_response(format!("Cannot build response: {}", e)));
    }

    Ok(response.unwrap())
}

/// Starts a new instance of an HTTP mock server. You should never need to use this function
/// directly. Use it if you absolutely need to manage the low-level details of how the mock
/// server operates.
pub async fn start_server(
    http_mock_config: HttpMockConfig,
    state: &Arc<MockServerState>,
    socket_addr_sender: Option<tokio::sync::oneshot::Sender<SocketAddr>>,
) -> Result<(), String> {
    let port = http_mock_config.port;
    let host = if http_mock_config.expose {
        "0.0.0.0"
    } else {
        "127.0.0.1"
    };

    let state = state.clone();
    let new_service = make_service_fn(move |_| {
        let state = state.clone();
        async move {
            Ok::<_, GenericError>(service_fn(move |req: HyperRequest<Body>| {
                let state = state.clone();
                handle_server_request(req, state)
            }))
        }
    });

    let server = Server::bind(&format!("{}:{}", host, port).parse().unwrap()).serve(new_service);

    if let Some(socket_addr_sender) = socket_addr_sender {
        if let Err(e) = socket_addr_sender.send(server.local_addr()) {
            return Err(format!(
                "Cannot send socket information to the test thread: {:?}",
                e
            ));
        }
    }

    log::info!("Listening on {}", server.local_addr());
    if let Err(e) = server.await {
        return Err(format!("Err: {}", e));
    }

    Ok(())
}

/// Maps a server response to a hyper response.
fn map_response(route_response: ServerResponse) -> Result<HyperResponse<Body>, String> {
    let mut builder = HyperResponse::builder();
    builder = builder.status(route_response.status);

    for (key, value) in route_response.headers {
        let name = HeaderName::from_str(&key);
        if let Err(e) = name {
            return Err(format!("Cannot create header from name: {}", e));
        }

        let value = HeaderValue::from_str(&value);
        if let Err(e) = value {
            return Err(format!("Cannot create header from value: {}", e));
        }

        let value = value.unwrap();
        let value = value.to_str();
        if let Err(e) = value {
            return Err(format!("Cannot create header from value string: {}", e));
        }

        builder = builder.header(name.unwrap(), value.unwrap());
    }

    let result = builder.body(Body::from(route_response.body));
    if let Err(e) = result {
        return Err(format!("Cannot create HTTP response: {}", e));
    }

    Ok(result.unwrap())
}

/// Routes a request to the appropriate route handler.
async fn route_request(
    state: &MockServerState,
    request_header: &ServerRequestHeader,
    body: String,
) -> Result<ServerResponse, String> {
    log::trace!("Routing incoming request: {:?}", request_header);

    if PING_PATH.is_match(&request_header.path) {
        if let "GET" = request_header.method.as_str() {
            return routes::ping();
        }
    }

    if MOCKS_PATH.is_match(&request_header.path) {
        match request_header.method.as_str() {
            "POST" => return routes::add(state, body),
            "DELETE" => return routes::delete_all(state),
            _ => {}
        }
    }

    if MOCK_PATH.is_match(&request_header.path) {
        let id = get_path_param(&MOCK_PATH, 1, &request_header.path);
        if let Err(e) = id {
            return Err(format!("Cannot parse id from path: {}", e));
        }
        let id = id.unwrap();

        match request_header.method.as_str() {
            "GET" => return routes::read_one(state, id),
            "DELETE" => return routes::delete_one(state, id),
            _ => {}
        }
    }

    routes::serve(state, request_header, body).await
}

/// Get request path parameters.
fn get_path_param(regex: &Regex, idx: usize, path: &str) -> Result<usize, String> {
    let cap = regex.captures(path);
    if cap.is_none() {
        return Err(format!(
            "Error capturing parameter from request path: {}",
            path
        ));
    }
    let cap = cap.unwrap();

    let id = cap.get(idx);
    if id.is_none() {
        return Err(format!(
            "Error capturing resource id in request path: {}",
            path
        ));
    }
    let id = id.unwrap().as_str();

    let id = id.parse::<usize>();
    if let Err(e) = id {
        return Err(format!("Error parsing id as a number: {}", e));
    }
    let id = id.unwrap();

    Ok(id)
}

/// Creates a default error response.
fn error_response(body: String) -> HyperResponse<Body> {
    HyperResponse::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(body))
        .expect("Cannot build route error response")
}

lazy_static! {
    static ref PING_PATH: Regex = Regex::new(r"^/__ping$").unwrap();
    static ref MOCKS_PATH: Regex = Regex::new(r"^/__mocks$").unwrap();
    static ref MOCK_PATH: Regex = Regex::new(r"^/__mocks/([0-9]+)$").unwrap();
}

#[cfg(test)]
mod test {
    use crate::server::{
        error_response, get_path_param, map_response, ServerResponse, MOCKS_PATH, MOCK_PATH,
    };
    use crate::Regex;
    use futures_util::TryStreamExt;
    use hyper::Body;
    use std::borrow::Borrow;
    use std::collections::BTreeMap;

    #[test]
    fn route_regex_test() {
        assert_eq!(MOCK_PATH.is_match("/__mocks/1"), true);
        assert_eq!(MOCK_PATH.is_match("/__mocks/1295473892374"), true);
        assert_eq!(MOCK_PATH.is_match("/__mocks/abc"), false);
        assert_eq!(MOCK_PATH.is_match("/__mocks"), false);
        assert_eq!(MOCK_PATH.is_match("/__mocks/345345/test"), false);
        assert_eq!(MOCK_PATH.is_match("test/__mocks/345345/test"), false);

        assert_eq!(MOCKS_PATH.is_match("/__mocks"), true);
        assert_eq!(MOCKS_PATH.is_match("/__mocks/5"), false);
        assert_eq!(MOCKS_PATH.is_match("test/__mocks/5"), false);
        assert_eq!(MOCKS_PATH.is_match("test/__mocks/567"), false);
    }

    /// Make sure passing an empty string to the error response does not result in an error.
    #[test]
    fn error_response_test() {
        let res = error_response("test".into());
        let (parts, body) = res.into_parts();

        let body = async_std::task::block_on({
            body.try_fold(Vec::new(), |mut data, chunk| async move {
                data.extend_from_slice(&chunk);
                Ok(data)
            })
        });

        assert_eq!(
            String::from_utf8(body.unwrap()).unwrap(),
            "test".to_string()
        )
    }

    /// Makes sure an error is return if there is a header parsing error
    #[test]
    fn response_header_key_parsing_error_test() {
        // Arrange
        let mut headers = BTreeMap::new();
        headers.insert(";;;".to_string(), ";;;".to_string());

        let res = ServerResponse {
            body: "".to_string(),
            status: 500,
            headers,
        };

        // Act
        let result = map_response(res);

        // Assert
        assert_eq!(result.is_err(), true);
        assert_eq!(
            result
                .err()
                .unwrap()
                .contains("Cannot create header from name"),
            true
        );
    }

    #[test]
    fn get_path_param_regex_error_test() {
        // Arrange
        let re = Regex::new(r"^/__mocks/([0-9]+)$").unwrap();

        // Act
        let result = get_path_param(&re, 0, "");

        // Assert
        assert_eq!(result.is_err(), true);
        assert_eq!(
            result
                .err()
                .unwrap()
                .contains("Error capturing parameter from request path"),
            true
        );
    }

    #[test]
    fn get_path_param_index_error_test() {
        // Arrange
        let re = Regex::new(r"^/__mocks/([0-9]+)$").unwrap();

        // Act
        let result = get_path_param(&re, 5, "/__mocks/5");

        // Assert
        assert_eq!(result.is_err(), true);
        assert_eq!(
            "Error capturing resource id in request path: /__mocks/5",
            result.err().unwrap()
        );
    }

    #[test]
    fn get_path_param_number_error_test() {
        // Arrange
        let re = Regex::new(r"^/__mocks/([0-9]+)$").unwrap();

        // Act
        let result = get_path_param(&re, 0, "/__mocks/9999999999999999999999999");

        // Assert
        assert_eq!(result.is_err(), true);
        assert_eq!(
            "Error parsing id as a number: invalid digit found in string",
            result.err().unwrap()
        );
    }
}
