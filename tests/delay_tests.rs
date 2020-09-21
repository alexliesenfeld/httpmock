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
#[httpmock_example_test] // Internal macro to make testing easier. Ignore it.
fn delay_test() {
    // Arrange
    let _ = env_logger::try_init();
    let start_time = SystemTime::now();
    let delay = Duration::from_secs(3);

    let mock_server = MockServer::start();

    let search_mock = Mock::new()
        .expect_path("/delay")
        .return_delay(delay)
        .create_on(&mock_server);

    // Act: Send the HTTP request
    let response = get(mock_server.url("/delay")).unwrap();

    // Assert
    assert_eq!(response.status(), 200);
    assert_eq!(search_mock.times_called(), 1);
    assert_eq!(start_time.elapsed().unwrap() > delay, true);
}
