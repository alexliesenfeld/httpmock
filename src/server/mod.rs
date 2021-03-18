#![allow(clippy::trivial_regex)]

use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, RwLock};

use hyper::body::Buf;
use hyper::header::HeaderValue;
use hyper::http::header::HeaderName;
use hyper::service::{make_service_fn, service_fn};
use hyper::{
    Body, HeaderMap, Request as HyperRequest, Response as HyperResponse, Result as HyperResult,
    Server, StatusCode,
};
use regex::internal::Input;
use regex::Regex;

use matchers::generic::SingleValueMatcher;
use matchers::targets::{JSONBodyTarget, StringBodyTarget};
pub use matchers::{Diff, DiffResult, Mismatch, Reason, Tokenizer};

use crate::data::{ActiveMock, HttpMockRequest};
use crate::server::matchers::comparators::{
    AnyValueComparator, FunctionMatchesRequestComparator, JSONContainsMatchComparator,
    JSONExactMatchComparator, StringContainsMatchComparator, StringExactMatchComparator,
    StringRegexMatchComparator,
};
use crate::server::matchers::generic::{FunctionValueMatcher, MultiValueMatcher};
use crate::server::matchers::sources::{BodyRegexSource, ContainsCookieSource, ContainsHeaderSource, ContainsQueryParameterSource, CookieSource, FunctionSource, HeaderSource, JSONBodySource, MethodSource, PartialJSONBodySource, PathContainsSubstringSource, PathRegexSource, QueryParameterSource, StringBodyContainsSource, StringBodySource, StringPathSource, QueryParameterUrlencodedSource};
use crate::server::matchers::targets::{
    CookieTarget, FullRequestTarget, HeaderTarget, MethodTarget, PathTarget, QueryParameterTarget,
};
use crate::server::matchers::Matcher;
use crate::server::web::routes;
use std::iter::Map;
use crate::server::matchers::transformers::DecodeURLEncodingValueTransformer;

mod matchers;

mod util;
pub(crate) mod web;

/// The shared state accessible to all handlers
pub struct MockServerState {
    id_counter: AtomicUsize,
    pub mocks: RwLock<BTreeMap<usize, ActiveMock>>,
    pub history: RwLock<Vec<Arc<HttpMockRequest>>>,
    pub matchers: Vec<Box<dyn Matcher + Sync + Send>>,
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
            matchers: vec![
                // path exact
                Box::new(SingleValueMatcher {
                    entity_name: "path",
                    comparator: Box::new(StringExactMatchComparator::new(false)),
                    source: Box::new(StringPathSource::new()),
                    target: Box::new(PathTarget::new()),
                    transformer: None,
                    with_reason: true,
                    diff_with: None,
                    weight: 10,
                }),
                // path contains
                Box::new(SingleValueMatcher {
                    entity_name: "path",
                    comparator: Box::new(StringContainsMatchComparator::new(true)),
                    source: Box::new(PathContainsSubstringSource::new()),
                    target: Box::new(PathTarget::new()),
                    transformer: None,
                    with_reason: true,
                    diff_with: None,
                    weight: 10,
                }),
                // path matches regex
                Box::new(SingleValueMatcher {
                    entity_name: "path",
                    comparator: Box::new(StringRegexMatchComparator::new()),
                    source: Box::new(PathRegexSource::new()),
                    target: Box::new(PathTarget::new()),
                    transformer: None,
                    with_reason: true,
                    diff_with: None,
                    weight: 10,
                }),
                // method exact
                Box::new(SingleValueMatcher {
                    entity_name: "method",
                    comparator: Box::new(StringExactMatchComparator::new(false)),
                    source: Box::new(MethodSource::new()),
                    target: Box::new(MethodTarget::new()),
                    transformer: None,
                    with_reason: true,
                    diff_with: None,
                    weight: 3,
                }),
                // Query Param exact
                Box::new(MultiValueMatcher {
                    entity_name: "query parameter",
                    key_comparator: Box::new(StringExactMatchComparator::new(true)),
                    value_comparator: Box::new(StringExactMatchComparator::new(true)),
                    key_transformer: None,
                    value_transformer: None,
                    source: Box::new(QueryParameterSource::new()),
                    target: Box::new(QueryParameterTarget::new()),
                    with_reason: true,
                    diff_with: None,
                    weight: 1,
                }),
                // Query Param exists
                Box::new(MultiValueMatcher {
                    entity_name: "query parameter",
                    key_comparator: Box::new(StringExactMatchComparator::new(true)),
                    value_comparator: Box::new(AnyValueComparator::new()),
                    key_transformer: None,
                    value_transformer: None,
                    source: Box::new(ContainsQueryParameterSource::new()),
                    target: Box::new(QueryParameterTarget::new()),
                    with_reason: true,
                    diff_with: None,
                    weight: 1,
                }),
                // Query Param urlencoded exact
                Box::new(MultiValueMatcher {
                    entity_name: "query parameter (urlencoded)",
                    key_comparator: Box::new(StringExactMatchComparator::new(true)),
                    value_comparator: Box::new(StringExactMatchComparator::new(true)),
                    key_transformer: Some(Box::new(DecodeURLEncodingValueTransformer::new())),
                    value_transformer: Some(Box::new(DecodeURLEncodingValueTransformer::new())),
                    source: Box::new(QueryParameterUrlencodedSource::new()),
                    target: Box::new(QueryParameterTarget::new()),
                    with_reason: true,
                    diff_with: None,
                    weight: 1,
                }),
                // Cookie exact
                Box::new(MultiValueMatcher {
                    entity_name: "cookie",
                    key_comparator: Box::new(StringExactMatchComparator::new(true)),
                    value_comparator: Box::new(StringExactMatchComparator::new(true)),
                    key_transformer: None,
                    value_transformer: None,
                    source: Box::new(CookieSource::new()),
                    target: Box::new(CookieTarget::new()),
                    with_reason: true,
                    diff_with: None,
                    weight: 1,
                }),
                // Cookie exists
                Box::new(MultiValueMatcher {
                    entity_name: "cookie",
                    key_comparator: Box::new(StringExactMatchComparator::new(true)),
                    value_comparator: Box::new(AnyValueComparator::new()),
                    key_transformer: None,
                    value_transformer: None,
                    source: Box::new(ContainsCookieSource::new()),
                    target: Box::new(CookieTarget::new()),
                    with_reason: true,
                    diff_with: None,
                    weight: 1,
                }),
                // Header exact
                Box::new(MultiValueMatcher {
                    entity_name: "header",
                    key_comparator: Box::new(StringExactMatchComparator::new(false)),
                    value_comparator: Box::new(StringExactMatchComparator::new(true)),
                    key_transformer: None,
                    value_transformer: None,
                    source: Box::new(HeaderSource::new()),
                    target: Box::new(HeaderTarget::new()),
                    with_reason: true,
                    diff_with: None,
                    weight: 1,
                }),
                // Header exists
                Box::new(MultiValueMatcher {
                    entity_name: "header",
                    key_comparator: Box::new(StringExactMatchComparator::new(false)),
                    value_comparator: Box::new(AnyValueComparator::new()),
                    key_transformer: None,
                    value_transformer: None,
                    source: Box::new(ContainsHeaderSource::new()),
                    target: Box::new(HeaderTarget::new()),
                    with_reason: true,
                    diff_with: None,
                    weight: 1,
                }),
                // Box::new(CustomFunctionMatcher::new(1.0)),
                // string body exact
                Box::new(SingleValueMatcher {
                    entity_name: "body",
                    comparator: Box::new(StringExactMatchComparator::new(false)),
                    source: Box::new(StringBodySource::new()),
                    target: Box::new(StringBodyTarget::new()),
                    transformer: None,
                    with_reason: false,
                    diff_with: Some(Tokenizer::Line),
                    weight: 1,
                }),
                // string body contains
                Box::new(SingleValueMatcher {
                    entity_name: "body",
                    comparator: Box::new(StringContainsMatchComparator::new(true)),
                    source: Box::new(StringBodyContainsSource::new()),
                    target: Box::new(StringBodyTarget::new()),
                    transformer: None,
                    with_reason: false,
                    diff_with: Some(Tokenizer::Line),
                    weight: 1,
                }),
                // string body regex
                Box::new(SingleValueMatcher {
                    entity_name: "body",
                    comparator: Box::new(StringRegexMatchComparator::new()),
                    source: Box::new(BodyRegexSource::new()),
                    target: Box::new(StringBodyTarget::new()),
                    transformer: None,
                    with_reason: false,
                    diff_with: Some(Tokenizer::Line),
                    weight: 1,
                }),
                // JSON body contains
                Box::new(SingleValueMatcher {
                    entity_name: "body",
                    comparator: Box::new(JSONContainsMatchComparator::new()),
                    source: Box::new(PartialJSONBodySource::new()),
                    target: Box::new(JSONBodyTarget::new()),
                    transformer: None,
                    with_reason: false,
                    diff_with: Some(Tokenizer::Line),
                    weight: 1,
                }),
                // JSON body exact
                Box::new(SingleValueMatcher {
                    entity_name: "body",
                    comparator: Box::new(JSONExactMatchComparator::new()),
                    source: Box::new(JSONBodySource::new()),
                    target: Box::new(JSONBodyTarget::new()),
                    transformer: None,
                    with_reason: true,
                    diff_with: Some(Tokenizer::Line),
                    weight: 1,
                }),
                // User provided matcher function
                Box::new(FunctionValueMatcher {
                    entity_name: "user provided matcher function",
                    comparator: Box::new(FunctionMatchesRequestComparator::new()),
                    source: Box::new(FunctionSource::new()),
                    target: Box::new(FullRequestTarget::new()),
                    transformer: None,
                    weight: 1,
                }),
            ],
        }
    }
}

type GenericError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Default, Debug)]
pub(crate) struct ServerRequestHeader {
    pub method: String,
    pub path: String,
    pub query: String,
    pub headers: Vec<(String, String)>,
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
        headers: Vec<(String, String)>,
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
fn extract_headers(header_map: &HeaderMap) -> Result<Vec<(String, String)>, String> {
    let mut headers = Vec::new();
    for (hn, hv) in header_map {
        let hn = hn.as_str().to_string();
        let hv = hv.to_str();
        if let Err(e) = hv {
            return Err(format!("error parsing headers: {}", e));
        }
        headers.push((hn, hv.unwrap().to_string()));
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

    let entire_body = hyper::body::to_bytes(req.into_body()).await;
    if let Err(e) = entire_body {
        return Ok(error_response(format!("Cannot read request body: {}", e)));
    }

    let the_body = entire_body.unwrap();
    let body = String::from_utf8(the_body.to_vec());

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

    if VERIFY_PATH.is_match(&request_header.path) {
        match request_header.method.as_str() {
            "POST" => return routes::verify(state, body),
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
    static ref VERIFY_PATH: Regex = Regex::new(r"^/__verify$").unwrap();
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use futures_util::TryStreamExt;

    use crate::server::{
        error_response, get_path_param, map_response, ServerResponse, HISTORY_PATH, MOCKS_PATH,
        MOCK_PATH, PING_PATH, VERIFY_PATH,
    };
    use crate::Regex;
    use hyper::body::Bytes;
    use hyper::Error;

    #[test]
    fn route_regex_test() {
        assert_eq!(MOCK_PATH.is_match("/__mocks/1"), true);
        assert_eq!(MOCK_PATH.is_match("/__mocks/1295473892374"), true);
        assert_eq!(MOCK_PATH.is_match("/__mocks/abc"), false);
        assert_eq!(MOCK_PATH.is_match("/__mocks"), false);
        assert_eq!(MOCK_PATH.is_match("/__mocks/345345/test"), false);
        assert_eq!(MOCK_PATH.is_match("test/__mocks/345345/test"), false);

        assert_eq!(PING_PATH.is_match("/__ping"), true);
        assert_eq!(PING_PATH.is_match("/__ping/1295473892374"), false);
        assert_eq!(PING_PATH.is_match("test/ping/1295473892374"), false);

        assert_eq!(VERIFY_PATH.is_match("/__verify"), true);
        assert_eq!(VERIFY_PATH.is_match("/__verify/1295473892374"), false);
        assert_eq!(VERIFY_PATH.is_match("test/verify/1295473892374"), false);

        assert_eq!(HISTORY_PATH.is_match("/__history"), true);
        assert_eq!(HISTORY_PATH.is_match("/__history/1295473892374"), false);
        assert_eq!(HISTORY_PATH.is_match("test/history/1295473892374"), false);

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

        let body = async_std::task::block_on(async {
            return match hyper::body::to_bytes(body).await {
                Ok(bytes) => bytes.to_vec(),
                Err(e) => panic!(e),
            };
        });

        assert_eq!(String::from_utf8(body).unwrap(), "test".to_string())
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
