extern crate httpmock;

use isahc::prelude::*;
use isahc::HttpClientBuilder;

use httpmock::{Mock, MockServer};
use httpmock_macros::httpmock_example_test;
use isahc::config::RedirectPolicy;

#[test]
#[httpmock_example_test] // Internal macro to make testing easier. Ignore it.
fn multiple_mock_servers_redirect_test() {
    // Arrange
    let _ = env_logger::try_init();
    let mock_server1 = MockServer::start();
    let mock_server2 = MockServer::start();

    let redirect_mock = mock_server1.mock(|| {
        when.path("/redirectTest");
        then.status(302).header(
            "Location",
            &format!("http://{}/finalTarget", mock_server2.address()),
        );
    });

    let target_mock = mock_server2.mock(|when, then| {
        when.path("/finalTarget");
        then.return_status(200);
    });

    // Act: Send the HTTP request
    let http_client = HttpClientBuilder::new()
        .redirect_policy(RedirectPolicy::Follow)
        .build()
        .unwrap();

    let response = http_client
        .get(&format!("http://{}/redirectTest", mock_server1.address()))
        .unwrap();

    // Assert
    assert_eq!(response.status(), 200);
    assert_eq!(redirect_mock.times_called(), 1);
    assert_eq!(target_mock.times_called(), 1);
}
