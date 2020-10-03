use crate::data::HttpMockRequest;
use crate::server::matchers::{diff_str, Tokenizer};
use assert_json_diff::{assert_json_eq_no_panic, assert_json_include_no_panic};
use basic_cookies::Cookie;
use serde_json::Value;
use std::collections::BTreeMap;
use std::ops::Deref;
use std::str::FromStr;

/// Matches JSON
pub(crate) fn match_json(req: &Option<String>, mock: &Value, exact: bool) -> bool {
    match req {
        Some(req_string) => {
            // Parse the request body as JSON string
            let result = serde_json::Value::from_str(req_string);
            if let Err(e) = result {
                log::trace!("cannot deserialize request body to JSON: {}", e);
                return false;
            }
            let req_value = result.unwrap();

            log::trace!(
                "Comapring the following JSON values: (1){}, (2){}",
                &req_value,
                &mock
            );

            // Compare JSON values
            let result = if exact {
                assert_json_eq_no_panic(&req_value, mock)
            } else {
                assert_json_include_no_panic(&req_value, mock)
            };

            // Log and return the comparison result
            match result {
                Err(e) => {
                    log::trace!("Request body does not match mock JSON body: {}", e);
                    false
                }
                _ => {
                    log::trace!("Request body matched mock JSON body");
                    true
                }
            }
        }
        None => false,
    }
}

pub(crate) fn parse_cookies(req: &HttpMockRequest) -> Result<BTreeMap<String, String>, String> {
    let parsing_result = req.headers.as_ref().map_or(None, |request_headers| {
        request_headers
            .iter()
            .find(|(k, _)| k.to_lowercase().eq("cookie"))
            .map(|(k, v)| Cookie::parse(v))
    });

    match parsing_result {
        None => Ok(BTreeMap::new()),
        Some(res) => match res {
            Err(e) => Err(e.to_string()),
            Ok(v) => Ok(v
                .into_iter()
                .map(|c| (c.get_name().to_lowercase(), c.get_value().to_owned()))
                .collect()),
        },
    }
}

pub(crate) fn distance_for(expected: &str, actual: &str) -> usize {
    let max_distance = (expected.len() + actual.len());
    if max_distance == 0 {
        return 0;
    }
    let distance = levenshtein::levenshtein(expected, actual);
    100 - ((max_distance - distance) / max_distance)
}

pub(crate) fn distance_for_opt(expected: &str, actual: &Option<&String>) -> usize {
    let actual = actual.as_ref().map_or("", |x| &**x);
    distance_for(expected, actual)
}

pub(crate) fn diff_str_new(s1: &str, s2: &str) -> usize {
    levenshtein::levenshtein(s1, s2)
}
