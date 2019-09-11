use crate::server::io::{parse_json, response_with_status_and_body};
use crate::server::router::{HandlerConfig, HttpMockParams, HttpMockRequest, HttpMockResponse};
use serde::Deserialize;
use std::collections::HashMap;
use std::io::Error;

#[derive(Deserialize, Debug)]
pub struct MockRequest {
    pub path: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
}

pub fn add_mock(
    config: &HandlerConfig,
    req: HttpMockRequest,
    params: HttpMockParams,
) -> HttpMockResponse {
    let body: Option<Result<MockRequest, Error>> = parse_json(&req.body);
    return response_with_status_and_body(200, String::from("OK"));
}
