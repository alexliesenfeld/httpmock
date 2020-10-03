use crate::data::{HttpMockRequest, Pattern, RequestRequirements};
use crate::server::matchers::util::{distance_for, distance_for_opt};
use crate::server::matchers::{diff_str, Matcher, Mismatch, SimpleDiffResult, Tokenizer};

pub(crate) struct PathRegexMatcher {}

impl PathRegexMatcher {
    pub fn new(weight: f32) -> Self {
        Self {}
    }

    fn get_unmatched<'a>(
        &self,
        req: &HttpMockRequest,
        mock: &'a RequestRequirements,
    ) -> Vec<&'a Pattern> {
        mock.path_matches.as_ref().map_or(Vec::new(), |patterns| {
            patterns
                .iter()
                .filter(|pattern| !pattern.regex.is_match(&req.path))
                .collect()
        })
    }
}

impl Matcher for PathRegexMatcher {
    fn matches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> bool {
        self.get_unmatched(req, mock).is_empty()
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch> {
        self.get_unmatched(req, mock)
            .iter()
            .map(|pattern| Mismatch {
                title: "Request path does not contain the expected substring".to_string(),
                message: None,
                simple_diff: Some(SimpleDiffResult {
                    expected: format!("{:?}", pattern),
                    actual: req.path.to_owned(),
                    best_match: false,
                }),
                detailed_diff: None,
                score: distance_for(&format!("{:?}", pattern), &req.path),
            })
            .collect()
    }
}
