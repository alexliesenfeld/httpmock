use crate::data::{HttpMockRequest, RequestRequirements};
use crate::server::matchers::util::distance_for_opt;
use crate::server::matchers::{
    diff_str, DetailedDiffResult, Matcher, Mismatch, SimpleDiffResult, Tokenizer,
};
use crate::server::util::StringTreeMapExtension;

pub(crate) struct HeaderExistsMatcher {}

impl HeaderExistsMatcher {
    pub fn new(weight: f32) -> Self {
        Self {}
    }

    fn get_unmatched<'a>(
        &self,
        req: &HttpMockRequest,
        mock: &'a RequestRequirements,
    ) -> Vec<(&'a String, &'a String)> {
        mock.headers
            .as_ref()
            .map_or(Vec::new(), |mock_headers| match req.headers.as_ref() {
                None => Vec::new(),
                Some(req_headers) => mock_headers
                    .iter()
                    .filter(|(k, _)| !req_headers.contains_case_insensitive_key(k))
                    .collect(),
            })
    }

    fn get_best_match(&self, name: &str, req: &HttpMockRequest) -> Option<String> {
        if req.headers.as_ref().is_none() {
            return None;
        }

        req.headers
            .as_ref()
            .unwrap()
            .iter()
            .map(|(k, _)| {
                (
                    k,
                    diff_str(
                        &name.to_lowercase(),
                        &k.to_lowercase(),
                        Tokenizer::Character,
                    ),
                )
            })
            .min_by(|(_, d1), (_, d2)| d1.distance.cmp(&d2.distance))
            .map(|(k, _)| k.to_owned())
    }
}

impl Matcher for HeaderExistsMatcher {
    fn matches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> bool {
        self.get_unmatched(req, mock).is_empty()
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch> {
        self.get_unmatched(req, mock)
            .into_iter()
            .map(|(k, v)| (k, self.get_best_match(&k, req)))
            .map(|(k, best_match)| Mismatch {
                title: format!(
                    "Expected header '{}' to be present in the request but it wasn't.",
                    &k
                ),
                message: None,
                simple_diff: best_match.as_ref().map(|bm| SimpleDiffResult {
                    expected: k.to_lowercase(),
                    actual: bm.to_lowercase(),
                    best_match: true,
                }),
                detailed_diff: None,
                score: distance_for_opt(k, &best_match.as_ref()),
            })
            .collect()
    }
}
