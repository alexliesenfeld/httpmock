use httpmock::prelude::*;
use isahc::{prelude::*, Request};

#[test]
fn body_test() {
    // Arrange
    let server = MockServer::start();

    let m = server.mock(|when, then| {
        when.method(POST)
            .path("/books")
            .body("The Fellowship of the Ring")
            .body_contains("Ring")
            .body_matches(Regex::new("Fellowship").unwrap());
        then.status(201).body("The Lord of the Rings");
    });

    // Act: Send the request and deserialize the response to JSON
    let response = Request::post(&format!("http://{}/books", server.address()))
        .body("The Fellowship of the Ring")
        .unwrap()
        .send()
        .unwrap();

    // Assert
    m.assert();
    assert_eq!(response.status(), 201);
}
