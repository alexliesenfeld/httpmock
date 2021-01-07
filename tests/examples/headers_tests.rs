extern crate httpmock;

use isahc::{prelude::*, Request};

use httpmock::MockServer;

#[test]
fn headers_test() {
    // Arrange
    let _ = env_logger::try_init();
    let server = MockServer::start();

    let m = server.mock(|when, then| {
        when.path("/test")
            .header("Authorization", "token 123456789")
            .header_exists("Authorization");
        then.status(201).header("Content-Length", "0");
    });

    // Act: Send the request and deserialize the response to JSON
    let response = Request::post(&format!("http://{}/test", server.address()))
        .header("Authorization", "token 123456789")
        .body(())
        .unwrap()
        .send()
        .unwrap();

    // Assert
    m.assert();
    //assert_eq!(response.status(), 201);
}
