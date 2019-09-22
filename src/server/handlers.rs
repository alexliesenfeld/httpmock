use crate::server::data::{
    ActiveMock, ApplicationState, MockDefinition, MockServerHttpRequest, MockServerHttpResponse,
    RequestRequirements,
};
use crate::server::util::{StringTreeMapExtension, TreeMapExtension};

/// Contains HTTP methods which cannot have a body.
const NON_BODY_METHODS: &'static [&str] = &["GET", "HEAD", "DELETE"];

/// Adds a new mock to the internal state.
pub fn add_new_mock(state: &ApplicationState, mock_def: MockDefinition) -> Result<usize, String> {
    let result = validate_mock_definition(&mock_def);

    if let Err(error_msg) = result {
        let error_msg = format!("validation error: {}", error_msg);
        return Err(error_msg);
    }

    let mock_id = state.create_new_id();
    {
        let mut mocks = state.mocks.write().unwrap();
        mocks.insert(mock_id, ActiveMock::new(mock_id, mock_def));
        log::trace!("Number of routes = {}", mocks.len());
    }

    return Result::Ok(mock_id);
}

/// Reads exactly one mock object.
pub fn read_one(state: &ApplicationState, id: usize) -> Result<Option<ActiveMock>, String> {
    {
        let mocks = state.mocks.read().unwrap();
        let result = mocks.get(&id);
        return match result {
            Some(found) => Ok(Some(found.clone())),
            None => Ok(None),
        };
    }
}

/// Deletes all mocks that match the request. Returns the number of deleted elements.
pub fn delete_one(state: &ApplicationState, id: usize) -> Result<bool, String> {
    let result;
    {
        let mut mocks = state.mocks.write().unwrap();
        result = mocks.remove(&id);
    }

    return Result::Ok(result.is_some());
}

/// Finds a mock that matches the current request and serve a response according to the mock
/// specification. If no mock is found, an empty result is being returned.
pub fn find_mock(
    state: &ApplicationState,
    req: MockServerHttpRequest,
) -> Result<Option<MockServerHttpResponse>, String> {
    let found_mock_id: Option<usize>;
    {
        let mocks = state.mocks.read().unwrap();
        let result = mocks
            .values()
            .into_iter()
            .find(|&mock| request_matches(&req, &mock.definition.request));

        found_mock_id = match result {
            Some(mock) => Some(mock.id),
            None => None,
        };
    }

    if let Some(found_id) = found_mock_id {
        let mut mocks = state.mocks.write().unwrap();
        let mock = mocks.get_mut(&found_id).unwrap();
        mock.call_counter += 1;
        return Ok(Some(mock.definition.response.clone()));
    }

    return Result::Ok(None);
}

/// Checks if a request matches a mock.
fn request_matches(req: &MockServerHttpRequest, mock: &RequestRequirements) -> bool {
    log::info!("Matching incoming HTTP request");
    log::trace!(
        "Matching incoming HTTP request {:?} against mock {:?}",
        req,
        mock
    );

    if let Some(path) = &mock.path {
        if !&req.path.eq(path) {
            log::debug!("Request does not match the mock (attribute: path)");
            return false;
        }
    }

    if let Some(path_contains) = &mock.path_contains {
        if !path_contains.iter().all(|pc| req.path.contains(pc)) {
            log::debug!("Request does not match the mock (attribute: path contains)");
            return false;
        }
    }

    if let Some(path_patterns) = &mock.path_matches {
        if !path_patterns
            .iter()
            .all(|pat| pat.regex.is_match(&req.path))
        {
            log::debug!("Request does not match the mock (attribute: path regex)");
            return false;
        }
    }

    if let Some(method) = &mock.method {
        if !&req.method.eq(method) {
            log::debug!("Request does not match the mock (attribute: request method)");
            return false;
        }
    }

    if !match_body_exact(&req, &mock) {
        log::debug!("Request does not match the mock (attribute: body)");
        return false;
    }

    if let Some(substrings) = &mock.body_contains {
        let matches = match &req.body {
            Some(body) => substrings.iter().all(|s| body.contains(s)),
            None => substrings.is_empty(),
        };

        if !matches {
            log::debug!("Request does not match the mock (attribute: body contains)");
            return false;
        }
    }

    if let Some(patterns) = &mock.body_matches {
        let matches = match &req.body {
            Some(body) => patterns.iter().all(|pat| pat.regex.is_match(body)),
            None => false,
        };

        if !matches {
            log::debug!("Request does not match the mock (attribute: body regex)");
            return false;
        }
    }

    if let Some(header_names) = &mock.header_exists {
        let matches = match &req.headers {
            None => false,
            Some(request_headers) => header_names
                .iter()
                .all(|h| request_headers.contains_case_insensitive_key(h)),
        };

        if !matches {
            log::debug!("Request does not match the mock (attribute: header exists)");
            return false;
        }
    }

    if !match_headers_exact(&req, &mock) {
        log::debug!("Request does not match the mock (attribute: headers)");
        return false;
    }

    if let Some(query_param_names) = &mock.query_param_exists {
        let matches = match &req.query_params {
            None => false,
            Some(param_names) => query_param_names
                .iter()
                .all(|p| param_names.contains_key(p)),
        };

        if !matches {
            log::debug!("Request does not match the mock (attribute: query param exists)");
            return false;
        }
    }

    if !match_query_params_exact(&req, &mock) {
        log::debug!("Request does not match the mock (attribute: query param)");
        return false;
    }

    log::debug!("Request matched!");

    true
}

/// Matches headers from a request and a mock. Matches header names case-insensitive:
/// From RFC 2616 - "Hypertext Transfer Protocol -- HTTP/1.1", Section 4.2, "Message Headers":
/// "Each header field consists of a name followed by a colon (":") and the field value.
/// Field names are case-insensitive."
fn match_headers_exact(req: &MockServerHttpRequest, mock: &RequestRequirements) -> bool {
    return match (&req.headers, &mock.headers) {
        (Some(m1), Some(m2)) => m1.contains_with_case_insensitive_key(m2),
        (Some(_), None) => true,
        (None, Some(m2)) => m2.is_empty(),
        (None, None) => true,
    };
}

/// Matches query params from a request and a mock
fn match_query_params_exact(req: &MockServerHttpRequest, mock: &RequestRequirements) -> bool {
    return match (&req.query_params, &mock.query_param) {
        (Some(m1), Some(m2)) => m1.contains(m2),
        (Some(_), None) => true,
        (None, Some(m2)) => m2.is_empty(),
        (None, None) => true,
    };
}

/// Matches body
fn match_body_exact(req: &MockServerHttpRequest, mock: &RequestRequirements) -> bool {
    return match (&req.body, &mock.body) {
        (Some(rb), Some(mb)) => rb.eq(mb),
        (None, Some(mb)) => mb.is_empty(),
        (Some(rb), None) => rb.is_empty(),
        (None, None) => true,
    };
}

/// Validates a mock request.
fn validate_mock_definition(req: &MockDefinition) -> Result<(), String> {
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
    use crate::server::data::{
        MockDefinition, MockServerHttpRequest, MockServerHttpResponse, RequestRequirements,
    };
    use crate::server::handlers::{request_matches, validate_mock_definition};
    use std::collections::BTreeMap;

    /// TODO
    #[test]
    fn header_names_case_insensitive() {}

    /// TODO
    #[test]
    fn parsing_query_params_test() {}

    /// TODO
    #[test]
    fn parsing_query_contains_test() {}

    /// TODO
    #[test]
    fn header_exists_test() {}

    /// TODO
    #[test]
    fn path_contains_test() {}

    /// TODO
    #[test]
    fn path_pattern_test() {}

    /// TODO
    #[test]
    fn body_contains_test() {}

    /// TODO
    #[test]
    fn body_matches_regex_test() {}

    /// This test makes sure that a request is considered "matched" if the paths of the
    /// request and the mock are equal.
    #[test]
    fn request_matches_path_match() {
        // Arrange
        let req1: MockServerHttpRequest = MockServerHttpRequest::builder()
            .path("/test-path".to_string())
            .method("GET") // mandatory attribute
            .build();

        let req2: RequestRequirements = RequestRequirements::builder()
            .path(Some("/test-path".to_string()))
            .build();

        // Act
        let does_match = request_matches(&req1, &req2);

        // Assert
        assert_eq!(true, does_match);
    }

    /// This test makes sure that a request is considered "not matched" if the paths of the
    /// request and the mock are not equal.
    #[test]
    fn request_matches_path_no_match() {
        // Arrange
        let req1: MockServerHttpRequest = MockServerHttpRequest::builder()
            .path("/test-path".to_string())
            .method("GET") // mandatory attribute
            .build();

        let req2: RequestRequirements = RequestRequirements::builder()
            .path(Some("/another-path".to_string()))
            .build();

        // Act
        let does_match = request_matches(&req1, &req2);

        // Assert
        assert_eq!(false, does_match);
    }

    /// This test makes sure that a request is considered "matched" if the methods of the
    /// request and the mock are equal.
    #[test]
    fn request_matches_method_match() {
        // Arrange
        let req1: MockServerHttpRequest = MockServerHttpRequest::builder()
            .path("/test") // mandatory attribute
            .method("GET".to_string())
            .build();

        let req2: RequestRequirements = RequestRequirements::builder()
            .method("GET".to_string())
            .build();

        // Act
        let does_match = request_matches(&req1, &req2);

        // Assert
        assert_eq!(true, does_match);
    }

    /// This test makes sure that a request is considered "not matched" if the methods of the
    /// request and the mock are not equal.
    #[test]
    fn request_matches_method_no_match() {
        // Arrange
        let req1: MockServerHttpRequest = MockServerHttpRequest::builder()
            .path("/test") // mandatory attribute
            .method("GET".to_string())
            .build();

        let req2: RequestRequirements = RequestRequirements::builder()
            .method(Some("POST".to_string()))
            .build();

        // Act
        let does_match = request_matches(&req1, &req2);

        // Assert
        assert_eq!(false, does_match);
    }

    /// This test makes sure that a request is considered "matched" if the bodies of both,
    /// the request and the mock are present and have equal content.
    #[test]
    fn request_matches_body_match() {
        // Arrange
        let req1: MockServerHttpRequest = MockServerHttpRequest::builder()
            .body("test".to_string())
            .path("/test") // mandatory attribute
            .method("GET".to_string()) // mandatory attribute
            .build();

        let req2: RequestRequirements = RequestRequirements::builder()
            .body("test".to_string())
            .build();

        // Act
        let does_match = request_matches(&req1, &req2);

        // Assert
        assert_eq!(true, does_match);
    }

    /// This test makes sure that a request is considered "not matched" if the bodies of both,
    /// the request and the mock are present, but do have different content.
    #[test]
    fn request_matches_body_no_match() {
        // Arrange
        let req1: MockServerHttpRequest = MockServerHttpRequest::builder()
            .path("/test") // mandatory attribute
            .method("GET".to_string()) // mandatory attribute
            .body(Some("some text".to_string()))
            .build();

        let req2: RequestRequirements = RequestRequirements::builder()
            .body(Some("some other text".to_string()))
            .build();

        // Act
        let does_match = request_matches(&req1, &req2);

        // Assert
        assert_eq!(false, does_match);
    }

    /// This test makes sure that a request is considered "not matched" if the body of the request
    /// is present but the mock does not expect a body.
    #[test]
    fn request_matches_body_no_match_empty() {
        // Arrange
        let req1: MockServerHttpRequest = MockServerHttpRequest::builder()
            .path("/test") // mandatory attribute
            .method("GET".to_string()) // mandatory attribute
            .body("text".to_string())
            .build();

        let req2: RequestRequirements = RequestRequirements::builder().build();

        // Act
        let does_match = request_matches(&req1, &req2);

        // Assert
        assert_eq!(false, does_match);
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

        let req1: MockServerHttpRequest = MockServerHttpRequest::builder()
            .path("/test") // mandatory attribute
            .method("GET".to_string()) // mandatory attribute
            .headers(Some(h1))
            .build();

        let req2: RequestRequirements = RequestRequirements::builder().headers(Some(h2)).build();

        // Act
        let does_match = request_matches(&req1, &req2);

        // Assert
        assert_eq!(true, does_match);
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

        let req1: MockServerHttpRequest = MockServerHttpRequest::builder()
            .path("/test") // mandatory attribute
            .method("GET".to_string()) // mandatory attribute
            .headers(Some(h1))
            .build();

        let req2: RequestRequirements = RequestRequirements::builder().headers(Some(h2)).build();

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

        let req1: MockServerHttpRequest = MockServerHttpRequest::builder()
            .path("/test") // mandatory attribute
            .method("GET".to_string()) // mandatory attribute
            .headers(Some(h1))
            .build();

        let req2: RequestRequirements = RequestRequirements::builder().headers(Some(h2)).build();

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

        let req: MockServerHttpRequest = MockServerHttpRequest::builder()
            .path("/test") // mandatory attribute
            .method("GET".to_string()) // mandatory attribute
            .headers(Some(req_headers))
            .build();

        let mock: RequestRequirements = RequestRequirements::builder().headers(None).build();

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
        let req1: MockServerHttpRequest = MockServerHttpRequest::builder()
            .path("/test") // mandatory attribute
            .method("GET".to_string()) // mandatory attribute
            .headers(None)
            .build();

        let req2: RequestRequirements = RequestRequirements::builder().headers(None).build();

        // Act
        let does_match = request_matches(&req1, &req2);

        // Assert
        assert_eq!(true, does_match);
    }

    /// This test ensures that mock request cannot contain a request method that cannot
    /// be sent along with a request body.
    #[test]
    fn validate_mock_definition_no_body_method() {
        // Arrange
        let req: RequestRequirements = RequestRequirements::builder()
            .path("/test".to_string())
            .method("GET".to_string())
            .body(Some("test".to_string()))
            .build();

        let res: MockServerHttpResponse =
            MockServerHttpResponse::builder().status(418 as u16).build();

        let smr: MockDefinition = MockDefinition::builder().request(req).response(res).build();

        // Act
        let result = validate_mock_definition(&smr);

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
    fn validate_mock_definition_no_path() {
        // Arrange
        let req: RequestRequirements = RequestRequirements::builder().build();

        let res: MockServerHttpResponse =
            MockServerHttpResponse::builder().status(418 as u16).build();

        let smr: MockDefinition = MockDefinition::builder().request(req).response(res).build();

        // Act
        let result = validate_mock_definition(&smr);

        // Assert
        assert_eq!(true, result.is_err());
        assert_eq!(true, result.unwrap_err().eq("You need to provide a path"));
    }
}
