extern crate httpmock;

use httpmock::Method::{GET, POST};
use httpmock::{Mock, MockServer, MockServerRequest, Regex};
use httpmock_macros::httpmock_example_test;
use isahc::config::RedirectPolicy;
use isahc::prelude::*;
use isahc::{get, get_async, HttpClientBuilder};
use serde_json::{json, Value};
use std::fs::read_to_string;
use std::time::{Duration, SystemTime};

/// Tests and demonstrates body matching.
#[test]
#[httpmock_example_test] // Internal macro to make testing easier. Ignore it.
fn url_matching_test() {
    // Arrange
    let _ = env_logger::try_init();
    let mock_server = MockServer::start();

    let m = Mock::new()
        .expect_path("/appointments/20200922")
        .expect_path_contains("appointments")
        .expect_path_matches(Regex::new(r"\d{4}\d{2}\d{2}$").unwrap())
        .return_status(201)
        .create_on(&mock_server);

    // Act: Send the request and deserialize the response to JSON
    get(mock_server.url("/appointments/20200922")).unwrap();

    // Assert
    assert_eq!(m.times_called(), 1);
}
