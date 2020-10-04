use crate::data::{HttpMockRequest, RequestRequirements};
use crate::server::matchers::util::distance_for;
use crate::server::matchers::{diff_str, Matcher, Mismatch, SimpleDiffResult, Tokenizer};

pub(crate) struct PathContainsMatcher {}

impl PathContainsMatcher {
    pub fn new(weight: f32) -> Self {
        Self {}
    }

    fn get_unmatched<'a>(
        &self,
        req: &HttpMockRequest,
        mock: &'a RequestRequirements,
    ) -> Vec<&'a String> {
        mock.path_contains
            .as_ref()
            .map_or(Vec::new(), |path_contains| {
                path_contains
                    .iter()
                    .filter(|&pc| !req.path.contains(pc))
                    .collect()
            })
    }
}

impl Matcher for PathContainsMatcher {
    fn matches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> bool {
        self.get_unmatched(req, mock).is_empty()
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch> {
        self.get_unmatched(req, mock)
            .iter()
            .map(|substring| Mismatch {
                title: "Expected request path does not match".to_string(),
                message: None,
                reason: Some(SimpleDiffResult {
                    expected: format!("...{}...", substring),
                    actual: req.path.to_owned(),
                    operation_name: "TODO".to_string(),
                    best_match: false,
                }),
                detailed_diff: None,
                score: distance_for(&substring, &req.path),
            })
            .collect()
    }
}
