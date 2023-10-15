use std::collections::BTreeMap;
use std::fmt::Display;

#[cfg(feature = "cookies")]
use basic_cookies::Cookie;
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};

use crate::common::data::{
    Diff, DiffResult, HttpMockRequest, Mismatch, RequestRequirements, Tokenizer,
};

pub(crate) mod comparators;
pub(crate) mod generic;
pub(crate) mod sources;
pub(crate) mod targets;
pub(crate) mod transformers;

pub(crate) fn diff_str(base: &str, edit: &str, tokenizer: Tokenizer) -> DiffResult {
    let changes = match tokenizer {
        Tokenizer::Line => TextDiff::from_lines(base, edit),
        Tokenizer::Word => TextDiff::from_words(base, edit),
        Tokenizer::Character => TextDiff::from_chars(base, edit),
    };

    DiffResult {
        tokenizer,
        distance: changes.ratio(),
        differences: changes
            .iter_all_changes()
            .map(|change| match change.tag() {
                ChangeTag::Equal => Diff::Same(change.to_string_lossy().to_string()),
                ChangeTag::Insert => Diff::Add(change.to_string_lossy().to_string()),
                ChangeTag::Delete => Diff::Rem(change.to_string_lossy().to_string()),
            })
            .collect(),
    }
}

pub trait Matcher {
    fn matches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> bool;
    fn distance(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> usize;
    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch>;
}

// *************************************************************************************************
// Helper functions
// *************************************************************************************************
#[cfg(feature = "cookies")]
pub(crate) fn parse_cookies(req: &HttpMockRequest) -> Result<Vec<(String, String)>, String> {
    let parsing_result = req.headers.as_ref().map_or(None, |request_headers| {
        request_headers
            .iter()
            .find(|(k, _)| k.to_lowercase().eq("cookie"))
            .map(|(k, v)| Cookie::parse(v))
    });

    match parsing_result {
        None => Ok(Vec::new()),
        Some(res) => match res {
            Err(err) => Err(err.to_string()),
            Ok(vec) => Ok(vec
                .into_iter()
                .map(|c| (c.get_name().to_owned(), c.get_value().to_owned()))
                .collect()),
        },
    }
}

pub(crate) fn distance_for<T, U>(expected: &Option<&T>, actual: &Option<&U>) -> usize
where
    T: Display,
    U: Display,
{
    let expected = expected.map_or(String::new(), |x| x.to_string());
    let actual = actual.map_or(String::new(), |x| x.to_string());
    levenshtein::levenshtein(&expected, &actual)
}

pub(crate) fn distance_for_binary<A: AsRef<[u8]>, B: AsRef<[u8]>>(a: &Option<A>, b: &Option<B>) -> usize {
    match (a, b) {
        (Some(a_val), Some(b_val)) => {
            let a_slice = a_val.as_ref();
            let b_slice = b_val.as_ref();

            let min_len = std::cmp::min(a_slice.len(), b_slice.len());
            let max_len = std::cmp::max(a_slice.len(), b_slice.len());

            let mut differing_bytes = max_len - min_len;
            for i in 0..min_len {
                if a_slice[i] != b_slice[i] {
                    differing_bytes += 1;
                }
            }

            differing_bytes
        },
        (Some(a_val), None) => a_val.as_ref().len(),
        (None, Some(b_val)) => b_val.as_ref().len(),
        (None, None) => 0,
    }
}