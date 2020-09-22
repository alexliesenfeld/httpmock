extern crate httpmock;

use isahc::prelude::*;

use httpmock::{MockServer};
use httpmock_macros::httpmock_example_test;

#[test]
#[httpmock_example_test] // Internal macro to make testing easier. Ignore it.
fn headers_test() {
    // Arrange
    let _ = env_logger::try_init();
    let mock_server = MockServer::start();

    let m = mock_server.mock(|when, then| {
        when.path("/test")
            .header("Authorization", "token 123456789")
            .header_exists("Authorization");
        then.status(201).header("Content-Length", "0");
    });

    // Act: Send the request and deserialize the response to JSON
    let response = Request::post(&format!("http://{}/test", mock_server.address()))
        .header("Authorization", "token 123456789")
        .body(())
        .unwrap()
        .send()
        .unwrap();

    // Assert
    assert_eq!(response.status(), 201);
    assert_eq!(m.times_called(), 1);
}
