pub(crate) mod body_source;
pub(crate) mod cookie_source;

use crate::data::{HttpMockRequest, RequestRequirements};
use std::collections::BTreeMap;

pub(crate) trait MultiValueValueSource {
    fn parse_from_mock<'a>(
        &self,
        mock: &'a RequestRequirements,
    ) -> Option<&'a BTreeMap<String, String>>;
    fn parse_from_request(&self, req: &HttpMockRequest) -> BTreeMap<String, String>;
}

pub(crate) trait ValueSource {
    fn parse_from_mock<'a>(&self, mock: &'a RequestRequirements) -> Option<&'a String>;
    fn parse_from_request<'a>(&self, req: &'a HttpMockRequest) -> Option<&'a String>;
}

pub(crate) trait JSONValueSource {
    fn parse_from_mock<'a>(&self, mock: &'a RequestRequirements) -> Option<&'a serde_json::Value>;
    fn parse_from_request(&self, req: &HttpMockRequest) -> Option<serde_json::Value>;
}
