use std::rc::Rc;
use std::str::FromStr;

use assert_json_diff::{assert_json_eq_no_panic, assert_json_include_no_panic};
use serde_json::Value;

use crate::server::data::{
    ActiveMock, MockDefinition, MockServerHttpRequest, MockServerHttpResponse, MockServerState,
    RequestRequirements,
};
use crate::server::util::{StringTreeMapExtension, TreeMapExtension};
use basic_cookies::Cookie;

/// Contains HTTP methods which cannot have a body.
const NON_BODY_METHODS: &[&str] = &["GET", "HEAD", "DELETE"];

/// Adds a new mock to the internal state.
pub(crate) fn add_new_mock(state: &MockServerState, mock_def: MockDefinition) -> Result<usize, String> {
    let result = validate_mock_definition(&mock_def);

    if let Err(error_msg) = result {
        let error_msg = format!("Validation error: {}", error_msg);
        return Err(error_msg);
    }

    let mock_id = state.create_new_id();
    {
        log::debug!("Adding new mock with ID={}: {:?}", mock_id, mock_def);
        let mut mocks = state.mocks.write().unwrap();
        mocks.insert(mock_id, ActiveMock::new(mock_id, mock_def));
    }

    Result::Ok(mock_id)
}

/// Reads exactly one mock object.
pub(crate) fn read_one(state: &MockServerState, id: usize) -> Result<Option<ActiveMock>, String> {
    {
        let mocks = state.mocks.read().unwrap();
        let result = mocks.get(&id);
        match result {
            Some(found) => Ok(Some(found.clone())),
            None => Ok(None),
        }
    }
}

/// Deletes one mock by id. Returns the number of deleted elements.
pub(crate) fn delete_one(state: &MockServerState, id: usize) -> Result<bool, String> {
    let result;
    {
        let mut mocks = state.mocks.write().unwrap();
        result = mocks.remove(&id);
    }

    log::debug!("Deleted mock with id={}", id);
    Result::Ok(result.is_some())
}

/// Deletes all mocks. Returns the number of deleted elements.
pub(crate) fn delete_all(state: &MockServerState) -> Result<usize, String> {
    let result;
    {
        let mut mocks = state.mocks.write().unwrap();
        result = mocks.len();
        mocks.clear();
    }

    log::trace!("Deleted all mocks");
    Result::Ok(result)
}

/// Finds a mock that matches the current request and serve a response according to the mock
/// specification. If no mock is found, an empty result is being returned.
pub(crate) fn find_mock(
    state: &MockServerState,
    req: MockServerHttpRequest,
) -> Result<Option<MockServerHttpResponse>, String> {
    // TODO: Use reference instead of Rc
    let req = Rc::new(req);

    let found_mock_id: Option<usize>;
    {
        let mocks = state.mocks.read().unwrap();
        let result = mocks
            .values()
            .find(|&mock| request_matches(req.clone(), &mock.definition.request));

        found_mock_id = match result {
            Some(mock) => Some(mock.id),
            None => None,
        };
    }

    if let Some(found_id) = found_mock_id {
        log::debug!(
            "Matched mock with id={} to the following request: {:?}",
            found_id,
            req
        );
        let mut mocks = state.mocks.write().unwrap();
        let mock = mocks.get_mut(&found_id).unwrap();
        mock.call_counter += 1;
        return Ok(Some(mock.definition.response.clone()));
    }

    log::debug!(
        "Could not match any mock to the following request: {:?}",
        req
    );
    Result::Ok(None)
}

/// Checks if a request matches a mock.
fn request_matches(req: Rc<MockServerHttpRequest>, mock: &RequestRequirements) -> bool {
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

    if mock.body.is_some() && !match_body_exact(&req, &mock) {
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

    if let Some(expected_json_body) = &mock.json_body {
        if !match_json(&req.body, expected_json_body, true) {
            log::debug!("Request does not match the mock (attribute: body)");
            return false;
        }
    }

    if let Some(expected_json_includes) = &mock.json_body_includes {
        for expected_include_value in expected_json_includes {
            if !match_json(&req.body, expected_include_value, false) {
                log::debug!("Request does not match the mock (attribute: body)");
                return false;
            }
        }
    }

    if let Some(expected_headers) = &mock.header_exists {
        let matches = match &req.headers {
            None => false,
            Some(request_headers) => expected_headers
                .iter()
                .all(|eh| request_headers.contains_case_insensitive_key(eh)),
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

    if let Some(expected_cookie_names) = &mock.cookie_exists {
        let matches = expected_cookie_names
            .iter()
            .all(|name| contains_cookie(&req, name, None));

        if !matches {
            log::debug!("Request does not match the mock (attribute: cookie exists)");
            return false;
        }
    }

    if let Some(expected_cookies) = &mock.cookies {
        let matches = expected_cookies
            .iter()
            .all(|(name, value)| contains_cookie(&req, name, Some(value)));

        if !matches {
            log::debug!("Request does not match the mock (attribute: cookie name with value)");
            return false;
        }
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

    if let Some(matchers) = &mock.matchers {
        for (idx, matcher) in matchers.iter().enumerate() {
            if !(matcher)(req.clone()) {
                log::debug!("Request does not match the mock (attribute: custom closure/matcher, index: {})", idx);
                return false;
            }
        }
    }

    true
}

/// Matches headers from a request and a mock. Matches header names case-insensitive:
/// From RFC 2616 - "Hypertext Transfer Protocol -- HTTP/1.1", Section 4.2, "Message Headers":
/// "Each header field consists of a name followed by a colon (":") and the field value.
/// Field names are case-insensitive."
fn match_headers_exact(req: &MockServerHttpRequest, mock: &RequestRequirements) -> bool {
    match (&req.headers, &mock.headers) {
        (Some(m1), Some(m2)) => m1.contains_with_case_insensitive_key(m2),
        (Some(_), None) => true,
        (None, Some(m2)) => m2.is_empty(),
        (None, None) => true,
    }
}

/// Matches query params from a request and a mock
fn match_query_params_exact(req: &MockServerHttpRequest, mock: &RequestRequirements) -> bool {
    match (&req.query_params, &mock.query_param) {
        (Some(m1), Some(m2)) => m1.contains(m2),
        (Some(_), None) => true,
        (None, Some(m2)) => m2.is_empty(),
        (None, None) => true,
    }
}

/// Matches body
fn match_body_exact(req: &MockServerHttpRequest, mock: &RequestRequirements) -> bool {
    match (&req.body, &mock.body) {
        (Some(rb), Some(mb)) => rb.eq(mb),
        (None, Some(mb)) => mb.is_empty(),
        (Some(rb), None) => rb.is_empty(),
        (None, None) => true,
    }
}

/// Matches JSON
fn match_json(req: &Option<String>, mock: &Value, exact: bool) -> bool {
    match req {
        Some(req_string) => {
            // Parse the request body as JSON string
            let result = serde_json::Value::from_str(req_string);
            if let Err(e) = result {
                log::trace!("cannot deserialize request body to JSON: {}", e);
                return false;
            }
            let req_value = result.unwrap();

            log::trace!(
                "Comapring the following JSON values: (1){}, (2){}",
                &req_value,
                &mock
            );

            // Compare JSON values
            let result = if exact {
                assert_json_eq_no_panic(&req_value, mock)
            } else {
                assert_json_include_no_panic(&req_value, mock)
            };

            // Log and return the comparison result
            match result {
                Err(e) => {
                    log::trace!("Request body does not match mock JSON body: {}", e);
                    false
                }
                _ => {
                    log::trace!("Request body matched mock JSON body");
                    true
                }
            }
        }
        None => false,
    }
}

fn contains_cookie(
    req: &MockServerHttpRequest,
    expected_cookie_name: &str,
    expected_cookie_value: Option<&str>,
) -> bool {
    let expected_cookie_name = expected_cookie_name.to_lowercase();
    return match &req.headers {
        None => false,
        Some(request_headers) => {
            let cookie_header = request_headers
                .iter()
                .find(|(k, _)| k.to_lowercase().eq("cookie"));
            return match cookie_header {
                None => false,
                Some((_, val)) => {
                    let cookie_parse_result = Cookie::parse(val);
                    return match cookie_parse_result {
                        Err(e) => {
                            log::warn!("Cannot parse request cookie: {}", e);
                            false
                        }
                        Ok(req_cookies) => {
                            let found_cookie = req_cookies
                                .iter()
                                .find(|e| e.get_name().to_lowercase().eq(&expected_cookie_name));
                            match (found_cookie, expected_cookie_value) {
                                (None, _) => false,
                                (Some(_), None) => true,
                                (Some(cookie), Some(val)) => return cookie.get_value().eq(val),
                            }
                        }
                    };
                }
            };
        }
    };
}

/// Validates a mock request.
fn validate_mock_definition(req: &MockDefinition) -> Result<(), String> {
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
    use std::collections::BTreeMap;
    use std::rc::Rc;

    use crate::server::data::{
        MockDefinition, MockServerHttpRequest, MockServerHttpResponse, Pattern, RequestRequirements,
    };
    use crate::server::handlers::{
        add_new_mock, read_one, request_matches, validate_mock_definition,
    };
    use crate::server::MockServerState;
    use regex::Regex;

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

    #[test]
    fn body_contains_test() {
        // Arrange
        let request1 = MockServerHttpRequest::new("GET".to_string(), "/test-path".to_string())
            .with_body("test".to_string());
        let request2 = MockServerHttpRequest::new("GET".to_string(), "/test-path".to_string())
            .with_body("test".to_string());

        let requirements1 = RequestRequirements::new().with_body_contains(vec!["xxx".to_string()]);
        let requirements2 = RequestRequirements::new().with_body_contains(vec!["es".to_string()]);

        // Act
        let does_match1 = request_matches(Rc::new(request1), &requirements1);
        let does_match2 = request_matches(Rc::new(request2), &requirements2);

        // Assert
        assert_eq!(false, does_match1);
        assert_eq!(true, does_match2);
    }

    #[test]
    fn body_matches_query_params_exact_test() {
        // Arrange
        let mut params1 = BTreeMap::new();
        params1.insert("k".to_string(), "v".to_string());

        let mut params2 = BTreeMap::new();
        params2.insert("h".to_string(), "o".to_string());

        let request1 = MockServerHttpRequest::new("GET".to_string(), "/test-path".to_string())
            .with_query_params(params1.clone());
        let request2 = MockServerHttpRequest::new("GET".to_string(), "/test-path".to_string())
            .with_query_params(params1.clone());

        let requirements1 = RequestRequirements::new().with_query_param(params2);
        let requirements2 = RequestRequirements::new().with_query_param(params1.clone());

        // Act
        let does_match1 = request_matches(Rc::new(request1), &requirements1);
        let does_match2 = request_matches(Rc::new(request2), &requirements2);

        // Assert
        assert_eq!(false, does_match1);
        assert_eq!(true, does_match2);
    }

    /// TODO
    #[test]
    fn body_contains_includes_json_test() {}

    /// TODO
    #[test]
    fn body_json_exact_match_test() {}

    /// This test makes sure that a request is considered "matched" if the paths of the
    /// request and the mock are equal.
    #[test]
    fn request_matches_path_match() {
        // Arrange
        let req1 = MockServerHttpRequest::new("GET".to_string(), "/test-path".to_string());

        let req2 = RequestRequirements::new().with_path("/test-path".to_string());

        // Act
        let does_match = request_matches(Rc::new(req1), &req2);

        // Assert
        assert_eq!(true, does_match);
    }

    /// This test makes sure that a request is considered "not matched" if the paths of the
    /// request and the mock are not equal.
    #[test]
    fn request_matches_path_no_match() {
        // Arrange
        let req1 = MockServerHttpRequest::new("GET".to_string(), "/test-path".to_string());

        let req2 = RequestRequirements::new().with_path("/another-path".to_string());

        // Act
        let does_match = request_matches(Rc::new(req1), &req2);

        // Assert
        assert_eq!(false, does_match);
    }

    /// This test makes sure that a request is considered "matched" if the methods of the
    /// request and the mock are equal.
    #[test]
    fn request_matches_method_match() {
        // Arrange
        let req1 = MockServerHttpRequest::new("GET".to_string(), "/test".to_string());

        let req2 = RequestRequirements::new().with_method("GET".to_string());

        // Act
        let does_match = request_matches(Rc::new(req1), &req2);

        // Assert
        assert_eq!(true, does_match);
    }

    /// This test makes sure that a request is considered "not matched" if the methods of the
    /// request and the mock are not equal.
    #[test]
    fn request_matches_method_no_match() {
        // Arrange
        let req1 = MockServerHttpRequest::new("GET".to_string(), "/test".to_string());

        let req2 = RequestRequirements::new().with_method("POST".to_string());

        // Act
        let does_match = request_matches(Rc::new(req1), &req2);

        // Assert
        assert_eq!(false, does_match);
    }

    /// This test makes sure that a request is considered "matched" if the bodies of both,
    /// the request and the mock are present and have equal content.
    #[test]
    fn request_matches_body_match() {
        // Arrange
        let req1 = MockServerHttpRequest::new("GET".to_string(), "/test".to_string())
            .with_body("test".to_string());

        let req2 = RequestRequirements::new().with_body("test".to_string());

        // Act
        let does_match = request_matches(Rc::new(req1), &req2);

        // Assert
        assert_eq!(true, does_match);
    }

    /// This test makes sure that a request is considered "not matched" if the bodies of both,
    /// the request and the mock are present, but do have different content.
    #[test]
    fn request_matches_body_no_match() {
        // Arrange
        let req1 = MockServerHttpRequest::new("GET".to_string(), "/test".to_string())
            .with_body("some text".to_string());

        let req2 = RequestRequirements::new().with_body("some other text".to_string());

        // Act
        let does_match = request_matches(Rc::new(req1), &req2);

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

        let req1 =
            MockServerHttpRequest::new("GET".to_string(), "/test".to_string()).with_headers(h1);

        let req2 = RequestRequirements::new().with_headers(h2);

        // Act
        let does_match = request_matches(Rc::new(req1), &req2);

        // Assert
        assert_eq!(true, does_match);
    }

    /// This test makes sure that a request is considered "not matched" when the request misses
    /// headers.
    #[test]
    fn request_matches_query_param() {
        // Arrange
        let req1 = MockServerHttpRequest::new("GET".to_string(), "/test".to_string())
            .with_body("test".to_string());

        let req2 = RequestRequirements::new().with_body("test".to_string());

        // Act
        let does_match = request_matches(Rc::new(req1), &req2);

        // Assert
        assert_eq!(true, does_match);
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

        let req1 =
            MockServerHttpRequest::new("GET".to_string(), "/test".to_string()).with_headers(h1);
        let req2 = RequestRequirements::new().with_headers(h2);

        // Act
        let does_match = request_matches(Rc::new(req1), &req2);

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

        let req = MockServerHttpRequest::new("GET".to_string(), "/test".to_string())
            .with_headers(req_headers);

        let mock = RequestRequirements::new();

        // Act
        let does_match_1 = request_matches(Rc::new(req), &mock);

        // Assert
        assert_eq!(true, does_match_1); // effectively empty because mock does not expect any headers
    }

    /// This test makes sure no present headers on both sides, the mock and the request, are
    /// considered equal.
    #[test]
    fn request_matches_headers_match_empty() {
        // Arrange
        let req1 = MockServerHttpRequest::new("GET".to_string(), "/test".to_string());
        let req2 = RequestRequirements::new();

        // Act
        let does_match = request_matches(Rc::new(req1), &req2);

        // Assert
        assert_eq!(true, does_match);
    }

    /// This test ensures that mock request cannot contain a request method that cannot
    /// be sent along with a request body.
    #[test]
    fn validate_mock_definition_no_body_method() {
        // Arrange
        let req = RequestRequirements::new()
            .with_path("/test".to_string())
            .with_method("GET".to_string())
            .with_body("test".to_string());

        let res = MockServerHttpResponse::new(418 as u16);
        let smr = MockDefinition::new(req, res);

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
        let req = RequestRequirements::new();
        let res = MockServerHttpResponse::new(418 as u16);
        let smr = MockDefinition::new(req, res);

        // Act
        let result = validate_mock_definition(&smr);

        // Assert
        assert_eq!(true, result.is_ok());
    }

    /// This test ensures that mock validation is being invoked.
    #[test]
    fn add_new_mock_validation_error() {
        // Arrange
        let state = MockServerState::new();
        let mut req = RequestRequirements::new();
        req.method = Some("GET".into());
        req.body = Some("body".into());

        let res = MockServerHttpResponse::new(200);
        let mock_def = MockDefinition::new(req, res);

        // Act
        let result = add_new_mock(&state, mock_def);

        // Assert
        assert_eq!(result.is_err(), true);
        assert_eq!(result.err().unwrap().contains("Validation error"), true);
    }

    /// This test ensures that reading a non-existent mock does not result in an error but an
    /// empty result.
    #[test]
    fn read_one_returns_none_test() {
        // Arrange
        let state = MockServerState::new();

        // Act
        let result = read_one(&state, 6);

        // Assert
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap().is_none(), true);
    }

    /// This test checks if matching "path_contains" is working as expected.
    #[test]
    fn not_match_path_contains_test() {
        // Arrange
        let msr = Rc::new(MockServerHttpRequest::new("GET".into(), "test".into()));
        let mut mock1 = RequestRequirements::new();
        mock1.path_contains = Some(vec!["x".into()]);
        let mut mock2 = RequestRequirements::new();
        mock2.path_contains = Some(vec!["es".into()]);

        // Act
        let result1 = request_matches(msr.clone(), &mock1);
        let result2 = request_matches(msr.clone(), &mock2);

        // Assert
        assert_eq!(result1, false);
        assert_eq!(result2, true);
    }

    /// This test checks if matching "path_matches" is working as expected.
    #[test]
    fn not_match_path_matches_test() {
        // Arrange
        let msr = Rc::new(MockServerHttpRequest::new("GET".into(), "test".into()));
        let mut mock1 = RequestRequirements::new();
        mock1.path_matches = Some(vec![Pattern::from_regex(Regex::new(r#"x"#).unwrap())]);
        let mut mock2 = RequestRequirements::new();
        mock2.path_matches = Some(vec![Pattern::from_regex(Regex::new(r#"test"#).unwrap())]);

        // Act
        let result1 = request_matches(msr.clone(), &mock1);
        let result2 = request_matches(msr.clone(), &mock2);

        // Assert
        assert_eq!(result1, false);
        assert_eq!(result2, true);
    }
}
