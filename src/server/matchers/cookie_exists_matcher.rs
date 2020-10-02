use crate::data::{HttpMockRequest, RequestRequirements};
use crate::server::matchers::util::{parse_cookies, score_for};
use crate::server::matchers::{
    diff_str, DetailedDiffResult, Matcher, Mismatch, SimpleDiffResult, Tokenizer,
};
use crate::server::util::StringTreeMapExtension;
use basic_cookies::Cookie;
use std::collections::BTreeMap;

pub(crate) struct CookieExistsMatcher {}

impl CookieExistsMatcher {
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
                .filter(|(mk, mv)| req_cookies.get_case_insensitive(mk).is_none())
                .collect()
        })
    }

    fn get_best_match(&self, name: &str, req: &HttpMockRequest) -> Option<String> {
        let req_cookies = match parse_cookies(req) {
            Ok(v) => v,
            Err(err) => {
                log::info!("Cannot parse cookies. Cookie matching will not work for this request. Error: {}", err);
                return None;
            }
        };

        req_cookies
            .iter()
            .map(|(k, _)| {
                let diff = diff_str(
                    &name.to_lowercase(),
                    &k.to_lowercase(),
                    Tokenizer::Character,
                );
                (k, diff.distance)
            })
            .min_by(|(_, d1), (_, d2)| d1.cmp(d2))
            .map(|(k, _)| k.to_owned())
    }
}

impl Matcher for CookieExistsMatcher {
    fn matches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> bool {
        self.get_unmatched(req, mock).is_empty()
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch> {
        self.get_unmatched(req, mock)
            .into_iter()
            .map(|(k, v)| (k, self.get_best_match(&k, req)))
            .map(|(k, best_match)| Mismatch {
                title: format!(
                    "Expected cookie '{}' to be present in the request but it wasn't.",
                    &k
                ),
                message: None,
                score: score_for(k, &best_match.as_ref().unwrap_or(&String::new())),
                simple_diff: best_match.as_ref().map(|bm| SimpleDiffResult {
                    expected: k.to_lowercase(),
                    actual: bm.to_lowercase(),
                    best_match: true,
                }),
                detailed_diff: None,
            })
            .collect()
    }
}
