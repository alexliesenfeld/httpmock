use std::collections::BTreeMap;
use std::fmt::Display;

use basic_cookies::Cookie;
use difference::{Changeset, Difference};
use serde::{Deserialize, Serialize};

use crate::common::data::{
    Diff, DiffResult, HttpMockRequest, Mismatch, RequestRequirements, Tokenizer,
};

pub(crate) mod comparators;
pub(crate) mod generic;
pub(crate) mod sources;
pub(crate) mod targets;
pub(crate) mod transformers;

pub(crate) fn diff_str(base: &str, edit: &str, tokenizer: Tokenizer) -> DiffResult {
    let splitter = match tokenizer {
        Tokenizer::Line => "\n",
        Tokenizer::Word => " ",
        Tokenizer::Character => "",
    };

    let changes = Changeset::new(base, edit, splitter);
    DiffResult {
        tokenizer,
        distance: changes.distance,
        differences: changes
            .diffs
            .iter()
            .map(|d| match d {
                Difference::Same(v) => Diff::Same(v.to_owned()),
                Difference::Add(v) => Diff::Add(v.to_owned()),
                Difference::Rem(v) => Diff::Rem(v.to_owned()),
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
