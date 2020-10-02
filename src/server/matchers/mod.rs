use crate::data::{HttpMockRequest, RequestRequirements};
use difference::{Changeset, Difference};

pub(crate) mod body_contains_matcher;
pub(crate) mod body_json_includes_matcher;
pub(crate) mod body_json_matcher;
pub(crate) mod body_matcher;
pub(crate) mod body_regex_matcher;
pub(crate) mod cookie_exists_matcher;
pub(crate) mod cookie_matcher;
pub(crate) mod custom_function_matcher;
pub(crate) mod header_exists_matcher;
pub(crate) mod header_matcher;
pub(crate) mod method_matcher;
pub(crate) mod path_contains_matcher;
pub(crate) mod path_matcher;
pub(crate) mod path_regex_matcher;
pub(crate) mod query_parameter_exists_matcher;
pub(crate) mod query_parameter_matcher;
mod util;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// *************************************************************************************************
// Diff and Change correspond to difference::Changeset and Difference structs. They are duplicated
// here only for the reason to make them serializable/deserializable using serde.
// *************************************************************************************************
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub(crate) enum Diff {
    Same(String),
    Add(String),
    Rem(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct DetailedDiffResult {
    differences: Vec<Diff>,
    distance: i32,
    tokenizer: Tokenizer,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone, Copy)]
pub(crate) enum Tokenizer {
    Line,
    Word,
    Character,
}

pub(crate) fn diff_str(base: &str, edit: &str, tokenizer: Tokenizer) -> DetailedDiffResult {
    let splitter = match tokenizer {
        Tokenizer::Line => "\n",
        Tokenizer::Word => " ",
        Tokenizer::Character => "",
    };

    let changes = Changeset::new(base, edit, splitter);
    DetailedDiffResult {
        tokenizer,
        distance: changes.distance,
        differences: changes
            .diffs
            .iter()
            .map(|d| match d {
                Difference::Same(v) => Diff::Same(v.to_string()),
                Difference::Add(v) => Diff::Add(v.to_string()),
                Difference::Rem(v) => Diff::Rem(v.to_string()),
            })
            .collect(),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct SimpleDiffResult {
    expected: String,
    actual: String,
    best_match: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Mismatch {
    pub title: String,
    pub message: Option<String>,
    pub simple_diff: Option<SimpleDiffResult>,
    pub detailed_diff: Option<DetailedDiffResult>,
    pub score: f32,
}

pub(crate) trait Matcher {
    fn matches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> bool;
    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch>;
}
