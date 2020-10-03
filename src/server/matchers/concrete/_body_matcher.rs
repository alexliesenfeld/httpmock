use crate::data::{HttpMockRequest, RequestRequirements};
use crate::server::matchers::generic::value_exact::ValueExactMatcher;
use crate::server::matchers::sources::body_source::BodySource;
use crate::server::matchers::util::{distance_for, distance_for_opt};
use crate::server::matchers::{diff_str, Matcher, Mismatch, Tokenizer};

pub(crate) struct BodyMatcher {
    base: ValueExactMatcher,
}

impl BodyMatcher {
    pub fn new(weight: f32) -> Self {
        Self {
            base: ValueExactMatcher {
                entity_name: "body",
                source: Box::new(BodySource::new()),
            },
        }
    }
}

impl Matcher for BodyMatcher {
    fn matches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> bool {
        self.base.matches(req, mock)
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch> {
        self.base.mismatches(req, mock)
    }
}
