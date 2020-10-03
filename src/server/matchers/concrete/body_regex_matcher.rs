use crate::data::{HttpMockRequest, Pattern, RequestRequirements};
use crate::server::matchers::util::distance_for;
use crate::server::matchers::{diff_str, Matcher, Mismatch, SimpleDiffResult, Tokenizer};

pub(crate) struct BodyRegexMatcher {}

impl BodyRegexMatcher {
    pub fn new(weight: f32) -> Self {
        Self {}
    }

    fn get_unmatched<'a>(
        &self,
        req: &HttpMockRequest,
        mock: &'a RequestRequirements,
    ) -> Vec<&'a Pattern> {
        mock.body_matches.as_ref().map_or(Vec::new(), |patterns| {
            patterns
                .iter()
                .filter(|p| {
                    !p.regex
                        .is_match(&req.body.as_ref().unwrap_or(&String::new()))
                })
                .collect()
        })
    }
}

impl Matcher for BodyRegexMatcher {
    fn matches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> bool {
        self.get_unmatched(req, mock).is_empty()
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch> {
        self.get_unmatched(req, mock)
            .iter()
            .map(|pattern| Mismatch {
                title: format!(
                    "Expected body to match regex pattern '{:?}' but it didn't",
                    pattern
                ),
                message: None,
                score: distance_for(
                    &mock.body.as_ref().unwrap_or(&String::new()),
                    &req.body.as_ref().unwrap_or(&String::new()),
                ),
                simple_diff: None,
                detailed_diff: None,
            })
            .collect()
    }
}
