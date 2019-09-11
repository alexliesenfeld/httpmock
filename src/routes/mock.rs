use crate::server::io::{parse_json, response_with_status_and_body, response_with_status};
use crate::server::router::{HandlerConfig, HttpMockParams, HttpMockRequest, HttpMockResponse, HttpStatusCode, HttpMethod};
use serde::Deserialize;
use std::collections::HashMap;
use std::io::Error;

#[derive(Deserialize, Debug)]
pub struct MockRequest {
    pub path: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
}

pub fn mock_resource_handler(
    config: &HandlerConfig,
    req: HttpMockRequest,
    params: HttpMockParams,
) -> HttpMockResponse {

    if req.method == HttpMethod::POST {
        let body: Option<Result<MockRequest, Error>> = parse_json(&req.body);
        return response_with_status_and_body(HttpStatusCode::OK, String::from("OK"));
    }

    return response_with_status(HttpStatusCode::UNSUPPORTED_MEDIA_TYPE)
}

pub fn get_user_mock(
    config: &HandlerConfig,
    req: HttpMockRequest,
    params: HttpMockParams,
) -> HttpMockResponse {
    return response_with_status_and_body(HttpStatusCode::IM_A_TEAPOT, String::from("Teapot!"));
}
