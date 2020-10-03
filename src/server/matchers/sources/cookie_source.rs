use crate::data::{HttpMockRequest, RequestRequirements};
use crate::server::matchers::sources::MultiValueValueSource;
use crate::server::matchers::util::parse_cookies;
use std::collections::BTreeMap;

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

    fn parse_from_request(&self, req: &HttpMockRequest) -> BTreeMap<String, String> {
        let req_cookies = match parse_cookies(req) {
            Ok(v) => v,
            Err(err) => {
                log::info!(
                "Cannot parse cookies. Cookie matching will not work for this request. Error: {}",
                err
            );
                return BTreeMap::new();
            }
        };

        req_cookies
            .into_iter()
            .map(|(k, v)| (k.to_lowercase(), v))
            .collect()
    }
}
