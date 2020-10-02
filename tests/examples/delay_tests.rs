extern crate httpmock;

use isahc::get;

use httpmock::{Mock, MockServer};
use httpmock_macros::httpmock_example_test;
use std::time::{Duration, SystemTime};

#[test]
#[httpmock_example_test] // Internal macro to make testing easier. Ignore it.
fn delay_test() {
    // Arrange
    let _ = env_logger::try_init();
    let start_time = SystemTime::now();
    let delay = Duration::from_secs(3);

    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.path("/delay");
        then.delay(delay);
    });

    // Act: Send the HTTP request
    let response = get(server.url("/delay")).unwrap();

    // Assert
    assert_eq!(response.status(), 200);
    assert_eq!(mock.hits(), 1);
    assert_eq!(start_time.elapsed().unwrap() > delay, true);
}
