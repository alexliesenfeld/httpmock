use crate::server::router::{HttpMockResponse, HttpStatusCode};
use serde::Deserialize;
use std::collections::HashMap;
use std::io::Error;

pub fn parse_json<'a, T>(json: &'a String) -> Option<Result<T, Error>>
where
    T: Deserialize<'a>,
{
    let json = json.trim();
    if json.is_empty() {
        return Option::None;
    }

    let result = serde_json::from_str(json.trim());

    return match result {
        Ok(content) => Option::Some(Result::Ok(content)),
        Err(e) => Option::Some(Result::Err(Error::from(e))),
    };
}

pub fn response_with_status(status: HttpStatusCode) -> HttpMockResponse {
    HttpMockResponse::builder()
        .status_code(status)
        .headers(Option::None)
        .body(Option::None)
        .build()
}

pub fn response_with_status_and_body(status: HttpStatusCode, body: String) -> HttpMockResponse {
    HttpMockResponse::builder()
        .status_code(status)
        .headers(Option::None)
        .body(Option::Some(body))
        .build()
}

pub fn response_with_status_and_body_and_headers(
    status: HttpStatusCode,
    body: String,
    headers: HashMap<String, String>,
) -> HttpMockResponse {
    HttpMockResponse::builder()
        .status_code(status)
        .headers(Option::Some(headers))
        .body(Option::Some(body))
        .build()
}


