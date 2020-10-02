use crate::data::{HttpMockRequest, RequestRequirements};
use crate::server::matchers::util::{parse_cookies, score_for_opt};
use crate::server::matchers::{
    diff_str, DetailedDiffResult, Matcher, Mismatch, SimpleDiffResult, Tokenizer,
};
use crate::server::util::StringTreeMapExtension;
use basic_cookies::Cookie;
use std::collections::BTreeMap;
use std::ops::Not;

pub(crate) struct QueryParameterExistsMatcher {}

impl QueryParameterExistsMatcher {
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
                    .filter(|(mk, _)| {
                        req.query_params.as_ref().map_or(true, |req_params| {
                            (&req_params).get(mk.to_owned()).is_none()
                        })
                    })
                    .collect()
            })
    }

    fn get_best_match(&self, name: &str, req: &HttpMockRequest) -> Option<String> {
        let req_params = match req.query_params.as_ref() {
            None => return None,
            Some(v) => v,
        };

        req_params
            .iter()
            .map(|(k, _)| {
                let diff = diff_str(&name, &k, Tokenizer::Character);
                (k, diff.distance)
            })
            .min_by(|(_, d1), (_, d2)| d1.cmp(d2))
            .map(|(k, _)| k.to_owned())
    }
}

impl Matcher for QueryParameterExistsMatcher {
    fn matches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> bool {
        self.get_unmatched(req, mock).is_empty()
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch> {
        self.get_unmatched(req, mock)
            .into_iter()
            .map(|(k, _)| (k, self.get_best_match(&k, req)))
            .map(|(k, best_match)| Mismatch {
                title: "Query parameter missing".to_string(),
                message: None,
                simple_diff: best_match.as_ref().map(|bmk| SimpleDiffResult {
                    expected: k.to_owned(),
                    actual: bmk.to_owned(),
                    best_match: true,
                }),
                detailed_diff: None,
                score: score_for_opt(k, &best_match.as_ref()),
            })
            .collect()
    }
}
