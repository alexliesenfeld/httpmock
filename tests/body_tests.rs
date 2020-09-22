extern crate httpmock;

use httpmock::Method::{POST};
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
fn body_test() {
    // Arrange
    let _ = env_logger::try_init();
    let mock_server = MockServer::start();

    let m = Mock::new()
        .expect_method(POST)
        .expect_path("/books")
        .expect_body("The Fellowship of the Ring")
        .expect_body_contains("Ring")
        .expect_body_matches(Regex::new("Fellowship").unwrap())
        .return_status(201)
        .return_body("The Lord of the Rings")
        .create_on(&mock_server);

    // Act: Send the request and deserialize the response to JSON
    let mut response = Request::post(&format!("http://{}/books", mock_server.address()))
        .body("The Fellowship of the Ring")
        .unwrap()
        .send()
        .unwrap();

    // Assert
    assert_eq!(response.status(), 201);
    assert_eq!(m.times_called(), 1);
}
