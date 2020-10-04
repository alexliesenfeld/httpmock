#![allow(clippy::trivial_regex)]

use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

use hyper::body::Buf;
use hyper::header::HeaderValue;
use hyper::http::header::HeaderName;
use hyper::service::{make_service_fn, service_fn};
use hyper::{
    Body, HeaderMap, Request as HyperRequest, Response as HyperResponse, Result as HyperResult,
    Server, StatusCode,
};
use regex::Regex;

use crate::data::{ActiveMock, HttpMockRequest};
use crate::server::matchers::Matcher;
use crate::server::web::routes;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;

mod matchers;

mod util;
pub(crate) mod web;

pub(crate) use matchers::{Diff, Mismatch, Tokenizer};
use crate::server::matchers::generic::single_value::SingleValueMatcher;
use crate::server::matchers::comparators::{StringContainsMatchComparator, StringExactMatchComparator, StringRegexMatchComparator, JSONContainsMatchComparator, JSONExactMatchComparator};
use matchers::targets::{StringBodyTarget, JSONBodyTarget};
use crate::server::matchers::concrete::path_matcher::PathMatcher;
use crate::server::matchers::concrete::path_contains_matcher::PathContainsMatcher;
use crate::server::matchers::concrete::path_regex_matcher::PathRegexMatcher;
use crate::server::matchers::concrete::method_matcher::MethodMatcher;
use crate::server::matchers::concrete::cookie_exists_matcher::CookieExistsMatcher;
use crate::server::matchers::concrete::header_matcher::HeaderMatcher;
use crate::server::matchers::concrete::header_exists_matcher::HeaderExistsMatcher;
use crate::server::matchers::concrete::query_parameter_exists_matcher::QueryParameterExistsMatcher;
use crate::server::matchers::concrete::query_parameter_matcher::QueryParameterMatcher;
use crate::server::matchers::concrete::custom_function_matcher::CustomFunctionMatcher;
use crate::server::matchers::sources::{StringBodySource, BodyRegexSource, PartialJSONBodySource, JSONBodySource};

pub(crate) struct Matchers {
    pub custom_function_matchers: Vec<Box<dyn Matcher + Sync + Send>>,
    pub path_matchers: Vec<Box<dyn Matcher + Sync + Send>>,
    pub method_matchers: Vec<Box<dyn Matcher + Sync + Send>>,
    pub headers_matchers: Vec<Box<dyn Matcher + Sync + Send>>,
    pub query_params_matchers: Vec<Box<dyn Matcher + Sync + Send>>,
    pub cookie_matchers: Vec<Box<dyn Matcher + Sync + Send>>,
    pub body_matchers: Vec<Box<dyn Matcher + Sync + Send>>,
    //pub multipart_matchers: Vec<Box<dyn Matcher + Sync + Send>>,
}

impl Matchers {
    fn all(&self) -> Vec<&Box<dyn Matcher + Sync + Send>> {
        let all = vec![
            &self.custom_function_matchers,
            &self.path_matchers,
            &self.method_matchers,
            &self.headers_matchers,
            &self.query_params_matchers,
            &self.cookie_matchers,
            &self.body_matchers,
        ];

        all.into_iter().flatten().collect()
    }
}

/// The shared state accessible to all handlers
pub(crate) struct MockServerState {
    id_counter: AtomicUsize,
    pub mocks: RwLock<BTreeMap<usize, ActiveMock>>,
    pub history: RwLock<Vec<Arc<HttpMockRequest>>>,
    pub matchers: Matchers,
}

impl MockServerState {
    pub fn create_new_id(&self) -> usize {
        self.id_counter.fetch_add(1, Relaxed)
    }

    pub fn new() -> Self {
        MockServerState {
            mocks: RwLock::new(BTreeMap::new()),
            history: RwLock::new(Vec::new()),
            id_counter: AtomicUsize::new(0),
            matchers: Matchers {
                method_matchers: vec![Box::new(MethodMatcher::new(3.0))],
                path_matchers: vec![
                    Box::new(PathMatcher::new(10.0)),
                    Box::new(PathContainsMatcher::new(10.0)),
                    Box::new(PathRegexMatcher::new(10.0)),
                ],
                body_matchers: vec![
                    // string body exact
                    Box::new(SingleValueMatcher {
                        entity_name: "body",
                        comparator: Box::new(StringExactMatchComparator::new()),
                        source: Box::new(StringBodySource::new()),
                        target: Box::new(StringBodyTarget::new()),
                        encoder: None,
                        with_reason: true,
                        with_diff: true,
                    }),
                    // string body contains
                    Box::new(SingleValueMatcher {
                        entity_name: "body",
                        comparator: Box::new(StringContainsMatchComparator::new()),
                        source: Box::new(StringBodySource::new()),
                        target: Box::new(StringBodyTarget::new()),
                        encoder: None,
                        with_reason: true,
                        with_diff: true,
                    }),
                    // string body regex
                    Box::new(SingleValueMatcher {
                        entity_name: "body",
                        comparator: Box::new(StringRegexMatchComparator::new()),
                        source: Box::new(BodyRegexSource::new()),
                        target: Box::new(StringBodyTarget::new()),
                        encoder: None,
                        with_reason: true,
                        with_diff: true,
                    }),
                    // JSON body contains
                    Box::new(SingleValueMatcher {
                        entity_name: "body",
                        comparator: Box::new(JSONContainsMatchComparator::new()),
                        source: Box::new(PartialJSONBodySource::new()),
                        target: Box::new(JSONBodyTarget::new()),
                        encoder: None,
                        with_reason: true,
                        with_diff: true,
                    }),
                    // JSON body exact
                    Box::new(SingleValueMatcher {
                        entity_name: "body",
                        comparator: Box::new(JSONExactMatchComparator::new()),
                        source: Box::new(JSONBodySource::new()),
                        target: Box::new(JSONBodyTarget::new()),
                        encoder: None,
                        with_reason: true,
                        with_diff: true,
                    }),
                ],
                cookie_matchers: vec![
                    Box::new(CookieExistsMatcher::new(1.0)),
                    //Box::new(CookieMatcher::new(None, None)),
                ],
                headers_matchers: vec![
                    Box::new(HeaderMatcher::new(1.0)),
                    Box::new(HeaderExistsMatcher::new(1.0)),
                ],
                query_params_matchers: vec![
                    Box::new(QueryParameterExistsMatcher::new(1.0)),
                    Box::new(QueryParameterMatcher::new(1.0)),
                ],
                custom_function_matchers: vec![Box::new(CustomFunctionMatcher::new(1.0))],
            },
        }
    }
}

type GenericError = Box<dyn std::error::Error + Send + Sync>;

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
    pub body: Vec<u8>,
}

impl ServerResponse {
    pub fn new(status: u16, headers: BTreeMap<String, String>, body: Vec<u8>) -> Self {
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

    let routing_result =
        route_request(state.borrow(), &request_header.unwrap(), body.unwrap()).await;
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
pub(crate) async fn start_server(
    port: u16,
    expose: bool,
    state: &Arc<MockServerState>,
    socket_addr_sender: Option<tokio::sync::oneshot::Sender<SocketAddr>>,
) -> Result<(), String> {
    let host = if expose { "0.0.0.0" } else { "127.0.0.1" };

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
            "DELETE" => return routes::delete_all_mocks(state),
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

    if HISTORY_PATH.is_match(&request_header.path) {
        match request_header.method.as_str() {
            "DELETE" => return routes::delete_history(state),
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
    static ref HISTORY_PATH: Regex = Regex::new(r"^/__history$").unwrap();
}

#[cfg(test)]
mod test {
    use crate::server::{
        error_response, get_path_param, map_response, ServerResponse, MOCKS_PATH, MOCK_PATH,
    };
    use crate::Regex;
    use futures_util::TryStreamExt;
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
            body: Vec::new(),
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
