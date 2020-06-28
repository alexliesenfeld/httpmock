use crate::server::{handlers, ServerRequestHeader, ServerResponse};

use crate::server::data::*;

use qstring::QString;
use serde::Serialize;
use std::collections::BTreeMap;

/// This route is responsible for adding a new mock
pub(crate) fn ping() -> Result<ServerResponse, String> {
    create_response(200, None, None)
}

/// This route is responsible for adding a new mock
pub(crate) fn add(state: &MockServerState, body: String) -> Result<ServerResponse, String> {
    let mock_def: serde_json::Result<MockDefinition> = serde_json::from_str(&body);
    if let Err(e) = mock_def {
        return create_json_response(500, None, ErrorResponse::new(&e));
    }
    let mock_def = mock_def.unwrap();

    let result = handlers::add_new_mock(&state, mock_def);

    match result {
        Err(e) => create_json_response(500, None, ErrorResponse::new(&e)),
        Ok(mock_id) => create_json_response(201, None, MockIdentification { mock_id }),
    }
}

/// This route is responsible for deleting mocks
pub(crate) fn delete_one(state: &MockServerState, id: usize) -> Result<ServerResponse, String> {
    let result = handlers::delete_one(state, id);
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
pub(crate) fn delete_all(state: &MockServerState) -> Result<ServerResponse, String> {
    let result = handlers::delete_all(state);
    match result {
        Err(e) => create_json_response(500, None, ErrorResponse::new(&e)),
        Ok(_) => create_response(202, None, None),
    }
}

/// This route is responsible for deleting mocks
pub(crate) fn read_one(state: &MockServerState, id: usize) -> Result<ServerResponse, String> {
    let handler_result = handlers::read_one(state, id);
    match handler_result {
        Err(e) => create_json_response(500, None, ErrorResponse { message: e }),
        Ok(mock_opt) => match mock_opt {
            Some(mock) => create_json_response(200, None, mock),
            None => create_response(404, None, None),
        },
    }
}

/// This route is responsible for finding a mock that matches the current request and serve a
/// response according to the mock specification
pub(crate) fn serve(
    state: &MockServerState,
    req: &ServerRequestHeader,
    body: String,
) -> Result<ServerResponse, String> {
    let handler_request_result = to_handler_request(&req, body);
    match handler_request_result {
        Ok(handler_request) => {
            let handler_response = handlers::find_mock(&state, handler_request);
            to_route_response(handler_response)
        }
        // TODO: Change status code 500 to something else. It is misleading to find a 500 when in fact the mock has not been found!
        Err(e) => create_json_response(500, None, ErrorResponse::new(&e)),
    }
}

/// Maps the result of the serve handler to an HTTP response which the web framework understands
fn to_route_response(
    handler_result: Result<Option<MockServerHttpResponse>, String>,
) -> Result<ServerResponse, String> {
    match handler_result {
        Err(e) => create_json_response(500 as u16, None, ErrorResponse { message: e }),
        Ok(res) => match res {
            None => create_json_response(
                500,
                None,
                ErrorResponse::new(&"Request did not match any route or mock"),
            ),
            Some(res) => create_response(res.status, res.headers, res.body),
        },
    }
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

    let mut headers = headers.unwrap_or_default();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    create_response(status, Some(headers), Some(body.unwrap()))
}

fn create_response(
    status: u16,
    headers: Option<BTreeMap<String, String>>,
    body: Option<String>,
) -> Result<ServerResponse, String> {
    let headers = headers.unwrap_or_default();
    let body = body.unwrap_or_default();
    Ok(ServerResponse::new(status, headers, body))
}

/// Maps the request of the serve handler to a request representation which the handlers understand
fn to_handler_request(
    req: &ServerRequestHeader,
    body: String,
) -> Result<MockServerHttpRequest, String> {
    let query_params = extract_query_params(&req.query);
    if let Err(e) = query_params {
        return Err(format!("error parsing query_params: {}", e));
    }

    let request = MockServerHttpRequest::new(req.method.to_string(), req.path.to_string())
        .with_headers(req.headers.clone())
        .with_query_params(query_params.unwrap())
        .with_body(body);

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
