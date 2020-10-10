extern crate httpmock;

use isahc::prelude::*;

use httpmock::MockServer;

#[test]
fn file_body_test() {
    // Arrange
    let server = MockServer::start();
    let m = server.mock(|when, then| {
        when.path("/hello");
        then.status(200)
            .body_from_file("tests/resources/simple_body.txt");
    });

    // Act
    let mut response = isahc::get(server.url("/hello")).unwrap();

    // Assert
    m.assert();
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().unwrap(), "ohi!");
}
