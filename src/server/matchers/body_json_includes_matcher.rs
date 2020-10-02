use crate::data::{HttpMockRequest, Pattern, RequestRequirements};
use crate::server::matchers::util::{match_json, score_for};
use crate::server::matchers::{diff_str, Matcher, Mismatch, SimpleDiffResult, Tokenizer};
use serde_json::Value;

pub(crate) struct BodyJsonIncludesMatcher {}

impl BodyJsonIncludesMatcher {
    pub fn new(weight: f32) -> Self {
        Self {}
    }

    fn get_unmatched<'a>(
        &self,
        req: &HttpMockRequest,
        mock: &'a RequestRequirements,
    ) -> Vec<&'a Value> {
        mock.json_body_includes
            .as_ref()
            .map_or(Vec::new(), |includes| {
                includes
                    .iter()
                    .filter(|v| !match_json(&req.body, v, false))
                    .collect()
            })
    }
}

impl Matcher for BodyJsonIncludesMatcher {
    fn matches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> bool {
        self.get_unmatched(req, mock).is_empty()
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch> {
        self.get_unmatched(req, mock)
            .into_iter()
            .enumerate()
            .map(|(i, v)| {
                let mock_body = mock
                    .json_body
                    .as_ref()
                    .map_or(String::new(), |v| v.to_string());
                let req_body = req.body.as_ref().map_or("", |x| &**x);
                Mismatch {
                    title: format!(
                        "Request body does not include the expected JSON value at index {}",
                        i
                    ),
                    message: None,
                    simple_diff: None,
                    detailed_diff: Some(diff_str(&mock_body, req_body, Tokenizer::Line)),
                    score: score_for(&mock_body, &req_body),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod test {

    /// TODO
    #[test]
    fn header_names_case_insensitive() {}
}
