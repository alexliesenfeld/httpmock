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

    let redirect_mock = Mock::new()
        .expect_path("/redirectTest")
        .return_status(302)
        .return_header(
            "Location",
            &format!("http://{}/finalTarget", mock_server2.address()),
        )
        .create_on(&mock_server1);

    let target_mock = Mock::new()
        .expect_path("/finalTarget")
        .return_status(200)
        .create_on(&mock_server2);

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
