use httpmock::prelude::*;
use regex::Regex;
use reqwest::blocking::Client;

#[test]
fn body_test() {
    // Arrange
    let server = MockServer::start();

    let m = server.mock(|when, then| {
        when.method(POST)
            .path("/books")
            .body("The Fellowship of the Ring")
            .body_includes("Ring")
            .body_matches(Regex::new("Fellowship").unwrap());
        then.status(201).body("The Lord of the Rings");
    });

    // Act: Send the request
    let client = Client::new();
    let response = client
        .post(format!("http://{}/books", server.address()))
        .body("The Fellowship of the Ring")
        .send()
        .unwrap();

    // Assert
    m.assert();
    assert_eq!(response.status(), 201);
}
