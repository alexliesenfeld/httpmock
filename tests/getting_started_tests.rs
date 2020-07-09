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
fn simple_test() {
    // Arrange
    let _ = env_logger::try_init();
    let mock_server = MockServer::start();

    let search_mock = Mock::new()
        .expect_path_contains("/search")
        .expect_query_param("query", "metallica")
        .return_status(202)
        .create_on(&mock_server);

    // Act: Send the HTTP request
    let response = get(&format!(
        "http://{}/search?query=metallica",
        mock_server.address()
    ))
    .unwrap();

    // Assert
    assert_eq!(response.status(), 202);
    assert_eq!(search_mock.times_called(), 1);
}

/// Demonstrates how to use async structures
#[async_std::test]
async fn simple_test_async() {
    // Arrange
    let _ = env_logger::try_init();
    let mock_server = MockServer::start_async().await;

    let search_mock = Mock::new()
        .expect_path_contains("/search")
        .return_status(202)
        .create_on_async(&mock_server)
        .await;

    // Act: Send the HTTP request

    // mock_server.url will create a full URL for the path on the server. In this case it will be
    // http://localhost:<port>/search
    let url = mock_server.url("/search");
    let response = get_async(url).await.unwrap();

    // Assert
    assert_eq!(response.status(), 202);
    assert_eq!(search_mock.times_called_async().await, 1);
}
