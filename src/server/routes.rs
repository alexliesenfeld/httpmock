use crate::server::{handlers, ServerRequest, ServerResponse};

use crate::server::data::*;

use qstring::QString;
use serde::Serialize;
use std::collections::BTreeMap;
use std::io::Cursor;

use hyper::rt::{Future, Stream};
use hyper::service::{make_service_fn, service_fn};

use hyper::Chunk;

/// This route is responsible for adding a new mock
pub fn add(
    state: &ApplicationState,
    req: ServerRequest,
) -> Result<ServerResponse, String> {
    let mock_def: serde_json::Result<MockDefinition> = serde_json::from_str(&req.body);
    if let Err(e) = mock_def {
        return create_json_response(500, None, ErrorResponse::new(&e));
    }
    let mock_def = mock_def.unwrap();

    let result = handlers::add_new_mock(&state, mock_def);

    return match result {
        Err(e) => create_json_response(500, None, ErrorResponse::new(&e)),
        Ok(mock_id) => create_json_response(201, None, MockIdentification { mock_id }),
    };
}

/// This route is responsible for deleting mocks
pub fn delete_one(
    state: &ApplicationState,
    _req: ServerRequest,
    id: usize,
) -> Result<ServerResponse, String> {
    let result = handlers::delete_one(state, id);
    return match result {
        Err(e) => create_json_response(500, None, ErrorResponse::new(&e)),
        Ok(found) => match found {
            true => create_response(202, None, None),
            false => create_response(404, None, None),
        },
    };
}

/// This route is responsible for deleting all mocks
pub fn delete_all(
    state: &ApplicationState,
    _req: ServerRequest,
) -> Result<ServerResponse, String> {
    let result = handlers::delete_all(state);
    return match result {
        Err(e) => create_json_response(500, None, ErrorResponse::new(&e)),
        Ok(_) => create_response(202, None, None),
    };
}

/// This route is responsible for deleting mocks
pub fn read_one(
    state: &ApplicationState,
    _req: ServerRequest,
    id: usize,
) -> Result<ServerResponse, String> {
    let handler_result = handlers::read_one(state, id);
    return match handler_result {
        Err(e) => create_json_response(500, None, ErrorResponse { message: e.clone() }),
        Ok(mock_opt) => {
            return match mock_opt {
                Some(mock) => create_json_response(200, None, mock),
                None => create_response(404, None, None),
            }
        }
    };
}

/// This route is responsible for finding a mock that matches the current request and serve a
/// response according to the mock specification
pub fn serve(state: &ApplicationState, req: ServerRequest) -> Result<ServerResponse, String> {
    let handler_request_result = to_handler_request(&req);
    return match handler_request_result {
        Ok(handler_request) => {
            let handler_response = handlers::find_mock(&state, handler_request);
            return to_route_response(handler_response);
        }
        Err(e) => create_json_response(500, None, ErrorResponse::new(&e)),
    };
}

/// Maps the result of the serve handler to an HTTP response which the web framework understands
fn to_route_response(
    handler_result: Result<Option<MockServerHttpResponse>, String>,
) -> Result<ServerResponse, String> {
    return match handler_result {
        Err(e) => create_json_response(500 as u16, None, ErrorResponse { message: e.clone() }),
        Ok(res) => {
            return match res {
                None => create_json_response(
                    500,
                    None,
                    ErrorResponse::new(&"Request did not match any route or mock"),
                ),
                Some(res) => create_response(res.status, res.headers, res.body),
            }
        }
    };
}

fn create_json_response<T>(
    status: u16,
    headers: Option<BTreeMap<String, String>>,
    body: T,
) -> Result<ServerResponse, String>
where
    T: Serialize,
{
    let body = serde_json::to_string(&body);
    if let Err(e) = body {
        return Err(format!("Cannot serialize body: {}", e));
    }

    let mut headers = headers.unwrap_or(BTreeMap::new());
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    create_response(status, Some(headers), Some(body.unwrap()))
}

fn create_response(
    status: u16,
    headers: Option<BTreeMap<String, String>>,
    body: Option<String>,
) -> Result<ServerResponse, String> {
    Ok(ServerResponse::builder()
        .status(status)
        .headers(headers.unwrap_or(BTreeMap::new()))
        .body(body.unwrap_or(String::new()))
        .build())
}

/// Maps the request of the serve handler to a request representation which the handlers understand
fn to_handler_request(req: &ServerRequest) -> Result<MockServerHttpRequest, String> {
    let query_params = extract_query_params(&req.query);
    if let Err(e) = query_params {
        return Err(format!("error parsing query_params: {}", e));
    }

    let request = MockServerHttpRequest::builder()
        .method(req.method.to_string())
        .path(req.path.to_string())
        .headers(req.headers.clone())
        .query_params(query_params.unwrap().clone())
        .body(req.body.to_string())
        .build();

    Ok(request)
}

/// Extracts all query parameters from the URI of the given request.
fn extract_query_params(query_string: &str) -> Result<BTreeMap<String, String>, String> {
    let mut query_params = BTreeMap::new();

    for (key, value) in QString::from(query_string) {
        query_params.insert(key.to_string(), value.to_string());
    }

    Ok(query_params)
}

/*
#[cfg(test)]
mod test {
    use crate::server::data::MockServerHttpResponse as HttpMockResponse;
    use crate::server::routes::{to_http_response, to_route_response};
    use actix_http::body::BodySize;
    use actix_http::body::{Body, MessageBody};
    use actix_http::Response;
    use actix_web::http::StatusCode;
    use std::io::Cursor;

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
        let actual :  Response<Cursor<Vec<u8>>> = to_http_response(input);

        // Assert
        assert_eq!(StatusCode::from_u16(200).unwrap(), &actual);
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
            500 as u16,
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

    fn body_size(body: &tiny_http::Response<Cursor<Vec<u8>>>) -> u64 {
        match body.body().size() {
            BodySize::Sized(x) => x as u64,
            BodySize::Sized64(x) => x,
            _ => 0,
        }
    }
}
*/
