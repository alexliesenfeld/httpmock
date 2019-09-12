use actix_web::web::Path;
use actix_web::{get, web, HttpRequest, HttpResponse, Result};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct MockRequest {
    pub path: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct MockResponse {
    pub how: String,
}

#[get("/__admin/health/{name}")]
pub fn index(name: Path<String>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(MockResponse {
        how: name.to_string(),
    }))
}

pub fn catch_all(req: HttpRequest) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(MockResponse {
        how: "catch_all".to_string(),
    }))
}
