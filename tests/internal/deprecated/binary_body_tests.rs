extern crate httpmock;

use self::httpmock::Mock;
use httpmock::MockServer;
use httpmock_macros::httpmock_example_test;
use isahc::prelude::*;
use std::io::Read;

#[test]
#[httpmock_example_test] // Internal macro to make testing easier. Ignore it.
fn binary_body_test() {
    let _ = env_logger::try_init();

    // Arrange
    let binary_content = b"\x80\x02\x03";

    let mock_server = MockServer::start();

    let m = Mock::new()
        .expect_path("/hello")
        .return_status(200)
        .return_body(binary_content)
        .create_on(&mock_server);

    // Act
    let mut response = isahc::get(mock_server.url("/hello")).unwrap();

    // Assert
    assert_eq!(response.status(), 200);
    assert_eq!(m.hits(), 1);
    assert_eq!(body_to_vec(response.body_mut()), binary_content.to_vec());
}

fn body_to_vec(body: &mut Body) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    body.read_to_end(&mut buf).expect("Cannot read from body");
    buf
}
