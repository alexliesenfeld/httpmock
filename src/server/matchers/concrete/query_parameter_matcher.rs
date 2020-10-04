use crate::data::{HttpMockRequest, RequestRequirements};
use crate::server::matchers::util::{distance_for, parse_cookies};
use crate::server::matchers::{
    diff_str, DetailedDiffResult, Matcher, Mismatch, SimpleDiffResult, Tokenizer,
};
use crate::server::util::StringTreeMapExtension;
use basic_cookies::Cookie;
use std::collections::BTreeMap;
use std::ops::Not;

pub(crate) struct QueryParameterMatcher {}

impl QueryParameterMatcher {
    pub fn new(weight: f32) -> Self {
        Self {}
    }

    fn get_unmatched<'a>(
        &self,
        req: &HttpMockRequest,
        mock: &'a RequestRequirements,
    ) -> BTreeMap<&'a String, &'a String> {
        mock.query_param
            .as_ref()
            .map_or(BTreeMap::new(), |mock_params| {
                mock_params
                    .into_iter()
                    .filter(|(mk, mv)| {
                        !req.query_params.as_ref().map_or(false, |req_params| {
                            match (&req_params).get(mk.to_owned()) {
                                None => false,
                                Some(v) => mv.eq(&v),
                            }
                        })
                    })
                    .collect()
            })
    }

    fn difference(&self, n1: &str, v1: &str, n2: &str, v2: &str) -> DetailedDiffResult {
        let h1 = format!("{}={}", &n1, v1);
        let h2 = format!("{}={}", &n2, v2);
        diff_str(&h1, &h2, Tokenizer::Character)
    }

    fn get_best_match(
        &self,
        name: &str,
        value: &str,
        req: &HttpMockRequest,
    ) -> Option<(String, String)> {
        let req_params = match req.query_params.as_ref() {
            None => return None,
            Some(v) => v,
        };

        if let Some(v) = req_params.get(name) {
            return Some((name.to_string(), v.to_string()));
        }

        req_params
            .iter()
            .map(|(k, v)| {
                let diff = self.difference(name, value, k, v);
                (k, v, diff.distance)
            })
            .min_by(|(_, _, d1), (_, _, d2)| d1.cmp(d2))
            .map(|(k, v, _)| (k.to_owned(), v.to_owned()))
    }
}

impl Matcher for QueryParameterMatcher {
    fn matches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> bool {
        self.get_unmatched(req, mock).is_empty()
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch> {
        self.get_unmatched(req, mock)
            .into_iter()
            .map(|(k, v)| (k, v, self.get_best_match(&k, &v, req)))
            .map(|(k, v, best_match)| Mismatch {
                title: format!("Expected query parameter '{}' with value '{}' to be present in the request but it wasn't.", &k, &v),
                message: None,
                reason: best_match.as_ref().map(|(bmk, bmv)| {
                    SimpleDiffResult{
                        expected: format!("{}={}", k, v),
                        actual: format!("{}={}", bmk, bmv),
                        operation_name: "TODO".to_string(),
                        best_match: true,
                    }
                }),
                detailed_diff: None,
                score: 0,  // TODO: score_for(&format!("{}={}", k, v), best_match.as_ref().map_or(&String::new(), |(kk,vv)| &format!("{}={}", kk, vv)))
            })
            .collect()
    }
}
