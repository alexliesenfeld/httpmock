extern crate httpmock;

use isahc::get;

use httpmock::{Mock, MockServer};
use httpmock_macros::httpmock_example_test;
use std::time::{SystemTime, Duration};

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
        .return_with_delay(delay)
        .create_on(&mock_server);

    // Act: Send the HTTP request
    let response = get(mock_server.url("/delay")).unwrap();

    // Assert
    assert_eq!(response.status(), 200);
    assert_eq!(search_mock.times_called(), 1);
    assert_eq!(start_time.elapsed().unwrap() > delay, true);
}
