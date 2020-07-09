extern crate httpmock;

use isahc::prelude::*;
use isahc::{get, get_async, HttpClientBuilder};

use httpmock::Method::{GET, POST};
use httpmock::{Mock, MockServer, MockServerRequest, Regex};
use httpmock_macros::test_executors;
use isahc::config::RedirectPolicy;
use std::fs::read_to_string;
use std::time::{Duration, SystemTime};

/// This test asserts that mocks can be stored, served and deleted as designed.
#[test]
#[test_executors] // Internal macro that executes this test in different async executors. Ignore it.
fn example_test() {
    // Start a local mock server for exclusive use by this test function.
    let mock_server = MockServer::start();

    // Create a mock on the mock server. The mock will return HTTP status code 200 whenever
    // the mock server receives a GET-request with path "/hello".
    let search_mock = Mock::new()
        .expect_method(GET)
        .expect_path("/hello")
        .return_status(200)
        .create_on(&mock_server);

    // Send an HTTP request to the mock server. This simulates your code.
    // The mock_server variable tis being used to generate a mock server URL for path "/hello".
    let response = get(mock_server.url("/hello")).unwrap();

    // Ensure the mock server did respond as specified above.
    assert_eq!(response.status(), 200);
    // Ensure the specified mock responded exactly one time.
    assert_eq!(search_mock.times_called(), 1);
}

/// Demonstrates how to use async structures
#[async_std::test]
async fn simple_test_async() {
    // Start a local mock server for exclusive use by this test function.
    let mock_server = MockServer::start_async().await;

    // Create a mock on the mock server. The mock will return HTTP status code 200 whenever
    // the mock server receives a GET-request with path "/hello".
    let search_mock = Mock::new()
        .expect_method(GET)
        .expect_path("/hello")
        .return_status(200)
        .create_on_async(&mock_server)
        .await;

    // Send an HTTP request to the mock server. This simulates your code.
    let url = format!("http://{}/hello", mock_server.address());
    let response = get_async(&url).await.unwrap();

    // Ensure the mock server did respond as specified above.
    assert_eq!(response.status(), 200);
    // Ensure the specified mock responded exactly one time.
    assert_eq!(search_mock.times_called_async().await, 1);
}
