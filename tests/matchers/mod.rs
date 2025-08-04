mod body;
mod cookies;
mod headers;
mod host;
mod method;
mod path;
mod port;
mod query_param;
mod scheme;
mod urlencoded_body;

use std::{
    convert::TryInto,
    panic::{self, AssertUnwindSafe, UnwindSafe},
};

pub fn expect_fails_with<F>(f: F, expected_texts: Vec<&str>)
where
    F: FnOnce() + UnwindSafe,
{
    let result = panic::catch_unwind(AssertUnwindSafe(f));

    match result {
        Err(err) => {
            let err_msg: &str = if let Some(err_msg) = err.downcast_ref::<String>() {
                err_msg
            } else if let Some(err_msg) = err.downcast_ref::<&str>() {
                err_msg
            } else {
                panic!(
                    "Expected error message containing:\n{:?}\nBut got a different type of panic.",
                    expected_texts
                );
            };

            // Check that all expected texts appear in order in the error message
            let mut start_index = 0;
            for expected_text in &expected_texts {
                if let Some(index) = err_msg[start_index..].find(expected_text) {
                    start_index += index + expected_text.len();
                } else {
                    panic!(
                        "Expected error message to contain in order:\n{:?}\nBut got:\n{}",
                        expected_texts, err_msg
                    );
                }
            }
        }
        _ => panic!(
            "Expected panic with error message containing in order:\n{:?}",
            expected_texts
        ),
    }
}

pub fn expect_fails_with2<F, V, S>(expected_texts: V, f: F)
where
    F: FnOnce() + panic::UnwindSafe,
    V: Into<Vec<S>>,
    S: ToString,
{
    // Convert expected texts into a Vec<String>
    let expected_texts: Vec<String> = expected_texts
        .into()
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    // Suppress panic output for this invocation
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    // Catch panic and unwind safely
    let result = panic::catch_unwind(AssertUnwindSafe(f));

    // Restore the default panic hook
    panic::set_hook(default_hook);

    match result {
        Err(err) => {
            // Extract the error message from the panic
            let err_msg: &str = if let Some(err_msg) = err.downcast_ref::<String>() {
                err_msg
            } else if let Some(err_msg) = err.downcast_ref::<&str>() {
                err_msg
            } else {
                panic!(
                    "Expected error message containing:\n{:?}\nBut got a different type of panic.",
                    expected_texts
                );
            };

            // Check that all expected texts appear in order in the error message
            let mut start_index = 0;
            for expected_text in &expected_texts {
                if let Some(index) = err_msg[start_index..].find(expected_text) {
                    start_index += index + expected_text.len();
                } else {
                    panic!(
                        "Expected error message to contain in order:\n{:?}\nBut got:\n{}",
                        expected_texts, err_msg
                    );
                }
            }
        }
        Ok(_) => panic!(
            "Expected panic with error message containing in order:\n{:?}",
            expected_texts
        ),
    }
}

enum SubstringPart {
    Prefix,
    Mid,
    Suffix,
}

fn substring_of<S: ToString>(s: S, part: SubstringPart) -> String {
    let s = s.to_string();

    let len = s.len();
    let half_length = len / 2;

    match part {
        SubstringPart::Prefix => s[..half_length].to_string(),
        SubstringPart::Mid => {
            let start = (len - half_length) / 2;
            s[start..start + half_length].to_string()
        }
        SubstringPart::Suffix => s[len - half_length..].to_string(),
    }
}

fn inverse_char(c: char) -> char {
    match c {
        'a'..='z' => (b'z' - (c as u8 - b'a')) as char,
        'A'..='Z' => (b'Z' - (c as u8 - b'A')) as char,
        '0'..='9' => (b'9' - (c as u8 - b'0')) as char,
        _ => c,
    }
}

fn string_inverse<S: ToString>(s: S) -> String {
    s.to_string().chars().map(inverse_char).collect()
}

#[derive(Debug)]
pub struct MultiValueMatcherData<ExpectedValue, K, V, M>
where
    K: Into<String>,
    V: Into<String>,
    M: Into<String>,
{
    scenario_name: String,
    expect: ExpectedValue,
    actual: Vec<(K, V)>,
    failure_msg: Option<Vec<M>>,
}

#[derive(Debug)]
pub struct MultiValueMatcherTestSet<K, V, C, M>
where
    K: Into<String>,
    V: Into<String>,
    C: TryInto<usize>,
    M: Into<String>,
{
    attribute: Vec<MultiValueMatcherData<(K, V), K, V, M>>,
    attribute_not: Vec<MultiValueMatcherData<(K, V), K, V, M>>,
    attribute_exists: Vec<MultiValueMatcherData<K, K, V, M>>,
    attribute_missing: Vec<MultiValueMatcherData<K, K, V, M>>,
    attribute_includes: Vec<MultiValueMatcherData<(K, V), K, V, M>>,
    attribute_excludes: Vec<MultiValueMatcherData<(K, V), K, V, M>>,
    attribute_prefix: Vec<MultiValueMatcherData<(K, V), K, V, M>>,
    attribute_suffix: Vec<MultiValueMatcherData<(K, V), K, V, M>>,
    attribute_prefix_not: Vec<MultiValueMatcherData<(K, V), K, V, M>>,
    attribute_suffix_not: Vec<MultiValueMatcherData<(K, V), K, V, M>>,
    attribute_matches: Vec<MultiValueMatcherData<(K, V), K, V, M>>,
    attribute_count: Vec<MultiValueMatcherData<(K, V, C), K, V, M>>,
}

impl MultiValueMatcherTestSet<&'static str, &'static str, usize, &'static str> {
    pub fn generate(
        entity: &'static str,
        mismatch_header: &'static str,
        case_sensitive: bool,
    ) -> Self {
        MultiValueMatcherTestSet {
            attribute: vec![
                MultiValueMatcherData {
                    scenario_name: format!("{entity} where 'word' equals 'hello'"),
                    expect: ("word", "hello"),
                    actual: vec![("lang", "en"), ("word", "hello"), ("short", "hi")],
                    failure_msg: None,
                },
                MultiValueMatcherData {
                    scenario_name: format!("{entity} with multiple keys in the request not matching"),
                    expect: ("word", "hello world"),
                    actual: vec![
                        ("lang", "en"),
                        ("weird", "hello world"),
                        ("short", "hi"),
                        ("word", "hallo welt"),
                    ],
                    failure_msg: Some(vec![
                        mismatch_header,
                        "key",
                        "equals",
                        "word",
                        "value",
                        "equals",
                        "hello world",
                        entity,
                    ]),
                }],
            attribute_not: vec![
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_not where 'word' does not equal 'hello'"),
                    expect: ("word", "hello"),
                    actual: vec![("word", "hallo")],
                    failure_msg: None,
                },
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_not where 'word' is empty"),
                    expect: ("word", "hello"),
                    actual: vec![("word", "")],
                    failure_msg: None,
                },
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_not where 'word' does not exactly equal 'hello'"),
                    expect: ("word", "hello"),
                    actual: vec![("word", "hello world")],
                    failure_msg: None,
                },
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_not with correct value but missing key"),
                    expect: ("hello", "world"),
                    actual: vec![("wrong_key", "world")],
                    failure_msg: Some(vec![
                        mismatch_header,
                        "Expected",
                        "key",
                        "equals",
                        "hello",
                        "value",
                        "not equal to",
                        "world",
                        "Received",
                        "wrong_key=world",
                        entity,
                        "_not",
                    ]),
                },
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_not with one key non-matching key"),
                    expect: ("hello", "world"),
                    actual: vec![("hello", "world")],
                    failure_msg: Some(vec![
                        mismatch_header,
                        "Expected",
                        "key",
                        "equals",
                        "hello",
                        "value",
                        "not equal to",
                        "world",
                        "Received",
                        "hello=world",
                        entity,
                        "_not",
                    ]),
                },
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_not where 'word' key should not match 'hello' but is not present"),
                    expect: ("word", "hello"),
                    actual: vec![("not_word", "hello world")],
                    failure_msg: Some(vec![
                        mismatch_header,
                        "Expected", "key", "equals", "word",
                        "value", "not equal to", "hello",
                        "Received", "not_word=hello world",
                        entity,
                        "_not",
                    ]),
                },
            ],
            attribute_exists: vec![
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_exists where 'word' is present with value"),
                    expect: "word",
                    actual: vec![("word", "hello")],
                    failure_msg: None,
                },
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_exists where 'word' is present without value"),
                    expect: "word",
                    actual: vec![("word", "")],
                    failure_msg: None,
                },
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_exists where parameter should be present but is missing"),
                    expect: "word",
                    actual: vec![("wald", "word"), ("world", "hello")],
                    failure_msg: Some(vec![
                        mismatch_header,
                        "Expected", "key", "equals", "word",
                        "to be in the request, but none was provided",
                        entity,
                        "_exists",
                    ]),
                },
            ],
            attribute_missing: vec![
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_missing where 'word' is absent"),
                    expect: "word",
                    actual: vec![("something", "different")],
                    failure_msg: None,
                },
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_missing where parameter 'word' should not be present but is found"),
                    expect: "word",
                    actual: vec![("welt", "different"), ("word", "")],  // Capturing 'word' as empty
                    failure_msg: Some(vec![
                        mismatch_header,
                        "Expected", "key", "not equal to", "word",
                        "not to be present, but the request contained it",
                        entity,
                        "_missing",
                    ]),
                },
            ],
            attribute_includes: vec![
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_includes where 'word' includes 'ello'"),
                    expect: ("word", "ello"),
                    actual: vec![("word", "hello")],
                    failure_msg: None,
                },
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_includes where 'word' value should include 'ello'"),
                    expect: ("word", "ello"),
                    actual: vec![("word", "world")],  // Actual value that fails to meet the expectation
                    failure_msg: Some(vec![
                        mismatch_header,
                        "Expected", "key", "equals", "word",
                        "value", "includes", "ello",
                        "Received", "word=world",
                        entity,
                        "_includes",
                    ]),
                },
            ],
            attribute_excludes: vec![
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_excludes where 'word' excludes 'ello'"),
                    expect: ("word", "ello"),
                    actual: vec![("word", "hallo")],
                    failure_msg: None,
                },
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_excludes where 'word' value should exclude 'ello'"),
                    expect: ("word", "ello"),
                    actual: vec![("word", "hello")],
                    failure_msg: Some(vec![
                        mismatch_header,
                        "Expected", "key", "equals", "word",
                        "value", "excludes", "ello",
                        "Received", "word=hello",
                        entity,
                        "_excludes",
                    ]),
                },
            ],
            attribute_prefix: vec![
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_prefix where 'word' starts with 'ha'"),
                    expect: ("word", "ha"),
                    actual: vec![("word", "hallo")],
                    failure_msg: None,
                },
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_prefix where 'word' value should start with 'ha'"),
                    expect: ("word", "ha"),
                    actual: vec![("word", "hello")],  // Actual value that correctly matches the prefix condition
                    failure_msg: Some(vec![
                        mismatch_header,
                        "Expected", "key", "equals", "word",
                        "value", "prefix", "ha",
                        "Received", "word=hello",
                        entity,
                        "_prefix",
                    ]),
                },
            ],
            attribute_suffix: vec![
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_suffix where 'word' ends with 'llo'"),
                    expect: ("word", "llo"),
                    actual: vec![("word", "hello")],
                    failure_msg: None,
                },
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_suffix where 'word' value should end with 'llo'"),
                    expect: ("word", "llo"),
                    actual: vec![("word", "world")],  // Actual value that fails to meet the suffix condition
                    failure_msg: Some(vec![
                        mismatch_header,
                        "Expected", "key", "equals", "word",
                        "value", "suffix", "llo",
                        "Received", "word=world",
                        entity,
                        "_suffix",
                    ]),
                },
            ],
            attribute_prefix_not: vec![
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_prefix_not where 'word' does not start with 'ha'"),
                    expect: ("word", "ha"),
                    actual: vec![("word", "hello")],
                    failure_msg: None,
                },
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_prefix_not where 'word' value should not start with 'ha'"),
                    expect: ("word", "ha"),
                    actual: vec![("word", "hallo")],  // Actual value that incorrectly matches the prefix condition
                    failure_msg: Some(vec![
                        mismatch_header,
                        "Expected", "key", "equals", "word",
                        "value", "prefix not", "ha",
                        "Received", "word=hallo",
                        entity,
                        "_prefix_not",
                    ]),
                },
            ],
            attribute_suffix_not: vec![
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_suffix_not where 'word' does not end with 'll'"),
                    expect: ("word", "ll"),
                    actual: vec![("word", "hallo")],
                    failure_msg: None,
                },
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_suffix_not where 'word' value should not end with 'ld'"),
                    expect: ("word", "ld"),
                    actual: vec![("word", "world")],  // Actual value that incorrectly matches the suffix condition
                    failure_msg: Some(vec![
                        mismatch_header,
                        "Expected", "key", "equals", "word",
                        "value", "suffix not", "ld",
                        "Received", "word=world",
                        entity,
                        "_suffix_not",
                    ]),
                },
            ],
            attribute_matches: vec![
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_matches where key matches '.*ll.*' and value matches '.*or.*'"),
                    expect: (".*ll.*", ".*or.*"),
                    actual: vec![("hello", "world")],
                    failure_msg: None,
                },
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_matches where key and value should match regex patterns"),
                    expect: (".*ll.*", ".*or.*"),
                    actual: vec![("hello", "peter")],  // Actual key-value that fails to match the expected regex patterns fully
                    failure_msg: Some(vec![
                        mismatch_header,
                        "Expected", "key", "matches regex", ".*ll.*",
                        "value", "matches regex", ".*or.*",
                        "Received", "hello=peter",
                        entity,
                        "_matches",
                    ]),
                },
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_matches where key and value should match regex patterns again"),
                    expect: (".*ll.*", ".*or.*"),
                    actual: vec![("peter", "world")],  // Actual key-value that fails both expected regex conditions
                    failure_msg: Some(vec![
                        mismatch_header,
                        "Expected", "key", "matches regex", ".*ll.*",
                        "value", "matches regex", ".*or.*",
                        "Received", "peter=world",
                        entity,
                        "_matches",
                    ]),
                },
            ],
            attribute_count: vec![
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_count where key matches '.*el.*' and value matches '.*al.*' appears 2 times"),
                    expect: (".*el.*", ".*al.*", 2),
                    actual: vec![("hello", "peter"), ("hello", "wallie"), ("nothing", ""), ("hello", ""), ("hello", "metallica")],
                    failure_msg: None,
                },
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_count where key matches '.*el.*' and value matches '.*al.*' appears 2 times"),
                    expect: (".*el.*", ".*al.*", 2),
                    actual: vec![("hello", "peter"), ("hello", "wallie"), ("nothing", ""), ("hello", ""), ("hello", "metallica")],
                    failure_msg: None,
                },
                MultiValueMatcherData {
                    scenario_name: format!("{entity}_count where parameters should match key and value regex and appear a specified number of times"),
                    expect: (".*ll.*", ".*", 10),
                    actual: vec![("hello", "peter"), ("hello", "wallie"), ("nothing", ""), ("hello", ""), ("hello", "metallica")],
                    failure_msg: Some(vec![
                        mismatch_header,
                        "Expected", "key", "matches regex", ".*ll.*",
                        "value", "matches regex", ".*",
                        "to appear 10 times but appeared 4",
                        entity,
                        "_count",
                    ]),
                },
            ],
        }
    }
}

#[derive(Debug)]
pub struct SingleValueMatcherData<ExpectedValue, V, M>
where
    V: Into<String>,
    M: Into<String>,
{
    scenario_name: String,
    expect: ExpectedValue,
    actual: V,
    failure_msg: Option<Vec<M>>,
}

#[derive(Debug)]
pub struct SingleValueMatcherDataSet<V, M>
where
    V: Into<String>,
    M: Into<String>,
{
    attribute: Vec<SingleValueMatcherData<V, V, M>>,
    attribute_not: Vec<SingleValueMatcherData<V, V, M>>,
    attribute_includes: Vec<SingleValueMatcherData<V, V, M>>,
    attribute_excludes: Vec<SingleValueMatcherData<V, V, M>>,
    attribute_prefix: Vec<SingleValueMatcherData<V, V, M>>,
    attribute_suffix: Vec<SingleValueMatcherData<V, V, M>>,
    attribute_prefix_not: Vec<SingleValueMatcherData<V, V, M>>,
    attribute_suffix_not: Vec<SingleValueMatcherData<V, V, M>>,
    attribute_matches: Vec<SingleValueMatcherData<V, V, M>>,
}

impl SingleValueMatcherDataSet<&'static str, &'static str> {
    pub fn generate(
        entity: &'static str,
        mismatch_header: &'static str,
        case_sensitive: bool,
    ) -> Self {
        SingleValueMatcherDataSet {
            attribute: vec![
                SingleValueMatcherData {
                    scenario_name: format!("{entity} TODO"),
                    expect: "test",
                    actual: "test",
                    failure_msg: None,
                },
                SingleValueMatcherData {
                    scenario_name: format!("{entity} TODO"),
                    expect: "test",
                    actual: "not-test",
                    failure_msg: Some(vec![
                        mismatch_header,
                        "Expected",
                        entity,
                        "equals",
                        "test",
                        "Received",
                        "not-test",
                    ]),
                },
            ],
            attribute_not: vec![
                SingleValueMatcherData {
                    scenario_name: format!("{entity}_not TODO"),
                    expect: "test",
                    actual: "twist",
                    failure_msg: None,
                },
                SingleValueMatcherData {
                    scenario_name: format!("{entity}_not TODO"),
                    expect: "test",
                    actual: "test",
                    failure_msg: Some(vec![
                        mismatch_header,
                        "Expected",
                        entity,
                        "not equal to",
                        "test",
                        "Received",
                        "test",
                    ]),
                },
            ],
            attribute_includes: vec![
                SingleValueMatcherData {
                    scenario_name: format!("{entity}_includes TODO"),
                    expect: "is-a",
                    actual: "this-is-a-value",
                    failure_msg: None,
                },
                SingleValueMatcherData {
                    scenario_name: format!("{entity}_includes TODO"),
                    expect: "dog",
                    actual: "tomato",
                    failure_msg: Some(vec![
                        mismatch_header,
                        "Expected",
                        entity,
                        "includes",
                        "dog",
                        "Received",
                        "tomato",
                    ]),
                },
            ],
            attribute_excludes: vec![
                SingleValueMatcherData {
                    scenario_name: format!("{entity}_excludes TODO"),
                    expect: "is-a",
                    actual: "this-is-the-value",
                    failure_msg: None,
                },
                SingleValueMatcherData {
                    scenario_name: format!("{entity}_excludes TODO"),
                    expect: "na",
                    actual: "banana",
                    failure_msg: Some(vec![
                        mismatch_header,
                        "Expected",
                        entity,
                        "excludes",
                        "na",
                        "Received",
                        "banana",
                    ]),
                },
            ],
            attribute_prefix: vec![
                SingleValueMatcherData {
                    scenario_name: format!("{entity}_prefix TODO"),
                    expect: "this",
                    actual: "this-is-the-value",
                    failure_msg: None,
                },
                SingleValueMatcherData {
                    scenario_name: format!("{entity}_prefix TODO"),
                    expect: "thi",
                    actual: "that",
                    failure_msg: Some(vec![
                        mismatch_header,
                        "Expected",
                        entity,
                        "has prefix",
                        "thi",
                        "Received",
                        "that",
                    ]),
                },
            ],
            attribute_suffix: vec![
                SingleValueMatcherData {
                    scenario_name: format!("{entity}_includes TODO"),
                    expect: "value",
                    actual: "this-is-the-value",
                    failure_msg: None,
                },
                SingleValueMatcherData {
                    scenario_name: format!("{entity}_includes TODO"),
                    expect: "bear",
                    actual: "banana",
                    failure_msg: Some(vec![
                        mismatch_header,
                        "Expected",
                        entity,
                        "suffix",
                        "bear",
                        "Received",
                        "banana",
                    ]),
                },
            ],
            attribute_prefix_not: vec![
                SingleValueMatcherData {
                    scenario_name: format!("{entity}_prefix_not TODO"),
                    expect: "value",
                    actual: "that-is-the-value",
                    failure_msg: None,
                },
                SingleValueMatcherData {
                    scenario_name: format!("{entity}_prefix_not TODO"),
                    expect: "this",
                    actual: "this-is-the-value",
                    failure_msg: Some(vec![
                        mismatch_header,
                        "Expected",
                        entity,
                        "prefix not",
                        "this",
                        "Received",
                        "this-is-the-value",
                    ]),
                },
            ],
            attribute_suffix_not: vec![
                SingleValueMatcherData {
                    scenario_name: format!("{entity}_suffix_not TODO"),
                    expect: "thing",
                    actual: "that-is-the-value",
                    failure_msg: None,
                },
                SingleValueMatcherData {
                    scenario_name: format!("{entity}_suffix_not TODO"),
                    expect: "to",
                    actual: "potato",
                    failure_msg: Some(vec![
                        mismatch_header,
                        "Expected",
                        entity,
                        "suffix not",
                        "to",
                        "Received",
                        "potato",
                    ]),
                },
            ],
            attribute_matches: vec![
                SingleValueMatcherData {
                    scenario_name: format!("{entity}_matches TODO"),
                    expect: ".*ll.*",
                    actual: "hello",
                    failure_msg: None,
                },
                SingleValueMatcherData {
                    scenario_name: format!("{entity}_matches TODO"),
                    expect: ".*is-the.*",
                    actual: "giggity",
                    failure_msg: Some(vec![
                        mismatch_header,
                        "Expected",
                        entity,
                        "matches",
                        ".*is-the.*",
                        "Received",
                        "giggity",
                    ]),
                },
            ],
        }
    }
}

fn to_urlencoded_query_string(params: Vec<(&str, &str)>) -> String {
    params
        .into_iter()
        .map(|(key, value)| {
            format!(
                "{}={}",
                urlencoding::encode(key),
                urlencoding::encode(value)
            )
        })
        .collect::<Vec<String>>()
        .join("&")
}
