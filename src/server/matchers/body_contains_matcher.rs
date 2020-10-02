use crate::data::{HttpMockRequest, RequestRequirements};
use crate::server::matchers::util::{score_for, score_for_opt};
use crate::server::matchers::{Matcher, Mismatch, SimpleDiffResult};

pub(crate) struct BodyContainsMatcher {}

impl BodyContainsMatcher {
    pub fn new(weight: f32) -> Self {
        Self {}
    }

    fn get_unmatched<'a>(
        &self,
        req: &HttpMockRequest,
        mock: &'a RequestRequirements,
    ) -> Vec<&'a String> {
        mock.body_contains
            .as_ref()
            .map_or(Vec::new(), |all_substrings| {
                all_substrings
                    .iter()
                    .filter(|substring| !substring.is_empty())
                    .filter(|&substring| {
                        !req.body.as_ref().map_or(false, |b| b.contains(substring))
                    })
                    .collect()
            })
    }
}

impl Matcher for BodyContainsMatcher {
    fn matches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> bool {
        self.get_unmatched(req, mock).is_empty()
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch> {
        self.get_unmatched(req, mock)
            .iter()
            .map(|substring| Mismatch {
                title: "Expected request body to contain the following substring but it didn't"
                    .to_string(),
                message: Some(substring.to_string()),
                simple_diff: None,
                detailed_diff: None,
                score: score_for_opt(substring, &req.body.as_ref()),
            })
            .collect()
    }
}
