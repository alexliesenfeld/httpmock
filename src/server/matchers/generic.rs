use crate::data::{HttpMockRequest, RequestRequirements};
use crate::server::matchers::comparators::ValueComparator;
use crate::server::matchers::sources::{MultiValueSource, ValueSource};
use crate::server::matchers::targets::{MultiValueTarget, ValueRefTarget, ValueTarget};
use crate::server::matchers::transformers::Transformer;
use crate::server::matchers::{diff_str, distance, Matcher, Reason};
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
    pub diff_with: Option<Tokenizer>,
    pub weight: usize,
}

impl<S, T> SingleValueMatcher<S, T>
where
    S: Display,
    T: Display,
{
    fn find_unmatched<'a>(
        &self,
        req_value: &Option<T>,
        mock_values: &Option<Vec<&'a S>>,
    ) -> Vec<&'a S> {
        let mock_values = match mock_values {
            None => return Vec::new(),
            Some(mv) => mv.to_vec()
        };
        let req_value = match req_value {
            None => return mock_values,
            Some(rv) => rv
        };
        mock_values
            .into_iter()
            .filter(|e| !self.comparator.matches(e, req_value))
            .collect()
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
        self.find_unmatched(&req_value, &mock_value).is_empty()
    }

    fn distance(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> usize {
        let req_value = self.target.parse_from_request(req);
        let mock_values = self.source.parse_from_mock(mock);
        self.find_unmatched(&req_value, &mock_values)
            .into_iter()
            .map(|s| self.comparator.distance(&Some(s), &req_value.as_ref()))
            .map(|d| d * self.weight)
            .sum()
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch> {
        let req_value = self.target.parse_from_request(req);
        let mock_value = self.source.parse_from_mock(mock);
        self.find_unmatched(&req_value, &mock_value)
            .into_iter()
            .map(|mock_value| {
                let mock_value = mock_value.to_string();
                let req_value = req_value.as_ref().unwrap().to_string();
                Mismatch {
                    title: format!("The {} does not match", self.entity_name),
                    reason: match self.with_reason {
                        true => Some(Reason {
                            expected: mock_value.to_owned(),
                            actual: req_value.to_owned(),
                            comparison: self.comparator.name().into(),
                            best_match: false,
                        }),
                        false => None,
                    },
                    diff: self.diff_with.map(|t| diff_str(&mock_value, &req_value, t)),
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
    pub diff_with: Option<Tokenizer>,
    pub weight: usize,
}

impl<SK, SV, TK, TV> MultiValueMatcher<SK, SV, TK, TV>
where
    SK: Display,
    SV: Display,
    TK: Display,
    TV: Display,
{
    fn find_unmatched<'a>(
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
                        let key_matches = self.key_comparator.matches(sk, &tk);
                        let value_matches = match (sv, tv) {
                            (Some(_), None) => false, // Mock required a value but none was present
                            (Some(sv), Some(tv)) => self.value_comparator.matches(sv, &tv),
                            _ => true,
                        };
                        key_matches && value_matches
                    })
                    .is_none()
            })
            .collect()
    }

    fn find_best_match<'a>(
        &self,
        sk: &SK,
        sv: &Option<&SV>,
        req_values: &'a Vec<(TK, Option<TV>)>,
    ) -> Option<(&'a TK, &'a Option<TV>)> {
        if req_values.is_empty() {
            return None;
        }

        let found = req_values
            .into_iter()
            .find(|(k, v)| k.to_string().eq(&sk.to_string()));
        if let Some((fk, fv)) = found {
            return Some((fk, fv));
        }

        req_values
            .into_iter()
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
        self.find_unmatched(&req_values, &mock_values).is_empty()
    }

    fn distance(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> usize {
        let req_values = self.target.parse_from_request(req).unwrap_or(Vec::new());
        let mock_values = self.source.parse_from_mock(mock).unwrap_or(Vec::new());
        self.find_unmatched(&req_values, &mock_values)
            .into_iter()
            .map(|(k, v)| (k, v, self.find_best_match(&k, v, &req_values)))
            .map(|(k, v, best_match)| match best_match {
                None => {
                    self.key_comparator.distance(&Some(k), &None)
                        + self.value_comparator.distance(v, &None)
                }
                Some((bmk, bmv)) => {
                    self.key_comparator.distance(&Some(k), &Some(bmk))
                        + self.value_comparator.distance(v, &bmv.as_ref())
                }
            })
            .map(|d| d * self.weight)
            .sum()
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch> {
        let req_values = self.target.parse_from_request(req).unwrap_or(Vec::new());
        let mock_values = self.source.parse_from_mock(mock).unwrap_or(Vec::new());
        self.find_unmatched(&req_values, &mock_values)
            .into_iter()
            .map(|(k, v)| (k, v, self.find_best_match(&k, v, &req_values)))
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
                        comparison: format!("key={}, value={}", self.key_comparator.name(), self.value_comparator.name()),
                        best_match: true,
                    }
                }),
                diff: None,
            })
            .collect()
    }
}
