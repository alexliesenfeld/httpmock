extern crate futures;
extern crate hyper;

pub mod handler;

use self::hyper::header::HeaderValue;
use self::hyper::http::header::HeaderName;
use self::hyper::service::service_fn;
use self::hyper::{HeaderMap, Request, StatusCode};
use crate::server::handler::RequestHandler;
use futures::future;
use handler::{HttpMockHandlerRequest, HttpMockHandlerResponse};
use hyper::rt::Future;
use hyper::{Body, Response, Server};
use log::info;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

type BoxFut = Box<dyn Future<Item = Response<Body>, Error = hyper::Error> + Send>;

#[derive(TypedBuilder, Debug)]
pub struct ServerConfig {
    pub port: u16,
    pub request_handler: RequestHandler,
}

pub struct HttpMockServer {
    server_config: Arc<ServerConfig>,
}

impl HttpMockServer {
    pub fn from_config(config: ServerConfig) -> HttpMockServer {
        HttpMockServer {
            server_config: Arc::new(config),
        }
    }

    pub fn start(&self) {
        let socket_address = ([127, 0, 0, 1], self.server_config.port).into();
        let server_config = self.server_config.clone();

        let server = Server::bind(&socket_address)
            .serve(move || {
                let server_config = server_config.clone();
                service_fn(move |req: Request<Body>| {
                    let handler_request = to_handler_request(&req);
                    let handler_response = server_config.request_handler.handle(handler_request);
                    let server_response = to_response(handler_response);
                    return Box::new(future::ok(server_response)) as BoxFut;
                })
            })
            .map_err(|e| eprintln!("server error: {}", e));

        info!("Listening on {}", socket_address);
        hyper::rt::run(server);
    }
}

fn to_headers(headers: &HashMap<String, String>) -> HeaderMap<HeaderValue> {
    let mut header_map = HeaderMap::with_capacity(headers.capacity());
    for (k, v) in headers {
        let hv = HeaderValue::from_str(v).expect(&format!("Cannot create header value from {}", v));
        let hn = HeaderName::from_str(k).expect(&format!("Cannot create header name from {}", k));;
        header_map.insert(hn, hv);
    }
    return header_map;
}

fn to_handler_request(req: &Request<Body>) -> HttpMockHandlerRequest {
    let req_path = req.uri().path().to_string();
    let req_method = req.method().as_str().to_string();
    let req_headers = HashMap::new();
    let _req_body = req.body();

    let handler_request = HttpMockHandlerRequest::builder()
        .method(req_method)
        .path(req_path)
        .headers(req_headers)
        .body(String::new())
        .build();

    handler_request
}

fn to_response(handler_response: HttpMockHandlerResponse) -> Response<Body> {
    let mut response = Response::new(Body::from(handler_response.body));
    *response.status_mut() = StatusCode::from_u16(handler_response.status_code)
        .expect("Cannot parse status code from handler");
    *response.headers_mut() = to_headers(&handler_response.headers);
    response
}
