use crate::data::{HttpMockRequest, RequestRequirements};
use crate::server::matchers::util::parse_cookies;
use std::collections::BTreeMap;
use serde_json::Value;
use crate::Regex;

pub(crate) trait MultiValueValueSource {
    fn parse_from_mock<'a>(
        &self,
        mock: &'a RequestRequirements,
    ) -> Option<&'a BTreeMap<String, String>>;
}

pub(crate) trait ValueSource<T> {
    fn parse_from_mock<'a>(&self, mock: &'a RequestRequirements) -> Option<Vec<&'a T>>;
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
        mock.json_body_includes.as_ref().map(|b| b.into_iter().collect())
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
// CookieSource
// ************************************************************************************************
pub(crate) struct CookieSource {}

impl CookieSource {
    pub fn new() -> Self {
        Self {}
    }
}
impl MultiValueValueSource for CookieSource {
    fn parse_from_mock<'a>(
        &self,
        mock: &'a RequestRequirements,
    ) -> Option<&'a BTreeMap<String, String>> {
        mock.cookies.as_ref()
    }
}
