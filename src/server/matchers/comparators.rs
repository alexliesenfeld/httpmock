use assert_json_diff::{assert_json_matches_no_panic, CompareMode, Config};
use serde_json::Value;

use crate::data::{HttpMockRequest, MockMatcherFunction, Pattern};
use crate::server::matchers::distance_for;
use crate::Regex;

pub trait ValueComparator<S, T> {
    fn matches(&self, mock_value: &S, req_value: &T) -> bool;
    fn name(&self) -> &str;
    fn distance(&self, mock_value: &Option<&S>, req_value: &Option<&T>) -> usize;
}

// ************************************************************************************************
// JSONExactMatchComparator
// ************************************************************************************************
pub struct JSONExactMatchComparator {}

impl JSONExactMatchComparator {
    pub fn new() -> Self {
        Self {}
    }
}

impl ValueComparator<Value, Value> for JSONExactMatchComparator {
    fn matches(&self, mock_value: &Value, req_value: &Value) -> bool {
        let config = Config::new(CompareMode::Strict);
        assert_json_matches_no_panic(req_value, mock_value, config).is_ok()
    }

    fn name(&self) -> &str {
        "equals"
    }

    fn distance(&self, mock_value: &Option<&Value>, req_value: &Option<&Value>) -> usize {
        distance_for(mock_value, req_value)
    }
}

// ************************************************************************************************
// JSONExactMatchComparator
// ************************************************************************************************
pub struct JSONContainsMatchComparator {}

impl JSONContainsMatchComparator {
    pub fn new() -> Self {
        Self {}
    }
}

impl ValueComparator<Value, Value> for JSONContainsMatchComparator {
    fn matches(&self, mock_value: &Value, req_value: &Value) -> bool {
        let config = Config::new(CompareMode::Inclusive);
        assert_json_matches_no_panic(req_value, mock_value, config).is_ok()
    }

    fn name(&self) -> &str {
        "contains"
    }

    fn distance(&self, mock_value: &Option<&Value>, req_value: &Option<&Value>) -> usize {
        distance_for(mock_value, req_value)
    }
}

// ************************************************************************************************
// StringExactMatchComparator
// ************************************************************************************************
pub struct StringExactMatchComparator {
    case_sensitive: bool,
}

impl StringExactMatchComparator {
    pub fn new(case_sensitive: bool) -> Self {
        Self { case_sensitive }
    }
}

impl ValueComparator<String, String> for StringExactMatchComparator {
    fn matches(&self, mock_value: &String, req_value: &String) -> bool {
        match self.case_sensitive {
            true => mock_value.eq(req_value),
            false => mock_value.to_lowercase().eq(&req_value.to_lowercase()),
        }
    }
    fn name(&self) -> &str {
        "equals"
    }
    fn distance(&self, mock_value: &Option<&String>, req_value: &Option<&String>) -> usize {
        distance_for(mock_value, req_value)
    }
}

// ************************************************************************************************
// StringExactMatchComparator
// ************************************************************************************************
pub struct StringContainsMatchComparator {
    case_sensitive: bool,
}

impl StringContainsMatchComparator {
    pub fn new(case_sensitive: bool) -> Self {
        Self { case_sensitive }
    }
}

impl ValueComparator<String, String> for StringContainsMatchComparator {
    fn matches(&self, mock_value: &String, req_value: &String) -> bool {
        match self.case_sensitive {
            true => req_value.contains(mock_value),
            false => req_value
                .to_lowercase()
                .contains(&mock_value.to_lowercase()),
        }
    }
    fn name(&self) -> &str {
        "contains"
    }
    fn distance(&self, mock_value: &Option<&String>, req_value: &Option<&String>) -> usize {
        distance_for(mock_value, req_value)
    }
}

// ************************************************************************************************
// StringExactMatchComparator
// ************************************************************************************************
pub struct StringRegexMatchComparator {}

impl StringRegexMatchComparator {
    pub fn new() -> Self {
        Self {}
    }
}

impl ValueComparator<Regex, String> for StringRegexMatchComparator {
    fn matches(&self, mock_value: &Regex, req_value: &String) -> bool {
        mock_value.is_match(req_value)
    }

    fn name(&self) -> &str {
        "matches regex"
    }

    fn distance(&self, mock_value: &Option<&Regex>, req_value: &Option<&String>) -> usize {
        distance_for(mock_value, req_value)
    }
}

// ************************************************************************************************
// AnyValueComparator
// ************************************************************************************************
pub struct AnyValueComparator {}

impl AnyValueComparator {
    pub fn new() -> Self {
        Self {}
    }
}

impl<T, U> ValueComparator<T, U> for AnyValueComparator {
    fn matches(&self, _: &T, _: &U) -> bool {
        true
    }
    fn name(&self) -> &str {
        "any"
    }
    fn distance(&self, _: &Option<&T>, _: &Option<&U>) -> usize {
        0
    }
}

// ************************************************************************************************
// FunctionMatchComparator
// ************************************************************************************************
pub struct FunctionMatchesRequestComparator {}

impl FunctionMatchesRequestComparator {
    pub fn new() -> Self {
        Self {}
    }
}

impl ValueComparator<MockMatcherFunction, HttpMockRequest> for FunctionMatchesRequestComparator {
    fn matches(&self, mock_value: &MockMatcherFunction, req_value: &HttpMockRequest) -> bool {
        (*mock_value)(req_value)
    }

    fn name(&self) -> &str {
        "matches"
    }

    fn distance(
        &self,
        mock_value: &Option<&MockMatcherFunction>,
        req_value: &Option<&HttpMockRequest>,
    ) -> usize {
        let mock_value = match mock_value {
            None => return 0,
            Some(v) => v,
        };
        let req_value = match req_value {
            None => return 1,
            Some(v) => v,
        };
        match self.matches(mock_value, req_value) {
            true => 0,
            false => 1,
        }
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use crate::server::matchers::comparators::{
        AnyValueComparator, JSONContainsMatchComparator, JSONExactMatchComparator,
        StringContainsMatchComparator, StringExactMatchComparator, StringRegexMatchComparator,
        ValueComparator,
    };
    use crate::Regex;

    fn run_test<S, T>(
        comparator: &dyn ValueComparator<S, T>,
        v1: &S,
        v2: &T,
        expected_match: bool,
        expected_distance: usize,
        expected_name: &str,
    ) {
        // Act
        let match_result = comparator.matches(&v1, &v2);
        let distance_result = comparator.distance(&Some(&v1), &Some(&v2));
        let name_result = comparator.name();

        // Assert
        assert_eq!(match_result, expected_match);
        assert_eq!(distance_result, expected_distance);
        assert_eq!(name_result, expected_name);
    }

    #[test]
    fn json_exact_match_comparator_match() {
        run_test(
            &JSONExactMatchComparator::new(),
            &json!({"name" : "Peter", "surname" : "Griffin"}),
            &json!({"name" : "Peter", "surname" : "Griffin"}),
            true,
            0,
            "equals",
        );
    }

    #[test]
    fn json_exact_match_comparator_no_match() {
        run_test(
            &JSONExactMatchComparator::new(),
            &json!({"name" : "Peter", "surname" : "Griffin"}),
            &json!({"name" : "Walter", "surname" : "White"}),
            false,
            9,
            "equals",
        );
    }

    #[test]
    fn json_contains_comparator_match() {
        run_test(
            &JSONContainsMatchComparator::new(),
            &json!({ "other" : { "human" : { "surname" : "Griffin" }}}),
            &json!({ "name" : "Peter", "other" : { "human" : { "surname" : "Griffin" }}}),
            true,
            15, // compute distance even if values match!
            "contains",
        );
    }

    #[test]
    fn json_contains_comparator_no_match() {
        run_test(
            &JSONContainsMatchComparator::new(),
            &json!({ "surname" : "Griffin" }),
            &json!({ "name" : "Peter", "other" : { "human" : { "surname" : "Griffin" }}}),
            false,
            35, // compute distance even if values match!
            "contains",
        );
    }

    #[test]
    fn string_exact_comparator_match() {
        run_test(
            &StringExactMatchComparator::new(true),
            &"test string".to_string(),
            &"test string".to_string(),
            true,
            0, // compute distance even if values match!
            "equals",
        );
    }

    #[test]
    fn string_exact_comparator_no_match() {
        run_test(
            &StringExactMatchComparator::new(true),
            &"test string".to_string(),
            &"not a test string".to_string(),
            false,
            6, // compute distance even if values match!
            "equals",
        );
    }

    #[test]
    fn string_exact_comparator_case_sensitive_match() {
        run_test(
            &StringExactMatchComparator::new(false),
            &"TEST string".to_string(),
            &"test STRING".to_string(),
            true,
            10, // compute distance even if values match!
            "equals",
        );
    }

    #[test]
    fn string_contains_comparator_match() {
        run_test(
            &StringContainsMatchComparator::new(true),
            &"st st".to_string(),
            &"test string".to_string(),
            true,
            6, // compute distance even if values match!
            "contains",
        );
    }

    #[test]
    fn string_contains_comparator_no_match() {
        run_test(
            &StringContainsMatchComparator::new(true),
            &"xxx".to_string(),
            &"yyy".to_string(),
            false,
            3, // compute distance even if values match!
            "contains",
        );
    }

    #[test]
    fn string_contains_comparator_case_sensitive_match() {
        run_test(
            &StringContainsMatchComparator::new(false),
            &"ST st".to_string(),
            &"test STRING".to_string(),
            true,
            9, // compute distance even if values match!
            "contains",
        );
    }

    #[test]
    fn regex_comparator_match() {
        run_test(
            &StringRegexMatchComparator::new(),
            &Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap(),
            &"2014-01-01".to_string(),
            true,
            16, // compute distance even if values match!
            "matches regex",
        );
    }

    #[test]
    fn regex_comparator_no_match() {
        run_test(
            &StringRegexMatchComparator::new(),
            &Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap(),
            &"xxx".to_string(),
            false,
            19, // compute distance even if values match!
            "matches regex",
        );
    }

    #[test]
    fn any_comparator_match() {
        run_test(
            &AnyValueComparator::new(),
            &"00000000".to_string(),
            &"xxx".to_string(),
            true,
            0, // compute distance even if values match!
            "any",
        );
    }
}
