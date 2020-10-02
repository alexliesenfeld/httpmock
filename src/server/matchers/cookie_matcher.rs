use crate::data::{HttpMockRequest, RequestRequirements};
use crate::server::matchers::util::{parse_cookies, score_for};
use crate::server::matchers::{
    diff_str, DetailedDiffResult, Matcher, Mismatch, SimpleDiffResult, Tokenizer,
};
use crate::server::util::StringTreeMapExtension;
use basic_cookies::Cookie;
use std::collections::BTreeMap;
use std::ops::Not;

pub(crate) struct CookieMatcher {}

impl CookieMatcher {
    pub fn new(weight: f32) -> Self {
        Self {}
    }

    fn get_unmatched<'a>(
        &self,
        req: &HttpMockRequest,
        mock: &'a RequestRequirements,
    ) -> BTreeMap<&'a String, &'a String> {
        mock.cookies.as_ref().map_or(BTreeMap::new(), |mock_cookies| {
            let req_cookies = match parse_cookies(req) {
                Ok(v) => v,
                Err(err) => {
                    log::info!("Cannot parse cookies. Cookie matching will not work for this request. Error: {}", err);
                    return BTreeMap::new();
                }
            };
            mock_cookies
                .into_iter()
                .filter(|(mk, mv)| {
                    match req_cookies.get_case_insensitive(mk) {
                        None => true,
                        Some(val) => !mv.eq(&val)
                    }
                })
                .collect()
        })
    }

    fn difference(&self, n1: &str, v1: &str, n2: &str, v2: &str) -> DetailedDiffResult {
        let h1 = format!("{}={}", &n1.to_lowercase(), v1);
        let h2 = format!("{}={}", &n2.to_lowercase(), v2);
        diff_str(&h1, &h2, Tokenizer::Character)
    }

    fn get_best_match(
        &self,
        name: &str,
        value: &str,
        req: &HttpMockRequest,
    ) -> Option<(String, String)> {
        let req_cookies = match parse_cookies(req) {
            Ok(v) => v,
            Err(err) => {
                log::info!("Cannot parse cookies. Cookie matching will not work for this request. Error: {}", err);
                return None;
            }
        };

        if let Some(v) = req_cookies.get_case_insensitive(name) {
            return Some((name.to_string(), v.to_string()));
        }

        req_cookies
            .iter()
            .map(|(k, v)| {
                let diff = self.difference(name, value, k, v);
                (k, v, diff.distance)
            })
            .min_by(|(_, _, d1), (_, _, d2)| d1.cmp(d2))
            .map(|(k, v, _)| (k.to_owned(), v.to_owned()))
    }
}

impl Matcher for CookieMatcher {
    fn matches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> bool {
        self.get_unmatched(req, mock).is_empty()
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch> {
        self.get_unmatched(req, mock)
            .into_iter()
            .map(|(k, v)| (k, v, self.get_best_match(&k, &v, req)))
            .map(|(k, v, best_match)| Mismatch {
                title: format!("Expected cookie '{}' with value '{}' to be present in the request but it wasn't.", &k, &v),
                message: None,
                score: 0.0f32,
                /*score_for(
                    &format!("{}={}", k, v),&best_match.as_ref()
                    .map_or(&String::new(), |(kk,vv)| format!("{}={}", kk, vv)))*/
                simple_diff: best_match.as_ref().map(|(bmk, bmv)| {
                    SimpleDiffResult{
                        expected: format!("{}={}", k, v),
                        actual: format!("{}={}", bmk, bmv),
                        best_match: true
                    }
                }),
                detailed_diff: None,
            })
            .collect()
    }
}
