use std::sync::Arc;

use crate::common::data::{MockDefinition, MockServerHttpResponse, Pattern, RequestRequirements};
use crate::common::util::read_file;
use crate::server::web::handlers::add_new_mock;
use crate::server::{start_server, MockServerState};
use crate::Method;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::fs::read_dir;
use std::path::PathBuf;
use std::str::FromStr;
use tokio::time::Duration;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct NameValuePair {
    name: String,
    value: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct YAMLRequestRequirements {
    pub path: Option<String>,
    pub path_contains: Option<Vec<String>>,
    pub path_matches: Option<Vec<String>>,
    pub method: Option<Method>,
    pub header: Option<Vec<NameValuePair>>,
    pub header_exists: Option<Vec<String>>,
    pub cookie: Option<Vec<NameValuePair>>,
    pub cookie_exists: Option<Vec<String>>,
    pub body: Option<String>,
    pub json_body: Option<Value>,
    pub json_body_partial: Option<Vec<Value>>,
    pub body_contains: Option<Vec<String>>,
    pub body_matches: Option<Vec<String>>,
    pub query_param_exists: Option<Vec<String>>,
    pub query_param: Option<Vec<NameValuePair>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct YAMLHTTPResponse {
    pub status: Option<u16>,
    pub header: Option<Vec<NameValuePair>>,
    pub body: Option<String>,
    pub delay: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct YAMLMockDefinition {
    when: YAMLRequestRequirements,
    then: YAMLHTTPResponse,
}

pub async fn start_standalone_server(
    port: u16,
    expose: bool,
    static_mock_dir_path: Option<PathBuf>,
    print_access_log: bool,
) -> Result<(), String> {
    let state = Arc::new(MockServerState::new());

    #[cfg(feature = "standalone")]
    static_mock_dir_path.map(|path| {
        read_static_mocks(path)
            .into_iter()
            .map(|d| map_to_mock_definition(d))
            .for_each(|static_mock| {
                add_new_mock(&state, static_mock, true).expect("cannot add static mock");
            })
    });

    start_server(port, expose, &state, None, print_access_log).await
}

#[cfg(feature = "standalone")]
fn read_static_mocks(path: PathBuf) -> Vec<YAMLMockDefinition> {
    let mut definitions = Vec::new();

    let paths = read_dir(path).expect("cannot list files in directory");
    for file_path in paths {
        let file_path = file_path.unwrap().path();
        if let Some(ext) = file_path.extension() {
            if !"yaml".eq(ext) && !"yml".eq(ext) {
                continue;
            }
        }

        log::info!(
            "Loading static mock file from '{}'",
            file_path.to_string_lossy()
        );
        let content = read_file(file_path).expect("cannot read from file");
        let content = String::from_utf8(content).expect("cannot convert file content");

        definitions.push(serde_yaml::from_str(&content).unwrap());
    }

    return definitions;
}

#[cfg(feature = "standalone")]
fn map_to_mock_definition(yaml_definition: YAMLMockDefinition) -> MockDefinition {
    MockDefinition {
        request: RequestRequirements {
            path: yaml_definition.when.path,
            path_contains: yaml_definition.when.path_contains,
            path_matches: to_pattern_vec(yaml_definition.when.path_matches),
            method: yaml_definition.when.method.map(|m| m.to_string()),
            headers: to_pair_vec(yaml_definition.when.header),
            header_exists: yaml_definition.when.header_exists,
            cookies: to_pair_vec(yaml_definition.when.cookie),
            cookie_exists: yaml_definition.when.cookie_exists,
            body: yaml_definition.when.body,
            json_body: yaml_definition.when.json_body,
            json_body_includes: yaml_definition.when.json_body_partial,
            body_contains: yaml_definition.when.body_contains,
            body_matches: to_pattern_vec(yaml_definition.when.body_matches),
            query_param_exists: yaml_definition.when.query_param_exists,
            query_param: to_pair_vec(yaml_definition.when.query_param),
            matchers: None,
        },
        response: MockServerHttpResponse {
            status: yaml_definition.then.status,
            headers: to_pair_vec(yaml_definition.then.header),
            body: yaml_definition.then.body.map(|b| b.into_bytes()),
            delay: yaml_definition.then.delay.map(|v| Duration::from_millis(v)),
        },
    }
}

#[cfg(feature = "standalone")]
fn to_pattern_vec(vec: Option<Vec<String>>) -> Option<Vec<Pattern>> {
    vec.map(|vec| {
        vec.iter()
            .map(|val| Pattern::from_regex(Regex::from_str(val).expect("cannot parse regex")))
            .collect()
    })
}

#[cfg(feature = "standalone")]
fn to_pair_vec(kvp: Option<Vec<NameValuePair>>) -> Option<Vec<(String, String)>> {
    kvp.map(|vec| vec.into_iter().map(|nvp| (nvp.name, nvp.value)).collect())
}
