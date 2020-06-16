use crate::server::data::ApplicationState;

use regex::Regex;

use futures::{future, Future, Stream};

use hyper::service::service_fn;
use hyper::{
    http::response::Builder as HyperResponseBuilder, Body, Chunk, HeaderMap,
    Request as HyperRequest, Response as HyperResponse, Server, StatusCode,
};

use hyper::header::HeaderValue;
use hyper::http::header::HeaderName;
use std::collections::BTreeMap;
use std::str::FromStr;

pub(crate) mod data;
mod handlers;
mod routes;
mod util;

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type ResponseFuture = Box<dyn Future<Item = HyperResponse<Body>, Error = GenericError> + Send>;

/// Holds server configuration properties.
#[derive(Debug)]
pub struct HttpMockConfig {
    pub port: u16,
    pub workers: usize,
    pub expose: bool,
}

impl HttpMockConfig {
    pub fn new(port: u16, workers: usize, expose: bool) -> Self {
        Self {
            port,
            workers,
            expose,
        }
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

/// Starts a new instance of an HTTP mock server. You should never need to use this function
/// directly. Use it if you absolutely need to manage the low-level details of how the mock
/// server operates.
pub fn start_server(http_mock_config: HttpMockConfig) {
    let port = http_mock_config.port;
    let host = match http_mock_config.expose {
        true => "0.0.0.0",    // allow traffic from all sources
        false => "127.0.0.1", // allow traffic from localhost only
    };

    hyper::rt::run(future::lazy(move || {
        let new_service = move || {
            service_fn(|req: HyperRequest<Body>| {
                let request_header = ServerRequestHeader::from(&req);
                Box::new(
                    req.into_body()
                        .concat2()
                        .from_err()
                        .and_then(|entire_body: Chunk| {
                            if let Err(e) = request_header {
                                return Ok(error_response(format!("Cannot parse request: {}", e)));
                            }

                            let body = String::from_utf8(entire_body.to_vec());
                            if let Err(e) = body {
                                return Ok(error_response(format!("Cannot read body: {}", e)));
                            }

                            let routing_result =
                                route_request(&STATE, &request_header.unwrap(), body.unwrap());
                            if let Err(e) = routing_result {
                                return Ok(error_response(format!("Request handler error: {}", e)));
                            }

                            let response = map_response(routing_result.unwrap());
                            if let Err(e) = response {
                                return Ok(error_response(format!("Cannot build response: {}", e)));
                            }

                            Ok(response.unwrap())
                        }),
                ) as ResponseFuture
            })
        };

        let addr = &format!("{}:{}", host, port).parse().unwrap();
        let server = Server::bind(&addr)
            .serve(new_service)
            .map_err(|e| log::error!("server error: {}", e));

        log::info!("Listening on {}", addr);

        server
    }));
}

/// Maps a server response to a hyper response.
fn map_response(route_response: ServerResponse) -> Result<HyperResponse<Body>, String> {
    let mut builder = HyperResponse::builder();
    builder.status(route_response.status);

    for (k, v) in route_response.headers {
        let value = add_header(&mut builder, &k, &v);
        if let Err(e) = value {
            return Err(format!("Cannot create header from value string: {}", e));
        }
    }

    let result = builder.body(Body::from(route_response.body));
    if let Err(e) = result {
        return Err(format!("Cannot create HTTP response: {}", e));
    }

    Ok(result.unwrap())
}

/// Adds a header to a hyper response.
fn add_header(builder: &mut HyperResponseBuilder, key: &str, value: &str) -> Result<(), String> {
    let name = HeaderName::from_str(key);
    if let Err(e) = name {
        return Err(format!("Cannot create header from name: {}", e));
    }

    let value = HeaderValue::from_str(value);
    if let Err(e) = value {
        return Err(format!("Cannot create header from value: {}", e));
    }

    let value = value.unwrap();
    let value = value.to_str();
    if let Err(e) = value {
        return Err(format!("Cannot create header from value string: {}", e));
    }

    builder.header(name.unwrap(), value.unwrap());

    Ok(())
}

/// Routes a request to the appropriate route handler.
fn route_request(
    state: &ApplicationState,
    request_header: &ServerRequestHeader,
    body: String,
) -> Result<ServerResponse, String> {
    log::trace!("Routing incoming request: {:?}", request_header);

    if MOCKS_PATH.is_match(&request_header.path) {
        match request_header.method.as_str() {
            "GET" => return routes::read_all(state),
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

    return routes::serve(state, request_header, body);
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
    static ref MOCK_PATH: Regex = Regex::new(r"/__mocks/([0-9]+)$").unwrap();
    static ref MOCKS_PATH: Regex = Regex::new(r"/__mocks(/)?$").unwrap();
    static ref STATE: ApplicationState = ApplicationState::new();
}

#[cfg(test)]
mod test {
    use crate::server::{MOCKS_PATH, MOCK_PATH};

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
}
