use crate::data::{HttpMockRequest, RequestRequirements};
use crate::server::matchers::util::{score_for, score_for_opt};
use crate::server::matchers::{diff_str, Matcher, Mismatch, Tokenizer};

pub(crate) struct BodyMatcher {}

impl BodyMatcher {
    pub fn new(weight: f32) -> Self {
        Self {}
    }
}

impl Matcher for BodyMatcher {
    fn matches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> bool {
        mock.body.as_ref().map_or(true, |mock_body| {
            req.body
                .as_ref()
                .map_or(true, |req_body| mock_body.eq(req_body))
        })
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch> {
        if !self.matches(req, mock) {
            return Vec::new();
        }

        let mock_body = mock
            .json_body
            .as_ref()
            .map_or(String::new(), |v| v.to_string());
        let req_body = req.body.as_ref().map_or("", |x| &**x);

        vec![Mismatch {
            title: "Request body does not match".to_string(),
            message: None,
            score: score_for(&mock_body, &req_body),
            simple_diff: None,
            detailed_diff: Some(diff_str(&mock_body, req_body, Tokenizer::Line)),
        }]
    }
}
