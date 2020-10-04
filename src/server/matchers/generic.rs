use crate::data::{HttpMockRequest, RequestRequirements};
use crate::server::matchers::comparators::ValueComparator;
use crate::server::matchers::transformers::Transformer;
use crate::server::matchers::sources::{MultiValueSource, ValueSource};
use crate::server::matchers::targets::{MultiValueTarget, ValueRefTarget, ValueTarget};
use crate::server::matchers::{
    diff_str, distance_for, distance_for_vec, Matcher, Reason,
};
use crate::server::{Mismatch, Tokenizer};
use assert_json_diff::assert_json_eq_no_panic;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::net::ToSocketAddrs;

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
    pub transformer: Option<Box<dyn Transformer<T, T> + Send + Sync>>,
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
            None => return 0,
            Some(v) => v,
        };
        let req_value = self.target.parse_from_request(req);
        mock_value
            .into_iter()
            .map(|s| self.comparator.distance(&Some(s), &req_value.as_ref()))
            .sum()
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

    fn distance(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> usize {
        0
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
                        true => Some(Reason {
                            expected: mock_value.to_string(),
                            actual: req_value.to_string(),
                            best_match: false,
                        }),
                        false => None,
                    },
                    diff: match self.with_reason {
                        true => Some(diff_str(&mock_value, &req_value, Tokenizer::Line)),
                        false => None,
                    },
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
    pub key_transformer: Option<Box<dyn Transformer<SK, SK> + Send + Sync>>,
    pub value_transformer: Option<Box<dyn Transformer<SV, SV> + Send + Sync>>,
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
    fn get_unmatched<'a>(
        &self,
        req_values: &Vec<(TK, Option<TV>)>,
        mock_values: &'a Vec<(&'a SK, Option<&'a SV>)>,
    ) -> Vec<&'a (&'a SK, Option<&'a SV>)> {
        mock_values
            .into_iter()
            .filter(|(sk, sv)| {
                req_values
                    .iter()
                    .find(|(tk, tv)| {
                        let key_matches = self.key_comparator.matches(sk, tk);
                        let value_matches = match (sv, tv) {
                            (Some(_), None) => false, // Mock required a value but none was present
                            (Some(sv), Some(tv)) => self.value_comparator.matches(sv, tv),
                            (None, Some(_)) => true, // Mock did not require any value but there was one
                            (None, None) => true,
                        };
                        key_matches && value_matches
                    })
                    .is_none()
            })
            .collect()
    }

    fn get_best_match<'a>(
        &self,
        sk: &SK,
        sv: &Option<&SV>,
        req_values: &'a Vec<(TK, Option<TV>)>,
    ) -> Option<(&'a TK, &'a Option<TV>)> {
        if req_values.is_empty() {
            return None;
        }

        let found = req_values
            .iter()
            .find(|(k, v)| k.to_string().eq(&sk.to_string()));
        if let Some((fk, fv)) = found {
            return Some((fk, fv));
        }

        req_values
            .iter()
            .map(|(tk, tv)| {
                let key_distance = self.key_comparator.distance(&Some(sk), &Some(tk));
                let value_distance = self.value_comparator.distance(&sv, &tv.as_ref());
                (tk, tv, key_distance + value_distance)
            })
            .min_by(|(_, _, d1), (_, _, d2)| d1.cmp(d2))
            .map(|(k, v, _)| (k.to_owned(), v.to_owned()))
    }
}

impl<SK, SV, TK, TV> Matcher for MultiValueMatcher<SK, SV, TK, TV>
where
    SK: Display,
    SV: Display,
    TK: Display,
    TV: Display,
{
    fn matches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> bool {
        let req_values = self.target.parse_from_request(req).unwrap_or(Vec::new());
        let mock_values = self.source.parse_from_mock(mock).unwrap_or(Vec::new());
        self.get_unmatched(&req_values, &mock_values).is_empty()
    }

    fn distance(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> usize {
        0
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch> {
        let req_values = self.target.parse_from_request(req).unwrap_or(Vec::new());
        let mock_values = self.source.parse_from_mock(mock).unwrap_or(Vec::new());
        self.get_unmatched(&req_values, &mock_values)
            .into_iter()
            .map(|(k, v)| (k, v, self.get_best_match(&k, v, &req_values)))
            .map(|(k, v, best_match)| Mismatch {
                title: match v {
                    None => format!("Expected {} with name '{}' to be present in the request but it wasn't.", self.entity_name, &k),
                    Some(v) => format!("Expected {} with name '{}' and value '{}' to be present in the request but it wasn't.", self.entity_name, &k, v),
                },
                reason: best_match.as_ref().map(|(bmk, bmv)| {
                    Reason {
                        expected: match v {
                            None => format!("{}", k),
                            Some(v) => format!("{}={}", k, v),
                        },
                        actual: match bmv {
                            None => format!("{}", bmk),
                            Some(bmv) => format!("{}={}", bmk, bmv),
                        },
                        best_match: true,
                    }
                }),
                diff: None,
            })
            .collect()
    }
}
