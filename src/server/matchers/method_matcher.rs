use crate::data::{HttpMockRequest, RequestRequirements};
use crate::server::matchers::util::{score_for, score_for_opt};
use crate::server::matchers::{Matcher, Mismatch, SimpleDiffResult};

pub(crate) struct MethodMatcher {}

impl MethodMatcher {
    pub fn new(weight: f32) -> Self {
        Self {}
    }
}

impl Matcher for MethodMatcher {
    fn matches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> bool {
        mock.method
            .as_ref()
            .map_or(true, |method| method.eq(&req.method))
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch> {
        match self.matches(req, mock) {
            true => Vec::new(),
            false => vec![Mismatch {
                title: "Request method does not match".to_string(),
                message: None,
                simple_diff: Some(SimpleDiffResult {
                    expected: mock.method.as_ref().unwrap().to_owned(),
                    actual: req.method.to_owned(),
                    best_match: false,
                }),
                detailed_diff: None,
                score: score_for_opt(&req.method, &mock.method.as_ref()),
            }],
        }
    }
}
