use crate::data::{HttpMockRequest, Pattern, RequestRequirements};
use crate::server::matchers::util::distance_for;
use crate::server::matchers::{diff_str, Matcher, Mismatch, SimpleDiffResult, Tokenizer};

pub(crate) struct CustomFunctionMatcher {}

impl CustomFunctionMatcher {
    pub fn new(weight: f32) -> Self {
        Self {}
    }

    fn get_unmatched(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<usize> {
        /*mock.matchers.as_ref().map_or(Vec::new(), |matchers| {
            matchers
                .iter()
                .enumerate()
                .filter(|(idx, f)| (f)(req.clone()))
                .map(|(idx, f)| idx)
                .collect()
        })*/
        Vec::new()
    }
}

impl Matcher for CustomFunctionMatcher {
    fn matches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> bool {
        self.get_unmatched(req, mock).is_empty()
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch> {
        self.get_unmatched(req, mock)
            .iter()
            .map(|idx| Mismatch {
                title: format!(
                    "Expected request to match custom matching function at position '{}' but it didn't.",
                    idx + 1
                ),
                message: None,
                score: distance_for(&mock.body.as_ref().unwrap_or(&String::new()), &req.body.as_ref().unwrap_or(&String::new())),
                reason: None,
                detailed_diff: None,
            })
            .collect()
    }
}
