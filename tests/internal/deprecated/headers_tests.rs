extern crate httpmock;

use isahc::prelude::*;

use httpmock::{Mock, MockServer};

#[test]
fn headers_test() {
    // Arrange
    let _ = env_logger::try_init();
    let server = MockServer::start();

    let m = Mock::new()
        .expect_path("/test")
        .expect_header("Authorization", "token 123456789")
        .expect_header_exists("Authorization")
        .return_status(201)
        .return_header("Content-Length", "0")
        .create_on(&server);

    // Act: Send the request and deserialize the response to JSON
    let response = Request::post(&format!("http://{}/test", server.address()))
        .header("Authorization", "token 123456789")
        .body(())
        .unwrap()
        .send()
        .unwrap();

    // Assert
    m.assert();
    assert_eq!(response.status(), 201);
}
