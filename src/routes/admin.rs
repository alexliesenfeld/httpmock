use crate::handlers;
use crate::handlers::mocks::SetMockRequest;
use crate::handlers::{HttpMockRequest, HttpMockResponse, HttpMockState};
use actix_web::dev::HttpResponseBuilder;
use actix_web::http::StatusCode;
use actix_web::web::{Bytes, BytesMut, Data, Json, Payload};
use actix_web::{error, Error, HttpRequest, HttpResponse, Result};
use futures::{Future, Stream};

use std::collections::BTreeMap;

/// This route is responsible for listing all currently stored mocks
pub fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().body("OK"))
}
