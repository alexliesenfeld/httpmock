use std::collections::BTreeMap;

use serde::Serialize;

use crate::common::data::{
    ErrorResponse, HttpMockRequest, MockDefinition, MockRef, MockServerHttpResponse,
    RequestRequirements,
};
use crate::server::web::handlers;
use crate::server::{MockServerState, ServerRequestHeader, ServerResponse};
use std::time::Instant;
use tokio::time::Duration;

/// This route is responsible for adding a new mock
pub(crate) fn ping() -> Result<ServerResponse, String> {
    create_response(200, None, None)
}

/// This route is responsible for adding a new mock
pub(crate) fn add(state: &MockServerState, body: Vec<u8>) -> Result<ServerResponse, String> {
    let mock_def: serde_json::Result<MockDefinition> = serde_json::from_slice(&body);

    if let Err(e) = mock_def {
        return create_json_response(500, None, ErrorResponse::new(&e));
    }
    let mock_def = mock_def.unwrap();

    let result = handlers::add_new_mock(&state, mock_def, false);

    match result {
        Err(e) => create_json_response(500, None, ErrorResponse::new(&e)),
        Ok(mock_id) => create_json_response(201, None, MockRef { mock_id }),
    }
}

/// This route is responsible for deleting mocks
pub(crate) fn delete_one(state: &MockServerState, id: usize) -> Result<ServerResponse, String> {
    let result = handlers::delete_one_mock(state, id);
    match result {
        Err(e) => create_json_response(500, None, ErrorResponse::new(&e)),
        Ok(found) => {
            if found {
                create_response(202, None, None)
            } else {
                create_response(404, None, None)
            }
        }
    }
}

/// This route is responsible for deleting all mocks
pub(crate) fn delete_all_mocks(state: &MockServerState) -> Result<ServerResponse, String> {
    handlers::delete_all_mocks(state);
    create_response(202, None, None)
}

/// This route is responsible for deleting all mocks
pub(crate) fn delete_history(state: &MockServerState) -> Result<ServerResponse, String> {
    handlers::delete_history(state);
    create_response(202, None, None)
}

/// This route is responsible for deleting mocks
pub(crate) fn read_one(state: &MockServerState, id: usize) -> Result<ServerResponse, String> {
    let handler_result = handlers::read_one_mock(state, id);
    match handler_result {
        Err(e) => create_json_response(500, None, ErrorResponse { message: e }),
        Ok(mock_opt) => match mock_opt {
            Some(mock) => create_json_response(200, None, mock),
            None => create_response(404, None, None),
        },
    }
}

/// This route is responsible for verification
pub(crate) fn verify(state: &MockServerState, body: Vec<u8>) -> Result<ServerResponse, String> {
    let mock_rr: serde_json::Result<RequestRequirements> = serde_json::from_slice(&body);
    if let Err(e) = mock_rr {
        return create_json_response(500, None, ErrorResponse::new(&e));
    }

    match handlers::verify(&state, &mock_rr.unwrap()) {
        Err(e) => create_json_response(500, None, ErrorResponse::new(&e)),
        Ok(closest_match) => match closest_match {
            None => create_response(404, None, None),
            Some(cm) => create_json_response(200, None, cm),
        },
    }
}

/// This route is responsible for finding a mock that matches the current request and serve a
/// response according to the mock specification
pub(crate) async fn serve(
    state: &MockServerState,
    req: &ServerRequestHeader,
    body: Vec<u8>,
) -> Result<ServerResponse, String> {
    let handler_request_result = to_handler_request(&req, body);
    let result = match handler_request_result {
        Ok(handler_request) => {
            let handler_response = handlers::find_mock(&state, handler_request);
            let handler_response = postprocess_response(handler_response).await;
            to_route_response(handler_response)
        }
        Err(e) => create_json_response(500, None, ErrorResponse::new(&e)),
    };
    return result;
}

/// Maps the result of the serve handler to an HTTP response which the web framework understands
fn to_route_response(
    handler_result: Result<Option<MockServerHttpResponse>, String>,
) -> Result<ServerResponse, String> {
    match handler_result {
        Err(e) => create_json_response(500 as u16, None, ErrorResponse { message: e }),
        Ok(res) => match res {
            None => create_json_response(
                404,
                None,
                ErrorResponse::new(&"Request did not match any route or mock"),
            ),
            Some(res) => create_response(res.status.unwrap_or(200), res.headers, res.body),
        },
    }
}

fn create_json_response<T>(
    status: u16,
    headers: Option<Vec<(String, String)>>,
    body: T,
) -> Result<ServerResponse, String>
where
    T: Serialize,
{
    let body = serde_json::to_vec(&body);
    if let Err(e) = body {
        return Err(format!("Cannot serialize body: {}", e));
    }

    let mut headers = headers.unwrap_or_default();
    headers.push(("content-type".to_string(), "application/json".to_string()));

    create_response(status, Some(headers), Some(body.unwrap()))
}

fn create_response(
    status: u16,
    headers: Option<Vec<(String, String)>>,
    body: Option<Vec<u8>>,
) -> Result<ServerResponse, String> {
    let headers = headers.unwrap_or_default();
    let body = body.unwrap_or_default();
    Ok(ServerResponse::new(status, headers, body))
}

/// Maps the request of the serve handler to a request representation which the handlers understand
fn to_handler_request(req: &ServerRequestHeader, body: Vec<u8>) -> Result<HttpMockRequest, String> {
    let query_params = extract_query_params(&req.query);
    if let Err(e) = query_params {
        return Err(format!("error parsing query_params: {}", e));
    }

    let request = HttpMockRequest::new(req.method.to_string(), req.path.to_string())
        .with_headers(req.headers.clone())
        .with_query_params(query_params.unwrap())
        .with_body(body);

    Ok(request)
}

/// Extracts all query parameters from the URI of the given request.
fn extract_query_params(query_string: &str) -> Result<Vec<(String, String)>, String> {
    // HACK: There doesn't seem to be a way to just parse Query string with `url` crate
    // Lets just prefix a dummy URL for parsing.
    let url = format!("http://dummy?{}", query_string);
    let url = url::Url::parse(&url).map_err(|e| e.to_string())?;

    let query_params = url
        .query_pairs()
        .map(|(k, v)| (k.into(), v.into()))
        .collect();

    Ok(query_params)
}

/// Processes the response
async fn postprocess_response(
    result: Result<Option<MockServerHttpResponse>, String>,
) -> Result<Option<MockServerHttpResponse>, String> {
    if let Ok(Some(response_def)) = &result {
        if let Some(duration) = response_def.delay {
            tokio::time::sleep(duration).await;
        }
    }
    result
}
