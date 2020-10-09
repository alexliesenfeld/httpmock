extern crate httpmock;

use isahc::prelude::*;
use isahc::HttpClientBuilder;

use httpmock::{Mock, MockServer};
use httpmock_macros::httpmock_example_test;
use isahc::config::RedirectPolicy;

#[test]
#[httpmock_example_test] // Internal macro to make testing easier. Ignore it.
fn multiserver_test() {
    // Arrange
    let _ = env_logger::try_init();
    let server1 = MockServer::start();
    let server2 = MockServer::start();

    let redirect_mock = Mock::new()
        .expect_path("/redirectTest")
        .return_temporary_redirect(&server2.url("/finalTarget"))
        .create_on(&server1);

    let target_mock = Mock::new()
        .expect_path("/finalTarget")
        .return_status(200)
        .create_on(&server2);

    // Act: Send the HTTP request with an HTTP client that automatically follows redirects!
    let http_client = HttpClientBuilder::new()
        .redirect_policy(RedirectPolicy::Follow)
        .build()
        .unwrap();

    let response = http_client.get(server1.url("/redirectTest")).unwrap();

    // Assert
    assert_eq!(response.status(), 200);
    assert_eq!(redirect_mock.hits(), 1);
    assert_eq!(target_mock.hits(), 1);
}
