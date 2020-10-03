use crate::data::{HttpMockRequest, RequestRequirements};
use crate::server::matchers::sources::MultiValueValueSource;
use crate::server::matchers::util::{diff_str_new, distance_for};
use crate::server::matchers::{Matcher, SimpleDiffResult};
use crate::server::Mismatch;
use std::collections::BTreeMap;

pub(crate) struct MultiValueExactMatcher {
    pub entity_name: &'static str,
    pub source: Box<dyn MultiValueValueSource + Send + Sync>,
}

impl MultiValueExactMatcher {
    fn get_unmatched<'a>(
        &self,
        req: &HttpMockRequest,
        mock: &'a RequestRequirements,
    ) -> BTreeMap<&'a String, &'a String> {
        let req_values = self.source.parse_from_request(req);
        let mock_values = match self.source.parse_from_mock(mock) {
            None => return BTreeMap::new(),
            Some(v) => v,
        };

        mock_values
            .into_iter()
            .filter(|(mk, mv)| match req_values.get(*mk) {
                None => true,
                Some(val) => !mv.eq(&val),
            })
            .collect()
    }

    fn get_best_match(
        &self,
        key: &str,
        value: &str,
        req: &HttpMockRequest,
    ) -> Option<(String, String)> {
        let req_values = self.source.parse_from_request(req);

        if let Some(v) = req_values.get(key) {
            return Some((key.to_string(), v.to_string()));
        }

        req_values
            .iter()
            .map(|(k, v)| {
                let d = diff_str_new(&format!("{}{}", &key, value), &format!("{}{}", &k, v));
                (k, v, d)
            })
            .min_by(|(_, _, d1), (_, _, d2)| d1.cmp(d2))
            .map(|(k, v, _)| (k.to_owned(), v.to_owned()))
    }

    fn distance(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> usize {
        self.get_unmatched_with_best_match(req, mock)
            .into_iter()
            .map(|(k, v, best_match)| {
                let bm_str = best_match
                    .as_ref()
                    .map_or(String::new(), |(bmk, bmv)| format!("{}{}", bmk, bmv));
                distance_for(&format!("{}{}", k, v), &bm_str)
            })
            .sum()
    }

    fn get_unmatched_with_best_match<'a>(
        &self,
        req: &HttpMockRequest,
        mock: &'a RequestRequirements,
    ) -> Vec<(&'a String, &'a String, Option<(String, String)>)> {
        self.get_unmatched(req, mock)
            .into_iter()
            .map(|(k, v)| (k, v, self.get_best_match(&k, &v, req)))
            .collect()
    }
}

impl Matcher for MultiValueExactMatcher {
    fn matches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> bool {
        self.get_unmatched(req, mock).is_empty()
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch> {
        self.get_unmatched_with_best_match(req, mock)
            .into_iter()
            .map(|(k, v, best_match)| Mismatch {
                title: format!("Expected {} with name '{}' with value '{}' to be present in the request but it wasn't.", self.entity_name, &k, &v),
                message: None,
                score: 0,
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
