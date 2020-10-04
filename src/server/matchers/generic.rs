use crate::data::{HttpMockRequest, RequestRequirements};
use crate::server::matchers::comparators::ValueComparator;
use crate::server::matchers::targets::{ValueRefTarget, ValueTarget, MultiValueTarget};
use crate::server::matchers::util::{diff_str_new, distance_for, distance_for_vec, match_json};
use crate::server::matchers::{diff_str, Matcher, SimpleDiffResult};
use crate::server::{Mismatch, Tokenizer};
use assert_json_diff::assert_json_eq_no_panic;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::net::ToSocketAddrs;
use crate::server::matchers::sources::{ValueSource, MultiValueSource};
use crate::server::matchers::decoders::ValueDecoder;

// ************************************************************************************************
// SingleValueMatcher
// ************************************************************************************************
pub(crate) struct SingleValueMatcher<S, T>
where
    S: Display,
    T: Display,
{
    pub entity_name: &'static str,
    pub source: Box<dyn ValueSource<S> + Send + Sync>,
    pub target: Box<dyn ValueTarget<T> + Send + Sync>,
    pub comparator: Box<dyn ValueComparator<S, T> + Send + Sync>,
    pub decoder: Option<Box<dyn ValueDecoder<T, T> + Send + Sync>>,
    pub with_reason: bool,
    pub with_diff: bool,
}

impl<S, T> SingleValueMatcher<S, T>
where
    S: Display,
    T: Display,
{
    fn distance(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> usize {
        let mock_value = match self.source.parse_from_mock(mock) {
            None => Vec::new(),
            Some(v) => v.into_iter().map(|e| e.to_string()).collect(),
        };

        let req_value = match self.target.parse_from_request(req) {
            None => String::new(),
            Some(v) => v.to_string(),
        };

        distance_for_vec(&req_value, &mock_value)
    }
}

impl<S, T> Matcher for SingleValueMatcher<S, T>
where
    S: Display,
    T: Display,
{
    fn matches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> bool {
        let req_value = self.target.parse_from_request(req);
        let mock_value = self.source.parse_from_mock(mock);

        match (mock_value, req_value) {
            (Some(mv), Some(rv)) => mv.into_iter().all(|e| self.comparator.matches(e, &rv)),
            (Some(_), None) => false,
            _ => true,
        }
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch> {
        let req_value = self.target.parse_from_request(req);
        let mock_value = self.source.parse_from_mock(mock);

        let unmatched = match (mock_value, req_value) {
            (Some(mv), Some(rv)) => mv
                .into_iter()
                .filter(|e| !self.comparator.matches(e, &rv))
                .collect(),
            (Some(mv), None) => mv,
            _ => return Vec::new(),
        };

        let req_value = self
            .target
            .parse_from_request(req)
            .map_or(String::new(), |v| v.to_string());

        unmatched
            .into_iter()
            .map(|mock_value| {
                let mock_value = mock_value.to_string();
                Mismatch {
                    title: format!("The {} does not match", self.entity_name),
                    reason: match self.with_reason {
                        true => Some(SimpleDiffResult {
                            expected: mock_value.to_string(),
                            actual: req_value.to_string(),
                            operation_name: self.comparator.operation_name().to_string(),
                            best_match: false,
                        }),
                        false => None,
                    },
                    detailed_diff: match self.with_reason {
                        true => Some(diff_str(&mock_value, &req_value, Tokenizer::Line)),
                        false => None,
                    },
                    message: None,
                    score: 0,
                }
            })
            .collect()
    }
}




















// ************************************************************************************************
// MultiValueMatcher
// ************************************************************************************************
pub(crate) struct MultiValueMatcher<SK, SV, TK, TV>
    where
        SK: Display,
        SV: Display,
        TK: Display,
        TV: Display,
{
    pub entity_name: &'static str,
    pub source: Box<dyn MultiValueSource<SK, SV> + Send + Sync>,
    pub target: Box<dyn MultiValueTarget<TK, TV> + Send + Sync>,
    pub key_comparator: Box<dyn ValueComparator<SK, TK> + Send + Sync>,
    pub value_comparator: Box<dyn ValueComparator<SV, TV> + Send + Sync>,
    pub key_decoder: Option<Box<dyn ValueDecoder<SK, SK> + Send + Sync>>,
    pub value_decoder: Option<Box<dyn ValueDecoder<SV, SV> + Send + Sync>>,
    pub with_reason: bool,
    pub with_diff: bool,
}

impl<SK, SV, TK, TV> MultiValueMatcher<SK, SV, TK, TV>
    where
        SK: Display,
        SV: Display,
        TK: Display,
        TV: Display,
{
    /*
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
                    .filter(|(k, v)| !req_headers.contains_entry_with_case_insensitive_key(k, v))
                    .collect(),
            })
    }
*/

}

impl<SK, SV, TK, TV> Matcher for MultiValueMatcher<SK, SV, TK, TV>
    where
        SK: Display,
        SV: Display,
        TK: Display,
        TV: Display,
{
    fn matches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> bool {
        true
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch> {
        Vec::new()
    }
}
