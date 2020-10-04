use crate::data::Pattern;
use crate::server::matchers::{distance, distance_for_opt};
use crate::Regex;
use assert_json_diff::{assert_json_eq_no_panic, assert_json_include_no_panic};
use serde_json::Value;

// TODO: Implement memoization for Comparators
pub trait ValueComparator<S, T> {
    fn matches(&self, mock_value: &S, req_value: &T) -> bool;
    fn operation_name(&self) -> &str;
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
        assert_json_eq_no_panic(mock_value, req_value).is_ok()
    }

    fn operation_name(&self) -> &str {
        "equals"
    }

    fn distance(&self, mock_value: &Option<&Value>, req_value: &Option<&Value>) -> usize {
        distance_for_opt(mock_value, req_value)
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
        assert_json_include_no_panic(mock_value, req_value).is_ok()
    }

    fn operation_name(&self) -> &str {
        "equals"
    }

    fn distance(&self, mock_value: &Option<&Value>, req_value: &Option<&Value>) -> usize {
        distance_for_opt(mock_value, req_value)
    }
}

// ************************************************************************************************
// StringExactMatchComparator
// ************************************************************************************************
pub struct StringExactMatchComparator {}

impl StringExactMatchComparator {
    pub fn new() -> Self {
        Self {}
    }
}

impl ValueComparator<String, String> for StringExactMatchComparator {
    fn matches(&self, mock_value: &String, req_value: &String) -> bool {
        mock_value.eq(req_value)
    }
    fn operation_name(&self) -> &str {
        "equals"
    }
    fn distance(&self, mock_value: &Option<&String>, req_value: &Option<&String>) -> usize {
        distance_for_opt(mock_value, req_value)
    }
}

// ************************************************************************************************
// StringExactMatchComparator
// ************************************************************************************************
pub struct StringContainsMatchComparator {}

impl StringContainsMatchComparator {
    pub fn new() -> Self {
        Self {}
    }
}

impl ValueComparator<String, String> for StringContainsMatchComparator {
    fn matches(&self, mock_value: &String, req_value: &String) -> bool {
        mock_value.contains(req_value)
    }
    fn operation_name(&self) -> &str {
        "contains"
    }
    fn distance(&self, mock_value: &Option<&String>, req_value: &Option<&String>) -> usize {
        distance_for_opt(mock_value, req_value)
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

    fn operation_name(&self) -> &str {
        "matches"
    }

    fn distance(&self, mock_value: &Option<&Regex>, req_value: &Option<&String>) -> usize {
        distance_for_opt(mock_value, req_value)
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
    fn operation_name(&self) -> &str {
        "any"
    }
    fn distance(&self, _: &Option<&T>, _: &Option<&U>) -> usize {
        0
    }
}
