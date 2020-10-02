use crate::data::{HttpMockRequest, Pattern, RequestRequirements};
use crate::server::matchers::util::{match_json, score_for};
use crate::server::matchers::{diff_str, Matcher, Mismatch, SimpleDiffResult, Tokenizer};

pub(crate) struct BodyJsonMatcher {}

impl BodyJsonMatcher {
    pub fn new(weight: f32) -> Self {
        Self {}
    }
}

impl Matcher for BodyJsonMatcher {
    fn matches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> bool {
        mock.json_body
            .as_ref()
            .map_or(true, |mock_body| match_json(&req.body, mock_body, true))
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch> {
        if self.matches(req, mock) {
            return Vec::new();
        }

        let mock_body = mock
            .json_body
            .as_ref()
            .map_or(String::new(), |v| v.to_string());

        let req_body = req.body.as_ref().map_or("", |x| &**x);

        vec![Mismatch {
            title: "Request body does not match the expected JSON value".to_string(),
            message: None,
            score: score_for(&mock_body, &req_body),
            simple_diff: None,
            detailed_diff: Some(diff_str(&mock_body, req_body, Tokenizer::Line)),
        }]
    }
}
