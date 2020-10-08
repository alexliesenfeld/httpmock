extern crate httpmock;

use self::httpmock::Mock;
use httpmock::MockServer;
use httpmock_macros::httpmock_example_test;
use isahc::prelude::*;

#[test]
#[httpmock_example_test] // Internal macro to make testing easier. Ignore it.
fn file_body_test() {
    // Arrange
    let server = MockServer::start();
    let m = Mock::new()
        .expect_path("/hello")
        .return_status(200)
        .return_body_from_file("tests/resources/simple_body.txt")
        .create_on(&server);

    // Act
    let mut response = isahc::get(server.url("/hello")).unwrap();

    // Assert
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().unwrap(), "ohi!");
    assert_eq!(m.hits(), 1);
}
