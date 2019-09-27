use crate::server::data::ApplicationState;

use regex::Regex;
use std::io::Cursor;
use std::sync::Arc;
use std::thread;

use futures::{future, Future, IntoFuture, Stream};

use hyper::client::HttpConnector;
use hyper::service::{service_fn, service_fn_ok};
use hyper::{
    header, Body, Chunk, Client, HeaderMap, Method, Request, Response, Server, StatusCode,
};

use hyper::header::HeaderValue;
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};

pub(crate) mod data;
mod handlers;
mod routes;
mod util;

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type ResponseFuture = Box<dyn Future<Item = Response<Body>, Error = GenericError> + Send>;

/// Holds server configuration properties.
#[derive(TypedBuilder, Debug)]
pub struct HttpMockConfig {
    pub port: u16,
    pub workers: usize,
    pub expose: bool,
}

#[derive(TypedBuilder, Default, Debug)]
pub struct ServerRequest {
    pub method: String,
    pub path: String,
    pub query: String,
    pub headers: BTreeMap<String, String>,
    pub body: String,
}

#[derive(TypedBuilder, Default, Debug)]
pub struct ServerResponse {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body: String,
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


            service_fn(|req: Request<Body>| {
                let headers = req.headers().clone();
                let path = req.uri().path().to_string();
                let query = req.uri().query().unwrap_or("").to_string();
                let method = req.method().as_str().to_string();

                Box::new(
                    req.into_body()
                        .concat2()
                        .from_err()
                        .and_then(|entire_body: Chunk| {

                            Ok(process_request(
                                &APPLICATION_STATE,
                                method,
                                path,
                                query,
                                headers,
                                entire_body
                            ))

                        }),
                ) as ResponseFuture
            })
        };

        let addr = &format!("{}:{}", host, port).parse().unwrap();
        let server = Server::bind(&addr)
            .serve(new_service)
            .map_err(|e| eprintln!("server error: {}", e));

        println!("Listening on http://{}", addr);

        server
    }));
}

fn process_request(
    state: &ApplicationState,
    method: String,
    path: String,
    query: String,
    headers: HeaderMap<HeaderValue>,
    body: Chunk,
) -> hyper::Response<Body> {
    let body = String::from_utf8(body.to_vec());
    if let Err(e) = body {
        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(format!("cannot build response: {}", e)))
            .expect("Cannot build body error response");
    }

    let headers = extract_headers(&headers);
    if let Err(e) = headers {
        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(format!(
                "cannot build headers error response: {}",
                e
            )))
            .expect("Cannot build response");
    }

    let routing_result = route_request(
        state,
        ServerRequest::builder()
            .method(method)
            .path(path)
            .query(query)
            .headers(headers.unwrap())
            .body(body.unwrap())
            .build(),
    );

    if let Err(e) = routing_result {
        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(format!("Cannot build body: {}", e)))
            .expect("Cannot build route error response");
    }

    let route_response = routing_result.unwrap();

    let mut builder = hyper::Response::builder();
    builder.status(route_response.status);

    for (k, v) in route_response.headers {
        let name = hyper::header::HeaderName::from_bytes(k.as_bytes());
        if let Err(e) = name {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!("Cannot create header from name: {}", e)))
                .expect("Cannot build route error response");
        }

        let value = hyper::header::HeaderValue::from_bytes(v.as_bytes());
        if let Err(e) = value {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!("Cannot create header from value: {}", e)))
                .expect("Cannot build route error response");
        }
        let value = value.unwrap();
        let value = value.to_str();
        if let Err(e) = value {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!("Cannot create header from value string: {}", e)))
                .expect("Cannot build route error response");
        }

        builder.header(name.unwrap(), value.unwrap());
    }

    let result = builder.body(Body::from(route_response.body));
    if let Err(e) = result {
        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(format!("Cannot create HTTP response: {}", e)))
            .expect("Cannot build response HTTP error response");
    }

    result.unwrap()
}

fn route_request(state: &ApplicationState, req: ServerRequest) -> Result<ServerResponse, String> {
    // log::trace!("Matching new incoming request with uri '{}'", req.());

        if MOCKS_PATH.is_match(&req.path) {
            match req.method.as_str() {
                "POST" => return routes::add(state, req),
                "DELETE" => return routes::delete_all(state, req),
                _ => {}
            }
        }

        if MOCK_PATH.is_match(&req.path) {
            let id = get_path_param(&MOCK_PATH, 1, &req.path);
            if let Err(e) = id {
                return Err(format!("Cannot parse id from path: {}", e));
            }
            let id = id.unwrap();

            match req.method.as_str() {
                "GET" => return routes::read_one(state, req, id),
                "DELETE" => return routes::delete_one(state, req, id),
                _ => {}
            }
        }

    return routes::serve(state, req);
}
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

lazy_static! {
    static ref MOCK_PATH: Regex = Regex::new(r"/__mocks/([0-9]+)$").unwrap();
    static ref MOCKS_PATH: Regex = Regex::new(r"/__mocks$").unwrap();
    static ref APPLICATION_STATE: ApplicationState = ApplicationState::new();

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
    }
}
