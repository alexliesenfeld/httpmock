use crate::data::{HttpMockRequest, RequestRequirements};
use crate::server::matchers::util::{score_for, score_for_opt};
use crate::server::matchers::{
    diff_str, DetailedDiffResult, Matcher, Mismatch, SimpleDiffResult, Tokenizer,
};
use crate::server::util::StringTreeMapExtension;

pub(crate) struct HeaderMatcher {}

impl HeaderMatcher {
    pub fn new(weight: f32) -> Self {
        Self {}
    }

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

    fn difference(
        &self,
        name1: &str,
        value1: &str,
        name2: &str,
        value2: &str,
    ) -> DetailedDiffResult {
        let h1 = format!("{}:{}", &name1.to_lowercase(), value1);
        let h2 = format!("{}:{}", &name2.to_lowercase(), value2);
        diff_str(&h1, &h2, Tokenizer::Character)
    }

    fn get_best_match(
        &self,
        name: &str,
        value: &str,
        req: &HttpMockRequest,
    ) -> Option<(String, String)> {
        if req.headers.as_ref().is_none() {
            return None;
        }

        let headers = req.headers.as_ref().unwrap();
        if let Some(value) = headers.get_case_insensitive(name) {
            return Some((name.to_string(), value.to_string()));
        }

        headers
            .iter()
            .map(|(k, v)| {
                let diff = self.difference(name, value, k, v);
                (k, v, diff.distance)
            })
            .min_by(|(_, _, d1), (_, _, d2)| d1.cmp(d2))
            .map(|(k, v, _)| (k.to_owned(), v.to_owned()))
    }
}

impl Matcher for HeaderMatcher {
    fn matches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> bool {
        self.get_unmatched(req, mock).is_empty()
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch> {
        self.get_unmatched(req, mock)
            .into_iter()
            .map(|(k, v)| (k, v, self.get_best_match(&k, &v, req)))
            .map(|(k, v, best_match)| Mismatch {
                title: format!("Expected header '{}' with value '{}' to be present in the request but it wasn't.", &k, &v),
                message: None,
                simple_diff: best_match.as_ref().map(|(bmk, bmv)| {
                    SimpleDiffResult{
                        expected: format!("{}:{}", k, v),
                        actual: format!("{}:{}", bmk, bmv),
                        best_match: true,
                    }
                }),
                detailed_diff: None,
                score: 0.0, /*score_for(
                    &format!("{}:{}", k, v),
                    best_match.as_ref().map_or(&String::new(), |(kk,vv)| &format!("{}:{}", kk, vv)))*/
            })
            .collect()
    }
}
