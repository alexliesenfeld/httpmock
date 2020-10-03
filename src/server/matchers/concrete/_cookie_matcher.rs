#[macro_use]
use crate::data::{HttpMockRequest, RequestRequirements};
use crate::server::matchers::generic::multi_value_exact::MultiValueExactMatcher;
use crate::server::matchers::sources::cookie_source::CookieSource;
use crate::server::matchers::util::{diff_str_new, distance_for, parse_cookies};
use crate::server::matchers::{
    diff_str, DetailedDiffResult, Matcher, Mismatch, SimpleDiffResult, Tokenizer,
};
use crate::server::util::StringTreeMapExtension;
use basic_cookies::Cookie;
use std::borrow::{Borrow, Cow};
use std::collections::BTreeMap;
use std::fs::read_to_string;
use std::ops::Not;

pub(crate) struct CookieMatcher {
    base: MultiValueExactMatcher,
}

impl CookieMatcher {
    pub fn new(weight: f32) -> Self {
        Self {
            base: MultiValueExactMatcher {
                entity_name: "cookie",
                source: Box::new(CookieSource::new()),
            },
        }
    }
}

impl Matcher for CookieMatcher {
    fn matches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> bool {
        self.base.matches(req, mock)
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch> {
        self.base.mismatches(req, mock)
    }
}
