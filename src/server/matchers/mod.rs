use crate::data::{HttpMockRequest, RequestRequirements};
use difference::{Changeset, Difference};

pub(crate) mod comparators;
pub(crate) mod decoders;
pub(crate) mod generic;
pub(crate) mod sources;
pub(crate) mod targets;
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
    pub differences: Vec<Diff>,
    pub distance: i32,
    pub tokenizer: Tokenizer,
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
    pub expected: String,
    pub actual: String,
    pub operation_name: String,
    pub best_match: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Mismatch {
    pub title: String,
    pub message: Option<String>,
    pub reason: Option<SimpleDiffResult>,
    pub detailed_diff: Option<DetailedDiffResult>,
    pub score: usize,
}

pub(crate) trait Matcher {
    fn matches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> bool;
    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch>;
}
