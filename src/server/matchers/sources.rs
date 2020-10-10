use std::collections::BTreeMap;

use serde_json::Value;

use crate::data::{HttpMockRequest, Pattern, RequestRequirements};
use crate::Regex;

pub(crate) trait ValueSource<T> {
    fn parse_from_mock<'a>(&self, mock: &'a RequestRequirements) -> Option<Vec<&'a T>>;
}

pub(crate) trait MultiValueSource<T, U> {
    fn parse_from_mock<'a>(
        &self,
        mock: &'a RequestRequirements,
    ) -> Option<Vec<(&'a T, Option<&'a U>)>>;
}

// ************************************************************************************************
// StringBodySource
// ************************************************************************************************
pub(crate) struct StringBodySource {}

impl StringBodySource {
    pub fn new() -> Self {
        Self {}
    }
}

impl ValueSource<String> for StringBodySource {
    fn parse_from_mock<'a>(&self, mock: &'a RequestRequirements) -> Option<Vec<&'a String>> {
        mock.body.as_ref().map(|b| vec![b])
    }
}

// ************************************************************************************************
// StringBodySource
// ************************************************************************************************
pub(crate) struct StringBodyContainsSource {}

impl StringBodyContainsSource {
    pub fn new() -> Self {
        Self {}
    }
}

impl ValueSource<String> for StringBodyContainsSource {
    fn parse_from_mock<'a>(&self, mock: &'a RequestRequirements) -> Option<Vec<&'a String>> {
        mock.body_contains
            .as_ref()
            .map(|v| v.into_iter().map(|bc| bc).collect())
    }
}

// ************************************************************************************************
// BodyRegexSource
// ************************************************************************************************
pub(crate) struct JSONBodySource {}

impl JSONBodySource {
    pub fn new() -> Self {
        Self {}
    }
}

impl ValueSource<Value> for JSONBodySource {
    fn parse_from_mock<'a>(&self, mock: &'a RequestRequirements) -> Option<Vec<&'a Value>> {
        mock.json_body.as_ref().map(|b| vec![b])
    }
}

// ************************************************************************************************
// PartialJSONBodySource
// ************************************************************************************************
pub(crate) struct PartialJSONBodySource {}

impl PartialJSONBodySource {
    pub fn new() -> Self {
        Self {}
    }
}

impl ValueSource<Value> for PartialJSONBodySource {
    fn parse_from_mock<'a>(&self, mock: &'a RequestRequirements) -> Option<Vec<&'a Value>> {
        mock.json_body_includes
            .as_ref()
            .map(|b| b.into_iter().collect())
    }
}

// ************************************************************************************************
// BodyRegexSource
// ************************************************************************************************
pub(crate) struct BodyRegexSource {}

impl BodyRegexSource {
    pub fn new() -> Self {
        Self {}
    }
}

impl ValueSource<Regex> for BodyRegexSource {
    fn parse_from_mock<'a>(&self, mock: &'a RequestRequirements) -> Option<Vec<&'a Regex>> {
        mock.body_matches
            .as_ref()
            .map(|b| b.iter().map(|p| &p.regex).collect())
    }
}

// ************************************************************************************************
// MethodSource
// ************************************************************************************************
pub(crate) struct MethodSource {}

impl MethodSource {
    pub fn new() -> Self {
        Self {}
    }
}

impl ValueSource<String> for MethodSource {
    fn parse_from_mock<'a>(&self, mock: &'a RequestRequirements) -> Option<Vec<&'a String>> {
        mock.method.as_ref().map(|b| vec![b])
    }
}

// ************************************************************************************************
// StringPathSource
// ************************************************************************************************
pub(crate) struct StringPathSource {}

impl StringPathSource {
    pub fn new() -> Self {
        Self {}
    }
}

impl ValueSource<String> for StringPathSource {
    fn parse_from_mock<'a>(&self, mock: &'a RequestRequirements) -> Option<Vec<&'a String>> {
        mock.path.as_ref().map(|b| vec![b])
    }
}

// ************************************************************************************************
// StringPathContainsSource
// ************************************************************************************************
pub(crate) struct PathContainsSubstringSource {}

impl PathContainsSubstringSource {
    pub fn new() -> Self {
        Self {}
    }
}

impl ValueSource<String> for PathContainsSubstringSource {
    fn parse_from_mock<'a>(&self, mock: &'a RequestRequirements) -> Option<Vec<&'a String>> {
        mock.path_contains
            .as_ref()
            .map(|b| b.into_iter().map(|v| v).collect())
    }
}

// ************************************************************************************************
// PathRegexSource
// ************************************************************************************************
pub(crate) struct PathRegexSource {}

impl PathRegexSource {
    pub fn new() -> Self {
        Self {}
    }
}

impl ValueSource<Regex> for PathRegexSource {
    fn parse_from_mock<'a>(&self, mock: &'a RequestRequirements) -> Option<Vec<&'a Regex>> {
        mock.path_matches
            .as_ref()
            .map(|b| b.into_iter().map(|v| &v.regex).collect())
    }
}

// ************************************************************************************************
// CookieSource
// ************************************************************************************************
pub(crate) struct CookieSource {}

impl CookieSource {
    pub fn new() -> Self {
        Self {}
    }
}

impl MultiValueSource<String, String> for CookieSource {
    fn parse_from_mock<'a>(
        &self,
        mock: &'a RequestRequirements,
    ) -> Option<Vec<(&'a String, Option<&'a String>)>> {
        mock.cookies
            .as_ref()
            .map(|c| c.iter().map(|(k, v)| (k, Some(v))).collect())
    }
}

// ************************************************************************************************
// ContainsCookieSource
// ************************************************************************************************
pub(crate) struct ContainsCookieSource {}

impl ContainsCookieSource {
    pub fn new() -> Self {
        Self {}
    }
}

impl MultiValueSource<String, String> for ContainsCookieSource {
    fn parse_from_mock<'a>(
        &self,
        mock: &'a RequestRequirements,
    ) -> Option<Vec<(&'a String, Option<&'a String>)>> {
        mock.cookie_exists
            .as_ref()
            .map(|c| c.iter().map(|v| (v, None)).collect())
    }
}

// ************************************************************************************************
// HeaderSource
// ************************************************************************************************
pub(crate) struct HeaderSource {}

impl HeaderSource {
    pub fn new() -> Self {
        Self {}
    }
}

impl MultiValueSource<String, String> for HeaderSource {
    fn parse_from_mock<'a>(
        &self,
        mock: &'a RequestRequirements,
    ) -> Option<Vec<(&'a String, Option<&'a String>)>> {
        mock.headers
            .as_ref()
            .map(|c| c.iter().map(|(k, v)| (k, Some(v))).collect())
    }
}

// ************************************************************************************************
// ContainsCookieSource
// ************************************************************************************************
pub(crate) struct ContainsHeaderSource {}

impl ContainsHeaderSource {
    pub fn new() -> Self {
        Self {}
    }
}

impl MultiValueSource<String, String> for ContainsHeaderSource {
    fn parse_from_mock<'a>(
        &self,
        mock: &'a RequestRequirements,
    ) -> Option<Vec<(&'a String, Option<&'a String>)>> {
        mock.header_exists
            .as_ref()
            .map(|c| c.iter().map(|v| (v, None)).collect())
    }
}

// ************************************************************************************************
// QueryParameterSource
// ************************************************************************************************
pub(crate) struct QueryParameterSource {}

impl QueryParameterSource {
    pub fn new() -> Self {
        Self {}
    }
}

impl MultiValueSource<String, String> for QueryParameterSource {
    fn parse_from_mock<'a>(
        &self,
        mock: &'a RequestRequirements,
    ) -> Option<Vec<(&'a String, Option<&'a String>)>> {
        mock.query_param
            .as_ref()
            .map(|v| v.into_iter().map(|(k, v)| (k, Some(v))).collect())
    }
}

// ************************************************************************************************
// ContainsQueryParameterSource
// ************************************************************************************************
pub(crate) struct ContainsQueryParameterSource {}

impl ContainsQueryParameterSource {
    pub fn new() -> Self {
        Self {}
    }
}

impl MultiValueSource<String, String> for ContainsQueryParameterSource {
    fn parse_from_mock<'a>(
        &self,
        mock: &'a RequestRequirements,
    ) -> Option<Vec<(&'a String, Option<&'a String>)>> {
        mock.query_param_exists
            .as_ref()
            .map(|v| v.into_iter().map(|v| (v, None)).collect())
    }
}
