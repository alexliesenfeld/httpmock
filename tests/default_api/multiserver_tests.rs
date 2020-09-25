extern crate httpmock;

use isahc::prelude::*;
use isahc::HttpClientBuilder;

use httpmock::{MockServer};
use httpmock_macros::httpmock_example_test;
use isahc::config::RedirectPolicy;

#[test]
#[httpmock_example_test] // Internal macro to make testing easier. Ignore it.
fn multiserver_test() {
    // Arrange
    let _ = env_logger::try_init();
    let mock_server1 = MockServer::start();
    let mock_server2 = MockServer::start();

    let redirect_mock = mock_server1.mock(|when, then| {
        when.path("/redirectTest");
        then.temporary_redirect(&mock_server2.url("/finalTarget"));
    });

    let target_mock = mock_server2.mock(|when, then| {
        when.path("/finalTarget");
        then.status(200);
    });

    // Act: Send the HTTP request with an HTTP client that automatically follows redirects!
    let http_client = HttpClientBuilder::new()
        .redirect_policy(RedirectPolicy::Follow)
        .build()
        .unwrap();

    let response = http_client.get(mock_server1.url("/redirectTest")).unwrap();

    // Assert
    assert_eq!(response.status(), 200);
    assert_eq!(redirect_mock.times_called(), 1);
    assert_eq!(target_mock.times_called(), 1);
}
