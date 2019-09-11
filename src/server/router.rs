extern crate typed_builder;

use crate::server::router::Method::{GET, POST};
use log::{debug, info};
use std::collections::HashMap;
use std::fmt;
use std::fmt::write;

#[derive(TypedBuilder, Debug)]
pub struct HttpMockRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(TypedBuilder, Debug)]
pub struct HttpMockResponse {
    pub status_code: u16,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
}

pub type RouterFunction = fn(&HandlerConfig, HttpMockRequest, HttpMockParams) -> HttpMockResponse;

#[derive(TypedBuilder)]
pub struct Route {
    pub path: String,
    pub method: Method,
    pub handler: RouterFunction,
}

impl Route {
    pub fn from(method: Method, path: &str, handler: RouterFunction) -> Route {
        Route {
            method,
            path: path.to_string(),
            handler,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Method {
    ANY,
    GET,
    POST,
    PUT,
    DELETE,
    OPTIONS,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(TypedBuilder)]
pub struct HandlerConfig {
    pub router: HttpMockRouter,
}

pub type HttpMockParams = route_recognizer::Params;
pub type HttpMockRouter = route_recognizer::Router<Route>;

pub fn handle_route(
    request: HttpMockRequest,
    router: &HttpMockRouter,
    handler_config: &HandlerConfig,
) -> Option<(HttpMockResponse)> {
    let route = router.recognize(&request.path);

    if let Ok(matched_route) = route {
        let route = matched_route.handler;
        let params = matched_route.params;
        let route_method = route.method.to_string();
        let request_method = request.method.to_string();

        if route_method == request_method || route.method == Method::ANY {
            let result = (route.handler)(handler_config, request, params);
            return Option::Some(result);
        }
    }

    return Option::None;
}
