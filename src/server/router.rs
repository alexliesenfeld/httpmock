extern crate typed_builder;

use log::{debug, info};
use std::collections::HashMap;
use std::fmt;
use std::fmt::write;

pub type HttpMockParams = route_recognizer::Params;
pub type HttpMockRouter = route_recognizer::Router<RouterFunction>;
pub type HttpStatusCode = http::status::StatusCode;
pub type HttpMethod = http::method::Method;

#[derive(TypedBuilder, Debug)]
pub struct HttpMockRequest {
    pub method: HttpMethod,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(TypedBuilder, Debug)]
pub struct HttpMockResponse {
    pub status_code: HttpStatusCode,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
}

pub type RouterFunction = fn(&HandlerConfig, HttpMockRequest, HttpMockParams) -> HttpMockResponse;

#[derive(TypedBuilder)]
pub struct HandlerConfig {
    pub router: HttpMockRouter,
}

pub fn handle_route(
    request: HttpMockRequest,
    router: &HttpMockRouter,
    handler_config: &HandlerConfig,
) -> Option<(HttpMockResponse)> {
    let result = router.recognize(&request.path);
    if let Ok(matched_handler) = result {
        let handler = matched_handler.handler;
        let params = matched_handler.params;
        let result = (handler)(handler_config, request, params);
        return Option::Some(result);
    }

    return Option::None;
}
