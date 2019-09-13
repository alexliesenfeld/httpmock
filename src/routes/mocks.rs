use crate::handlers;
use crate::handlers::{HttpMockRequest, HttpMockResponse, HttpMockState, SetMockRequest};
use actix_web::dev::HttpResponseBuilder;
use actix_web::http::StatusCode;
use actix_web::web::{Bytes, BytesMut, Data, Json, Payload};
use actix_web::{error, Error, HttpRequest, HttpResponse, Result};
use futures::{Future, Stream};
use std::collections::BTreeMap;

pub fn list() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().finish())
}

pub fn add(state: Data<HttpMockState>, req: Json<SetMockRequest>) -> Result<HttpResponse> {
    let result = handlers::mocks::add_new_mock(&state.into_inner(), req.into_inner());

    if let Err(e) = result {
        return Ok(HttpResponse::InternalServerError().body(e));
    }

    Ok(HttpResponse::Created().finish())
}

pub fn clear(state: Data<HttpMockState>, req: Json<SetMockRequest>) -> Result<HttpResponse> {
    let result = handlers::mocks::clear_mocks(&state.into_inner(), req.into_inner());

    if let Err(e) = result {
        return Ok(HttpResponse::InternalServerError().body(e));
    }

    Ok(HttpResponse::Ok().finish())
}

pub fn serve(
    state: Data<HttpMockState>,
    req: HttpRequest,
    payload: Payload,
) -> impl Future<Item = HttpResponse, Error = Error> {
    return payload
        .from_err()
        .fold(BytesMut::new(), append_chunk)
        .and_then(|body| handle_mock_request(body, state, req));
}

fn append_chunk(mut buf: BytesMut, chunk: Bytes) -> Result<BytesMut> {
    buf.extend_from_slice(&chunk);
    Ok::<_, Error>(buf)
}

fn handle_mock_request(
    body_buffer: BytesMut,
    state: Data<HttpMockState>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    return match String::from_utf8(body_buffer.to_vec()) {
        Ok(content) => {
            let handler_request = to_handler_request(req, content);
            let handler_response = handlers::mocks::find_mock(&state, handler_request);
            return to_route_response(handler_response);
        }
        Err(error) => Err(error::ErrorBadRequest(error.to_string())),
    };
}

fn to_route_response(
    handler_result: Result<Option<HttpMockResponse>, &'static str>,
) -> Result<HttpResponse> {
    return match handler_result {
        Err(e) => Err(error::ErrorInternalServerError(e)),
        Ok(res) => {
            return match res {
                None => Err(error::ErrorNotFound("No requests matched")),
                Some(http_mock_response) => Ok(to_http_response(http_mock_response)),
            }
        }
    };
}

fn to_handler_request(req: HttpRequest, body: String) -> HttpMockRequest {
    HttpMockRequest::builder()
        .method(req.method().as_str().to_string())
        .path(req.path().to_string())
        .headers(BTreeMap::new())
        .body(body)
        .build()
}

fn to_http_response(res: HttpMockResponse) -> HttpResponse {
    let _status_code = StatusCode::from_u16(res.status).unwrap();
    let mut response_builder = HttpResponseBuilder::new(StatusCode::from_u16(res.status).unwrap());

    return match res.body {
        Some(body) => response_builder.body(actix_web::body::Body::from(body.clone())),
        None => response_builder.finish(),
    };
}
