use crate::data::{HttpMockRequest, RequestRequirements};
use crate::server::matchers::sources::{MultiValueValueSource, ValueSource};
use crate::server::matchers::util::parse_cookies;
use std::collections::BTreeMap;

pub(crate) struct BodySource {}

impl BodySource {
    pub fn new() -> Self {
        Self {}
    }
}

impl ValueSource for BodySource {
    fn parse_from_mock<'a>(&self, mock: &'a RequestRequirements) -> Option<&'a String> {
        mock.body.as_ref()
    }

    fn parse_from_request<'a>(&self, req: &'a HttpMockRequest) -> Option<&'a String> {
        req.body.as_ref()
    }
}
