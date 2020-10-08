use std::str::FromStr;

use assert_json_diff::{assert_json_eq_no_panic, assert_json_include_no_panic};
use serde_json::Value;

use crate::data::{
    ActiveMock, ClosestMatch, HttpMockRequest, MockDefinition, MockServerHttpResponse,
    RequestRequirements,
};

use crate::server::matchers::Matcher;
use crate::server::util::{StringTreeMapExtension, TreeMapExtension};
use crate::server::{Mismatch, MockServerState};
use basic_cookies::Cookie;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Contains HTTP methods which cannot have a body.
const NON_BODY_METHODS: &[&str] = &["GET", "HEAD", "DELETE"];

/// Adds a new mock to the internal state.
pub(crate) fn add_new_mock(
    state: &MockServerState,
    mock_def: MockDefinition,
) -> Result<usize, String> {
    let result = validate_mock_definition(&mock_def);

    if let Err(error_msg) = result {
        let error_msg = format!("Validation error: {}", error_msg);
        return Err(error_msg);
    }

    let mock_id = state.create_new_id();
    {
        log::debug!("Adding new mock with ID={}: {:#?}", mock_id, mock_def);
        let mut mocks = state.mocks.write().unwrap();
        mocks.insert(mock_id, ActiveMock::new(mock_id, mock_def));
    }

    Result::Ok(mock_id)
}

/// Reads exactly one mock object.
pub(crate) fn read_one_mock(
    state: &MockServerState,
    id: usize,
) -> Result<Option<ActiveMock>, String> {
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
pub(crate) fn delete_one_mock(state: &MockServerState, id: usize) -> Result<bool, String> {
    let result;
    {
        let mut mocks = state.mocks.write().unwrap();
        result = mocks.remove(&id);
    }

    log::debug!("Deleted mock with id={}", id);
    Result::Ok(result.is_some())
}

/// Deletes all mocks.
pub(crate) fn delete_all_mocks(state: &MockServerState) {
    let mut mocks = state.mocks.write().unwrap();
    mocks.clear();
    log::trace!("Deleted all mocks");
}

/// Deletes the request history.
pub(crate) fn delete_history(state: &MockServerState) {
    let mut mocks = state.history.write().unwrap();
    mocks.clear();
    log::trace!("Deleted request history");
}

/// Finds a mock that matches the current request and serve a response according to the mock
/// specification. If no mock is found, an empty result is being returned.
pub(crate) fn find_mock(
    state: &MockServerState,
    req: HttpMockRequest,
) -> Result<Option<MockServerHttpResponse>, String> {
    let req = Arc::new(req);

    {
        let mut history = state.history.write().unwrap();
        history.push(req.clone());
    }

    let found_mock_id: Option<usize>;
    {
        let mocks = state.mocks.read().unwrap();
        let result = mocks
            .values()
            .find(|&mock| request_matches(&state, req.clone(), &mock.definition.request));

        found_mock_id = match result {
            Some(mock) => Some(mock.id),
            None => None,
        };
    }

    if let Some(found_id) = found_mock_id {
        log::debug!(
            "Matched mock with id={} to the following request: {:#?}",
            found_id,
            req
        );
        let mut mocks = state.mocks.write().unwrap();
        let mock = mocks.get_mut(&found_id).unwrap();
        mock.call_counter += 1;
        return Ok(Some(mock.definition.response.clone()));
    }

    log::debug!(
        "Could not match any mock to the following request: {:#?}",
        req
    );
    Result::Ok(None)
}

/// Checks if a request matches a mock.
fn request_matches(
    state: &MockServerState,
    req: Arc<HttpMockRequest>,
    mock: &RequestRequirements,
) -> bool {
    log::trace!(
        "Matching incoming HTTP request {:?} against mock {:?}",
        req,
        mock
    );

    state
        .matchers
        .iter()
        .enumerate()
        .all(|(i, x)| x.matches(&req, mock))
}

/// Deletes the request history.
pub(crate) fn find_closest_match(
    state: &MockServerState,
    mock_id: usize,
) -> Result<Option<ClosestMatch>, String> {
    let mock = match read_one_mock(state, mock_id)? {
        None => return Ok(None),
        Some(v) => v,
    };

    let mut history = state.history.write().unwrap();

    let hit_counts = get_request_hit_counts(state, &mock, &history);
    let max_hits_requests = get_max_hits_requests(&hit_counts);
    let request_distances = get_distances(&max_hits_requests, &history, &state.matchers, &mock);
    let best_matches = get_min_distance_requests(&request_distances);

    let closes_match_request_idx = match best_matches.get(0) {
        None => return Ok(None),
        Some(idx) => *idx,
    };

    let req = history.get(closes_match_request_idx).unwrap();
    let mismatches = get_request_mismatches(req, &mock, &state.matchers);

    Ok(Some(ClosestMatch {
        request: HttpMockRequest::clone(&req),
        request_index: closes_match_request_idx,
        mismatches,
    }))
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

// For each request, find the number of matchers that successfully matched
fn get_request_hit_counts(
    state: &MockServerState,
    mock: &ActiveMock,
    history: &Vec<Arc<HttpMockRequest>>,
) -> BTreeMap<usize, usize> {
    history
        .iter()
        .enumerate()
        .map(|(idx, req)| {
            (
                idx,
                state
                    .matchers
                    .iter()
                    .filter(|mm| mm.matches(&req, &mock.definition.request))
                    .count(),
            )
        })
        .collect()
}

// Remember the maximum number of matchers that successfully matched
fn get_max_hits_requests(hit_counts: &BTreeMap<usize, usize>) -> BTreeMap<usize, usize> {
    // Find the element with the maximum matches
    let max_hits_req = hit_counts
        .iter()
        .max_by(|(idx1, num1), (idx2, num2)| num1.cmp(num2));

    let max_hits = match max_hits_req {
        None => return BTreeMap::new(),
        Some((_, n)) => *n,
    };

    hit_counts
        .into_iter()
        .filter(|(a, b)| **b == max_hits)
        .map(|(a, b)| (*a, *b))
        .collect()
}

// Remember the maximum number of matchers that successfully matched
fn get_distances(
    requests: &BTreeMap<usize, usize>,
    history: &Vec<Arc<HttpMockRequest>>,
    matchers: &Vec<Box<dyn Matcher + Sync + Send>>,
    mock: &ActiveMock,
) -> BTreeMap<usize, usize> {
    history
        .iter()
        .enumerate()
        .filter(|(idx, r)| requests.contains_key(idx))
        .map(|(idx, req)| (idx, get_request_distance(req, mock, matchers)))
        .collect()
}

fn get_request_mismatches(
    req: &Arc<HttpMockRequest>,
    mock: &ActiveMock,
    matchers: &Vec<Box<dyn Matcher + Sync + Send>>,
) -> Vec<Mismatch> {
    matchers
        .iter()
        .map(|mat| mat.mismatches(req, &mock.definition.request))
        .flatten()
        .into_iter()
        .collect()
}

fn get_request_distance(
    req: &Arc<HttpMockRequest>,
    mock: &ActiveMock,
    matchers: &Vec<Box<dyn Matcher + Sync + Send>>,
) -> usize {
    matchers
        .iter()
        .map(|matcher| matcher.distance(req, &mock.definition.request))
        .sum()
}

// Remember the maximum number of matchers that successfully matched
fn get_min_distance_requests(request_distances: &BTreeMap<usize, usize>) -> Vec<usize> {
    // Find the element with the maximum matches
    let min_elem = request_distances
        .iter()
        .min_by(|(idx1, d1), (idx2, d2)| (**d1).cmp(d2));

    let max = match min_elem {
        None => return Vec::new(),
        Some((_, n)) => *n,
    };

    request_distances
        .into_iter()
        .filter(|(idx, distance)| **distance == max)
        .map(|(idx, _)| *idx)
        .collect()
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;
    use std::rc::Rc;

    use crate::data::{
        HttpMockRequest, MockDefinition, MockServerHttpResponse, Pattern, RequestRequirements,
    };
    use crate::server::web::handlers::{
        add_new_mock, read_one_mock, request_matches, validate_mock_definition,
    };
    use crate::server::MockServerState;
    use regex::Regex;
    use std::sync::Arc;

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
        let request1 = HttpMockRequest::new("GET".to_string(), "/test-path".to_string())
            .with_body("test".to_string());
        let request2 = HttpMockRequest::new("GET".to_string(), "/test-path".to_string())
            .with_body("test".to_string());

        let requirements1 = RequestRequirements::new().with_body_contains(vec!["xxx".to_string()]);
        let requirements2 = RequestRequirements::new().with_body_contains(vec!["es".to_string()]);

        // Act
        let does_match1 =
            request_matches(&MockServerState::new(), Arc::new(request1), &requirements1);
        let does_match2 =
            request_matches(&MockServerState::new(), Arc::new(request2), &requirements2);

        // Assert
        assert_eq!(false, does_match1);
        assert_eq!(true, does_match2);
    }

    #[test]
    fn body_matches_query_params_exact_test() {
        // Arrange
        let mut params1 = Vec::new();
        params1.push(("k".to_string(), "v".to_string()));

        let mut params2 = Vec::new();
        params2.push(("h".to_string(), "o".to_string()));

        let request1 = HttpMockRequest::new("GET".to_string(), "/test-path".to_string())
            .with_query_params(params1.clone());
        let request2 = HttpMockRequest::new("GET".to_string(), "/test-path".to_string())
            .with_query_params(params1.clone());

        let requirements1 = RequestRequirements::new().with_query_param(params2);
        let requirements2 = RequestRequirements::new().with_query_param(params1.clone());

        // Act
        let does_match1 =
            request_matches(&MockServerState::new(), Arc::new(request1), &requirements1);
        let does_match2 =
            request_matches(&MockServerState::new(), Arc::new(request2), &requirements2);

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
        let req1 = HttpMockRequest::new("GET".to_string(), "/test-path".to_string());

        let req2 = RequestRequirements::new().with_path("/test-path".to_string());

        // Act
        let does_match = request_matches(&MockServerState::new(), Arc::new(req1), &req2);

        // Assert
        assert_eq!(true, does_match);
    }

    /// This test makes sure that a request is considered "not matched" if the paths of the
    /// request and the mock are not equal.
    #[test]
    fn request_matches_path_no_match() {
        // Arrange
        let req1 = HttpMockRequest::new("GET".to_string(), "/test-path".to_string());

        let req2 = RequestRequirements::new().with_path("/another-path".to_string());

        // Act
        let does_match = request_matches(&MockServerState::new(), Arc::new(req1), &req2);

        // Assert
        assert_eq!(false, does_match);
    }

    /// This test makes sure that a request is considered "matched" if the methods of the
    /// request and the mock are equal.
    #[test]
    fn request_matches_method_match() {
        // Arrange
        let req1 = HttpMockRequest::new("GET".to_string(), "/test".to_string());

        let req2 = RequestRequirements::new().with_method("GET".to_string());

        // Act
        let does_match = request_matches(&MockServerState::new(), Arc::new(req1), &req2);

        // Assert
        assert_eq!(true, does_match);
    }

    /// This test makes sure that a request is considered "not matched" if the methods of the
    /// request and the mock are not equal.
    #[test]
    fn request_matches_method_no_match() {
        // Arrange
        let req1 = HttpMockRequest::new("GET".to_string(), "/test".to_string());

        let req2 = RequestRequirements::new().with_method("POST".to_string());

        // Act
        let does_match = request_matches(&MockServerState::new(), Arc::new(req1), &req2);

        // Assert
        assert_eq!(false, does_match);
    }

    /// This test makes sure that a request is considered "matched" if the bodies of both,
    /// the request and the mock are present and have equal content.
    #[test]
    fn request_matches_body_match() {
        // Arrange
        let req1 = HttpMockRequest::new("GET".to_string(), "/test".to_string())
            .with_body("test".to_string());

        let req2 = RequestRequirements::new().with_body("test".to_string());

        // Act
        let does_match = request_matches(&MockServerState::new(), Arc::new(req1), &req2);

        // Assert
        assert_eq!(true, does_match);
    }

    /// This test makes sure that a request is considered "not matched" if the bodies of both,
    /// the request and the mock are present, but do have different content.
    #[test]
    fn request_matches_body_no_match() {
        // Arrange
        let req1 = HttpMockRequest::new("GET".to_string(), "/test".to_string())
            .with_body("some text".to_string());

        let req2 = RequestRequirements::new().with_body("some other text".to_string());

        // Act
        let does_match = request_matches(&MockServerState::new(), Arc::new(req1), &req2);

        // Assert
        assert_eq!(false, does_match);
    }

    /// This test makes sure that a request is considered "matched" when the request contains
    /// exactly the same as the mock expects.
    #[test]
    fn request_matches_headers_exact_match() {
        // Arrange
        let mut h1 = Vec::new();
        h1.push(("h1".to_string(), "v1".to_string()));
        h1.push(("h2".to_string(), "v2".to_string()));

        let mut h2 = Vec::new();
        h2.push(("h1".to_string(), "v1".to_string()));
        h2.push(("h2".to_string(), "v2".to_string()));

        let req1 = HttpMockRequest::new("GET".to_string(), "/test".to_string()).with_headers(h1);

        let req2 = RequestRequirements::new().with_headers(h2);

        // Act
        let does_match = request_matches(&MockServerState::new(), Arc::new(req1), &req2);

        // Assert
        assert_eq!(true, does_match);
    }

    /// This test makes sure that a request is considered "not matched" when the request misses
    /// headers.
    #[test]
    fn request_matches_query_param() {
        // Arrange
        let req1 = HttpMockRequest::new("GET".to_string(), "/test".to_string())
            .with_body("test".to_string());

        let req2 = RequestRequirements::new().with_body("test".to_string());

        // Act
        let does_match = request_matches(&MockServerState::new(), Arc::new(req1), &req2);

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
        let mut h1 = Vec::new();
        h1.push(("h1".to_string(), "v1".to_string()));
        h1.push(("h2".to_string(), "v2".to_string()));

        let mut h2 = Vec::new();
        h2.push(("h1".to_string(), "v1".to_string()));

        let req1 = HttpMockRequest::new("GET".to_string(), "/test".to_string()).with_headers(h1);
        let req2 = RequestRequirements::new().with_headers(h2);

        // Act
        let does_match = request_matches(&MockServerState::new(), Arc::new(req1), &req2);

        // Assert
        assert_eq!(true, does_match); // matches, because request contains more headers than the mock expects
    }

    /// This test makes sure that even the headers of a mock and a request differ,
    /// the request still is considered "matched" when the mock does not expect any headers
    /// at all. Hence a request is allowed to contain headers that a mock does not.
    #[test]
    fn request_matches_headers_no_match_empty() {
        // Arrange
        let mut req_headers = Vec::new();
        req_headers.push(("req_headers".to_string(), "v1".to_string()));
        req_headers.push(("h2".to_string(), "v2".to_string()));

        let req =
            HttpMockRequest::new("GET".to_string(), "/test".to_string()).with_headers(req_headers);

        let mock = RequestRequirements::new();

        // Act
        let does_match_1 = request_matches(&MockServerState::new(), Arc::new(req), &mock);

        // Assert
        assert_eq!(true, does_match_1); // effectively empty because mock does not expect any headers
    }

    /// This test makes sure no present headers on both sides, the mock and the request, are
    /// considered equal.
    #[test]
    fn request_matches_headers_match_empty() {
        // Arrange
        let req1 = HttpMockRequest::new("GET".to_string(), "/test".to_string());
        let req2 = RequestRequirements::new();

        // Act
        let does_match = request_matches(&MockServerState::new(), Arc::new(req1), &req2);

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

        let res = MockServerHttpResponse {
            body: None,
            delay: None,
            status: Some(418),
            headers: None,
        };

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
        let res = MockServerHttpResponse {
            body: None,
            delay: None,
            status: Some(418),
            headers: None,
        };

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

        let res = MockServerHttpResponse {
            body: None,
            delay: None,
            status: Some(200),
            headers: None,
        };

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
        let result = read_one_mock(&state, 6);

        // Assert
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap().is_none(), true);
    }

    /// This test checks if matching "path_contains" is working as expected.
    #[test]
    fn not_match_path_contains_test() {
        // Arrange
        let msr = Arc::new(HttpMockRequest::new("GET".into(), "test".into()));
        let mut mock1 = RequestRequirements::new();
        mock1.path_contains = Some(vec!["x".into()]);
        let mut mock2 = RequestRequirements::new();
        mock2.path_contains = Some(vec!["es".into()]);

        // Act
        let result1 = request_matches(&MockServerState::new(), msr.clone(), &mock1);
        let result2 = request_matches(&MockServerState::new(), msr.clone(), &mock2);

        // Assert
        assert_eq!(result1, false);
        assert_eq!(result2, true);
    }

    /// This test checks if matching "path_matches" is working as expected.
    #[test]
    fn not_match_path_matches_test() {
        // Arrange
        let msr = Arc::new(HttpMockRequest::new("GET".into(), "test".into()));
        let mut mock1 = RequestRequirements::new();
        mock1.path_matches = Some(vec![Pattern::from_regex(Regex::new(r#"x"#).unwrap())]);
        let mut mock2 = RequestRequirements::new();
        mock2.path_matches = Some(vec![Pattern::from_regex(Regex::new(r#"test"#).unwrap())]);

        // Act
        let result1 = request_matches(&MockServerState::new(), msr.clone(), &mock1);
        let result2 = request_matches(&MockServerState::new(), msr.clone(), &mock2);

        // Assert
        assert_eq!(result1, false);
        assert_eq!(result2, true);
    }
}
