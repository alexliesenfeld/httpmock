use crate::server::handlers;
use crate::server::handlers::mocks::SetMockRequest;
use crate::server::handlers::{HttpMockRequest, HttpMockResponse, HttpMockState};
use actix_web::dev::HttpResponseBuilder;
use actix_web::http::StatusCode;
use actix_web::web::{Bytes, BytesMut, Data, Json, Payload};
use actix_web::{error, web, Error, HttpRequest, HttpResponse, Result};
use futures::{Future, Stream};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// This route is responsible for listing all currently stored mocks
pub fn list(state: Data<HttpMockState>) -> Result<HttpResponse> {
    let result = handlers::mocks::list_all(&state.into_inner());

    if let Err(e) = result {
        return Ok(HttpResponse::InternalServerError().body(e));
    }

    Ok(HttpResponse::Accepted().json(&result.unwrap()))
}

#[derive(Serialize, Deserialize, TypedBuilder, Clone, Debug)]
pub struct MockCreatedResponse {
    pub mock_id: usize,
}

/// This route is responsible for adding a new mock
pub fn add(state: Data<HttpMockState>, req: Json<SetMockRequest>) -> Result<HttpResponse> {
    let result = handlers::mocks::add_new_mock(&state.into_inner(), req.into_inner());

    return match result {
        Err(e) => Ok(HttpResponse::InternalServerError().body(e)),
        Ok(mock_id) => Ok(HttpResponse::Created().json(MockCreatedResponse { mock_id })),
    };
}

/// This route is responsible for clearing/deleting all mocks
pub fn clear(state: Data<HttpMockState>) -> Result<HttpResponse> {
    let result = handlers::mocks::clear_mocks(&state.into_inner());

    if let Err(e) = result {
        return Ok(HttpResponse::InternalServerError().body(e));
    }

    Ok(HttpResponse::Accepted().finish())
}

/// This route is responsible for deleting mocks
pub fn delete_one(state: Data<HttpMockState>, params: web::Path<usize>) -> Result<HttpResponse> {
    let result = handlers::mocks::delete_one(&state.into_inner(), params.into_inner());
    return match result {
        Err(e) => Ok(HttpResponse::InternalServerError().body(e)),
        Ok(found) => {
            if !found {
                return Ok(HttpResponse::NotFound().finish());
            }
            return Ok(HttpResponse::Accepted().finish());
        }
    };
}

/// This route is responsible for finding a mock that matches the current request and serve a
/// response according to the mock specification
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

/// Adds a byte chunk to an existing mutable byte buffer
fn append_chunk(mut buf: BytesMut, chunk: Bytes) -> Result<BytesMut> {
    buf.extend_from_slice(&chunk);
    Ok::<_, Error>(buf)
}

/// Processes an HTTP request to serve a mock
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

/// Maps the result of the serve handler to an HTTP response which the web framework understands
fn to_route_response(
    handler_result: Result<Option<HttpMockResponse>, String>,
) -> Result<HttpResponse> {
    return match handler_result {
        Err(e) => Err(error::ErrorInternalServerError(e)),
        Ok(res) => {
            return match res {
                None => Err(error::ErrorInternalServerError(
                    "Request did not match any route or mock",
                )),
                Some(http_mock_response) => Ok(to_http_response(http_mock_response)),
            }
        }
    };
}

/// Maps the request of the serve handler to a request representation which the handlers understand
fn to_handler_request(req: HttpRequest, body: String) -> HttpMockRequest {
    HttpMockRequest::builder()
        .method(req.method().as_str().to_string())
        .path(req.path().to_string())
        .headers(BTreeMap::new())
        .body(body)
        .build()
}

/// Maps the response of the serve handler to a response representation which the
/// web framework understand
fn to_http_response(res: HttpMockResponse) -> HttpResponse {
    let status_code = StatusCode::from_u16(res.status).unwrap();
    let mut response_builder = HttpResponseBuilder::new(status_code);

    return match res.body {
        Some(body) => response_builder.body(actix_web::body::Body::from(body.clone())),
        None => response_builder.finish(),
    };
}

#[cfg(test)]
mod test {
    use crate::server::handlers::HttpMockResponse;
    use crate::server::routes::mocks::{to_http_response, to_route_response};
    use actix_http::body::BodySize;
    use actix_http::body::{Body, MessageBody};
    use actix_http::Response;
    use actix_web::http::StatusCode;

    /// TODO: Checks if the delete route behaves as expected (especially with parameter parsing, bad request, etc.)
    #[test]
    fn delete_route() {}

    /// This test makes sure that a handler response with an HTTP status and an empty body is
    /// mapped correctly to a representation that the web framework understands
    #[test]
    fn to_http_response_has_no_body() {
        // Arrange
        let input = HttpMockResponse::builder().status(200 as u16).build();

        // Act
        let actual = to_http_response(input);

        // Assert
        assert_eq!(StatusCode::from_u16(200).unwrap(), actual.status());
        assert_eq!(0, body_size(&actual));
    }

    /// This test makes sure that a handler response with an HTTP status and a non-empty body is
    /// mapped correctly to a representation that the web framework understands
    #[test]
    fn to_http_response_has_body() {
        // Arrange
        let input = HttpMockResponse::builder()
            .status(200 as u16)
            .body("#".to_string())
            .build();

        // Act
        let actual = to_http_response(input);

        // Assert
        assert_eq!(StatusCode::from_u16(200).unwrap(), actual.status());
        assert_eq!(1, body_size(&actual));
    }

    /// This test makes sure that an invalid HTTP status code cannot be returned because
    /// the mapper panics.
    #[test]
    #[should_panic(expected = "value: InvalidStatusCode ")]
    fn to_http_response_fails_invalid_http_status() {
        // Arrange
        let input = HttpMockResponse::builder().status(999 as u16).build();

        // Act
        to_http_response(input);

        // Assert
        // See 'should panic' above
    }

    /// This test makes sure that a handler response with an error is mapped correctly
    /// to an Internal Server Error response.
    #[test]
    fn to_route_response_internal_server_error() {
        // Arrange
        let input = Err("error message".to_string());

        // Act
        let actual = to_route_response(input);

        // Assert
        assert_eq!(true, actual.is_err());
        let err = actual.unwrap_err();
        assert_eq!("error message", err.to_string());
        assert_eq!(
            500 as u16,
            err.as_response_error().error_response().status()
        );
    }

    /// This test makes sure that a status code 404 is returned if no mock has been found
    #[test]
    fn to_route_response_not_found() {
        // Arrange
        let input = Ok(None);

        // Act
        let actual = to_route_response(input);

        // Assert
        assert_eq!(actual.is_err(), true);
        let err = actual.unwrap_err();
        assert_eq!("Request did not match any route or mock", err.to_string());
        assert_eq!(
            404 as u16,
            err.as_response_error().error_response().status()
        );
    }

    /// This test makes sure that a mock is successfully returned if one is found.
    #[test]
    fn to_route_response_ok() {
        // Arrange
        let input_response = HttpMockResponse::builder().status(418 as u16).build();

        let input = Ok(Some(input_response));

        // Act
        let actual = to_route_response(input);

        // Assert
        assert_eq!(actual.is_ok(), true);
        assert_eq!(actual.unwrap().status().as_u16(), 418 as u16);
    }

    fn body_size(body: &Response<Body>) -> u64 {
        match body.body().size() {
            BodySize::Sized(x) => x as u64,
            BodySize::Sized64(x) => x,
            _ => 0,
        }
    }
}
