extern crate typed_builder;
use log::info;
use std::collections::HashMap;

#[derive(TypedBuilder, Debug)]
pub struct HttpMockHandlerRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(TypedBuilder, Debug)]
pub struct HttpMockHandlerResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(TypedBuilder, Debug)]
pub struct HandlerConfig {}

#[derive(Debug)]
pub struct RequestHandler {
    config: HandlerConfig,
}

impl RequestHandler {
    pub fn from_config(config: HandlerConfig) -> RequestHandler {
        RequestHandler { config }
    }

    pub fn handle(&self, request: HttpMockHandlerRequest) -> HttpMockHandlerResponse {
        info!("{}{}", request.method, request.path);

        let mut h = HashMap::new();
        h.insert(
            String::from("Content-Type"),
            String::from("application/json"),
        );

        HttpMockHandlerResponse::builder()
            .status_code(201 as u16)
            .headers(h)
            .body(String::from("{\"ok\" : \"true\"}"))
            .build()
    }
}
