use crate::data::{HttpMockRequest, RequestRequirements};
use crate::server::matchers::sources::{MultiValueValueSource, ValueSource};
use crate::server::matchers::util::{diff_str_new, distance_for};
use crate::server::matchers::{diff_str, Matcher, SimpleDiffResult};
use crate::server::{Mismatch, Tokenizer};
use std::collections::BTreeMap;

pub(crate) struct ValueExactMatcher {
    pub entity_name: &'static str,
    pub source: Box<dyn ValueSource + Send + Sync>,
}

impl ValueExactMatcher {
    fn distance(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> usize {
        distance_for(
            self.source
                .parse_from_request(req)
                .unwrap_or(&String::new()),
            self.source.parse_from_mock(mock).unwrap_or(&String::new()),
        )
    }
}

impl Matcher for ValueExactMatcher {
    fn matches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> bool {
        let req_value = self.source.parse_from_request(req);
        let mock_value = self.source.parse_from_mock(mock);

        match (mock_value, req_value) {
            (Some(mv), Some(rv)) => mv.eq(rv),
            (Some(_), None) => false,
            _ => true,
        }
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch> {
        if !self.matches(req, mock) {
            return Vec::new();
        }

        let req_value = self.source.parse_from_request(req);
        let mock_value = self.source.parse_from_mock(mock);

        vec![Mismatch {
            title: format!("The {} does not match.", self.entity_name),
            message: None,
            score: distance_for(
                mock_value.unwrap_or(&String::new()),
                req_value.unwrap_or(&String::new()),
            ),
            simple_diff: None,
            detailed_diff: Some(diff_str(
                mock_value.unwrap_or(&String::new()),
                req_value.unwrap_or(&String::new()),
                Tokenizer::Line,
            )),
        }]
    }
}
