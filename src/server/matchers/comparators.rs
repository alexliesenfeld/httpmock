use crate::data::Pattern;
use crate::Regex;
use assert_json_diff::{assert_json_eq_no_panic, assert_json_include_no_panic};
use serde_json::Value;

pub trait ValueComparator<S, T> {
    fn matches(&self, mock_value: &S, req_value: &T) -> bool;
    fn operation_name(&self) -> &str;
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
}
