use crate::handlers::{HttpMockRequest, HttpMockResponse, HttpMockState};
use crate::util::http::NON_BODY_METHODS;
use crate::util::std::{EqNoneAsEmpty, TreeMapOptExtension};
use serde::{Deserialize, Serialize};

/// Lists all mocks.
pub fn list_all(state: &HttpMockState) -> Result<Vec<SetMockRequest>, String> {
    {
        let mocks = state.mocks.read().unwrap();
        return Result::Ok(mocks.clone());
    }
}

/// Adds a new mock to the internal state.
pub fn add_new_mock(state: &HttpMockState, req: SetMockRequest) -> Result<(), String> {
    let result = validate_mock_request(&req);

    if let Err(error_msg) = result {
        let error_msg = format!("validation error: {}", error_msg);
        return Err(error_msg);
    }

    {
        let mut mocks = state.mocks.write().unwrap();
        mocks.push(req);
        log::debug!("Number of routes = {}", mocks.len());
    }

    return Result::Ok(());
}

/// A Request that is made to set a new mock.
#[derive(Serialize, Deserialize, TypedBuilder, Clone, Debug)]
pub struct SetMockRequest {
    pub request: HttpMockRequest,
    pub response: HttpMockResponse,
}

/// Clears all mocks from the internal state.
pub fn clear_mocks(state: &HttpMockState) -> Result<(), &'static str> {
    {
        let mut mocks = state.mocks.write().unwrap();
        mocks.clear();
    }

    return Result::Ok(());
}

/// Finds a mock that matches the current request and serve a response according to the mock
/// specification. If no mock is found, an empty result is being returned.
pub fn find_mock(
    state: &HttpMockState,
    req: HttpMockRequest,
) -> Result<Option<HttpMockResponse>, &'static str> {
    {
        let mocks = state.mocks.read().unwrap();
        let result = mocks.iter().find(|&m| request_matches(&req, &m.request));

        if let Some(found) = result {
            return Ok(Some(found.response.clone()));
        }
    }

    return Result::Ok(None);
}

/// Checks if a request matches a mock.
fn request_matches(req: &HttpMockRequest, mock: &HttpMockRequest) -> bool {
    if !&mock.path.eq(&req.path) {
        return false;
    }

    if !&mock.method.eq(&req.method) {
        return false;
    }

    if !req.headers.contains_opt(&mock.headers) {
        return false;
    }

    if !&mock.body.eq_none_as_default(&req.body) {
        return false;
    }

    true
}

/// Validates a mock request.
fn validate_mock_request(req: &SetMockRequest) -> Result<(), String> {
    if req.request.path.is_none() || req.request.path.as_ref().unwrap().trim().is_empty() {
        return Err(String::from("You need to provide a path"));
    }

    if let Some(_body) = &req.request.body {
        if let Some(method) = &req.request.method {
            if NON_BODY_METHODS.contains(&method.as_str()) {
                return Err(String::from(
                    "A body cannot be sent along with the specified method",
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::handlers::mocks::{request_matches, validate_mock_request, SetMockRequest};
    use crate::handlers::{HttpMockRequest, HttpMockResponse};
    use std::collections::BTreeMap;

    /// This test makes sure that a request is considered "matched" if the paths of the
    /// request and the mock are equal.
    #[test]
    fn request_matches_path_match() {
        // Arrange
        let req1: HttpMockRequest = HttpMockRequest::builder()
            .path(Some("/test-path".to_string()))
            .build();

        let req2: HttpMockRequest = HttpMockRequest::builder()
            .path(Some("/test-path".to_string()))
            .build();

        // Act
        let does_match_1 = request_matches(&req1, &req2);
        let does_match_2 = request_matches(&req2, &req1);

        // Assert
        assert_eq!(true, does_match_1);
        assert_eq!(true, does_match_2);
    }

    /// This test makes sure that a request is considered "not matched" if the paths of the
    /// request and the mock are not equal.
    #[test]
    fn request_matches_path_no_match() {
        // Arrange
        let req1: HttpMockRequest = HttpMockRequest::builder()
            .path(Some("/test-path".to_string()))
            .build();

        let req2: HttpMockRequest = HttpMockRequest::builder()
            .path(Some("/another-path".to_string()))
            .build();

        // Act
        let does_match_1 = request_matches(&req1, &req2);
        let does_match_2 = request_matches(&req2, &req1);

        // Assert
        assert_eq!(false, does_match_1);
        assert_eq!(false, does_match_2);
    }

    /// This test makes sure that a request is considered "not matched" if the path of the
    /// request is not set and that of the mock is set.
    ///
    /// TODO: Consider to remove this test because this case does not appear due to a
    /// request validation step.
    #[test]
    fn request_matches_path_no_match_empty() {
        // Arrange
        let req1: HttpMockRequest = HttpMockRequest::builder().build();

        let req2: HttpMockRequest = HttpMockRequest::builder()
            .path(Some("/another-path".to_string()))
            .build();

        // Act
        let does_match_1 = request_matches(&req1, &req2);
        let does_match_2 = request_matches(&req2, &req1);

        // Assert
        assert_eq!(false, does_match_1);
        assert_eq!(false, does_match_2);
    }

    /// This test makes sure that a request is considered "matched" if the methods of the
    /// request and the mock are equal.
    #[test]
    fn request_matches_method_match() {
        // Arrange
        let req1: HttpMockRequest = HttpMockRequest::builder()
            .method(Some("GET".to_string()))
            .build();

        let req2: HttpMockRequest = HttpMockRequest::builder()
            .method(Some("GET".to_string()))
            .build();

        // Act
        let _does_match = request_matches(&req1, &req2);

        // Assert
        let does_match_1 = request_matches(&req1, &req2);
        let does_match_2 = request_matches(&req2, &req1);

        // Assert
        assert_eq!(true, does_match_1);
        assert_eq!(true, does_match_2);
    }

    /// This test makes sure that a request is considered "not matched" if the methods of the
    /// request and the mock are not equal.
    #[test]
    fn request_matches_method_no_match() {
        // Arrange
        let req1: HttpMockRequest = HttpMockRequest::builder()
            .method(Some("GET".to_string()))
            .build();

        let req2: HttpMockRequest = HttpMockRequest::builder()
            .method(Some("POST".to_string()))
            .build();

        // Act
        let does_match_1 = request_matches(&req1, &req2);
        let does_match_2 = request_matches(&req2, &req1);

        // Assert
        assert_eq!(false, does_match_1);
        assert_eq!(false, does_match_2);
    }

    /// This test makes sure that a request is considered "matched" if the method of the request
    /// is present but the mock does not.
    ///
    /// TODO: This test is obsolete when HttpMockRequest is refactored to contain "method" a
    /// non-optional attribute (refactoring should address that Mocks and Requests should have
    /// different field requirements, i.e. optional/non-optional).
    #[test]
    fn request_matches_method_no_match_empty() {
        // Arrange
        let req1: HttpMockRequest = HttpMockRequest::builder()
            .method(Some("GET".to_string()))
            .build();

        let req2: HttpMockRequest = HttpMockRequest::builder().build();

        // Act
        let does_match_1 = request_matches(&req1, &req2);
        let does_match_2 = request_matches(&req2, &req1);

        // Assert
        assert_eq!(false, does_match_1);
        assert_eq!(false, does_match_2);
    }

    /// This test makes sure that a request is considered "matched" if the bodies of both,
    /// the request and the mock are present and have equal content.
    #[test]
    fn request_matches_body_match() {
        // Arrange
        let req1: HttpMockRequest = HttpMockRequest::builder()
            .body(Some("test".to_string()))
            .build();

        let req2: HttpMockRequest = HttpMockRequest::builder()
            .body(Some("test".to_string()))
            .build();

        // Act
        let does_match_1 = request_matches(&req1, &req2);
        let does_match_2 = request_matches(&req2, &req1);

        // Assert
        assert_eq!(true, does_match_1);
        assert_eq!(true, does_match_2);
    }

    /// This test makes sure that a request is considered "not matched" if the bodies of both,
    /// the request and the mock are present, but do have different content.
    #[test]
    fn request_matches_body_no_match() {
        // Arrange
        let req1: HttpMockRequest = HttpMockRequest::builder()
            .body(Some("some text".to_string()))
            .build();

        let req2: HttpMockRequest = HttpMockRequest::builder()
            .body(Some("some other text".to_string()))
            .build();

        // Act
        let does_match_1 = request_matches(&req1, &req2);
        let does_match_2 = request_matches(&req2, &req1);

        // Assert
        assert_eq!(false, does_match_1);
        assert_eq!(false, does_match_2);
    }

    /// This test makes sure that a request is considered "not matched" if the body of the request
    /// is present but the mock does not expect a body.
    #[test]
    fn request_matches_body_no_match_empty() {
        // Arrange
        let req1: HttpMockRequest = HttpMockRequest::builder()
            .body(Some("text".to_string()))
            .build();

        let req2: HttpMockRequest = HttpMockRequest::builder().build();

        // Act
        let does_match_1 = request_matches(&req1, &req2);
        let does_match_2 = request_matches(&req2, &req1);

        // Assert
        assert_eq!(false, does_match_1);
        assert_eq!(false, does_match_2);
    }

    /// This test makes sure that a request is considered "matched" if the bodies of both, the
    /// request and the mock, are not present.
    #[test]
    fn request_matches_body_match_empty() {
        // Arrange
        let req1: HttpMockRequest = HttpMockRequest::builder().build();

        let req2: HttpMockRequest = HttpMockRequest::builder().build();

        // Act
        let does_match_1 = request_matches(&req1, &req2);
        let does_match_2 = request_matches(&req2, &req1);

        // Assert
        assert_eq!(true, does_match_1);
        assert_eq!(true, does_match_2);
    }

    /// This test makes sure that a request is considered "matched" when the request contains
    /// exactly the same as the mock expects.
    #[test]
    fn request_matches_headers_exact_match() {
        // Arrange
        let mut h1 = BTreeMap::new();
        h1.insert("h1".to_string(), "v1".to_string());
        h1.insert("h2".to_string(), "v2".to_string());

        let mut h2 = BTreeMap::new();
        h2.insert("h1".to_string(), "v1".to_string());
        h2.insert("h2".to_string(), "v2".to_string());

        let req1: HttpMockRequest = HttpMockRequest::builder().headers(Some(h1)).build();

        let req2: HttpMockRequest = HttpMockRequest::builder().headers(Some(h2)).build();

        // Act
        let does_match_1 = request_matches(&req1, &req2);
        let does_match_2 = request_matches(&req2, &req1);

        // Assert
        assert_eq!(true, does_match_1);
        assert_eq!(true, does_match_2);
    }

    /// This test makes sure that a request is considered "not matched" when the request misses
    /// headers.
    #[test]
    fn request_matches_headers_no_match() {
        // Arrange
        let mut h1 = BTreeMap::new();
        h1.insert("h1".to_string(), "v1".to_string());

        let mut h2 = BTreeMap::new();
        h2.insert("h1".to_string(), "v1".to_string());
        h2.insert("h2".to_string(), "v2".to_string());

        let req1: HttpMockRequest = HttpMockRequest::builder().headers(Some(h1)).build();

        let req2: HttpMockRequest = HttpMockRequest::builder().headers(Some(h2)).build();

        // Act
        let does_match = request_matches(&req1, &req2);

        // Assert
        assert_eq!(false, does_match); // Request misses header "h2"
    }

    /// This test makes sure that even the headers of a mock and a request differ,
    /// the request still is considered "matched" when the request does contain more than
    /// all expected headers that. Hence a request is allowed to contain headers that a mock
    /// does not.
    #[test]
    fn request_matches_headers_match_superset() {
        // Arrange
        let mut h1 = BTreeMap::new();
        h1.insert("h1".to_string(), "v1".to_string());
        h1.insert("h2".to_string(), "v2".to_string());

        let mut h2 = BTreeMap::new();
        h2.insert("h1".to_string(), "v1".to_string());

        let req1: HttpMockRequest = HttpMockRequest::builder().headers(Some(h1)).build();

        let req2: HttpMockRequest = HttpMockRequest::builder().headers(Some(h2)).build();

        // Act
        let does_match = request_matches(&req1, &req2);

        // Assert
        assert_eq!(true, does_match); // matches, because request contains more headers than the mock expects
    }

    /// This test makes sure that even the headers of a mock and a request differ,
    /// the request still is considered "matched" when the mock does not expect any headers
    /// at all. Hence a request is allowed to contain headers that a mock does not.
    #[test]
    fn request_matches_headers_no_match_empty() {
        // Arrange
        let mut req_headers = BTreeMap::new();
        req_headers.insert("req_headers".to_string(), "v1".to_string());
        req_headers.insert("h2".to_string(), "v2".to_string());

        let req: HttpMockRequest = HttpMockRequest::builder()
            .headers(Some(req_headers))
            .build();

        let mock: HttpMockRequest = HttpMockRequest::builder().headers(None).build();

        // Act
        let does_match_1 = request_matches(&req, &mock);

        // Assert
        assert_eq!(true, does_match_1); // effectively empty because mock does not expect any headers
    }

    /// This test makes sure no present headers on both sides, the mock and the request, are
    /// considered equal.
    #[test]
    fn request_matches_headers_match_empty() {
        // Arrange
        let req1: HttpMockRequest = HttpMockRequest::builder().headers(None).build();

        let req2: HttpMockRequest = HttpMockRequest::builder().headers(None).build();

        // Act
        let does_match_1 = request_matches(&req1, &req2);
        let does_match_2 = request_matches(&req2, &req1);

        // Assert
        assert_eq!(true, does_match_1);
        assert_eq!(true, does_match_2);
    }

    /// This test ensures that mock request cannot contain a request method that cannot
    /// be sent along with a request body.
    #[test]
    fn validate_mock_request_no_body_method() {
        // Arrange
        let req: HttpMockRequest = HttpMockRequest::builder()
            .path(Some("/test".to_string()))
            .method(Some("GET".to_string()))
            .body(Some("test".to_string()))
            .build();

        let res: HttpMockResponse = HttpMockResponse::builder().status(418 as u16).build();

        let smr: SetMockRequest = SetMockRequest::builder().request(req).response(res).build();

        // Act
        let result = validate_mock_request(&smr);

        // Assert
        assert_eq!(true, result.is_err());
        assert_eq!(
            true,
            result
                .unwrap_err()
                .eq("A body cannot be sent along with the specified method")
        );
    }

    /// This test ensures that mock request cannot contain an empty path.
    #[test]
    fn validate_mock_request_no_path() {
        // Arrange
        let req: HttpMockRequest = HttpMockRequest::builder().build();

        let res: HttpMockResponse = HttpMockResponse::builder().status(418 as u16).build();

        let smr: SetMockRequest = SetMockRequest::builder().request(req).response(res).build();

        // Act
        let result = validate_mock_request(&smr);

        // Assert
        assert_eq!(true, result.is_err());
        assert_eq!(true, result.unwrap_err().eq("You need to provide a path"));
    }
}
