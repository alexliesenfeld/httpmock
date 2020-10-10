extern crate httpmock;

use isahc::prelude::*;

use httpmock::MockServer;

use self::httpmock::Mock;

#[test]
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
    m.assert();
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().unwrap(), "ohi!");
}
