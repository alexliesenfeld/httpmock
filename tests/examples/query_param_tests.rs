extern crate httpmock;

use isahc::get as http_get;
use ureq::get as httpget;

use httpmock::MockServer;

#[test]
fn url_param_matching_test() {
    // Arrange
    let server = MockServer::start();

    let m = server.mock(|when, then| {
        when.query_param("query", "Metallica")
            .query_param_exists("query");
        then.status(200);
    });

    // Act: Send the request and deserialize the response to JSON
    http_get(server.url("/search?query=Metallica")).unwrap();

    // Assert
    m.assert();
}

#[test]
fn url_param_urlencoded_matching_test() {
    // Arrange
    let server = MockServer::start();

    let m = server.mock(|when, then| {
        when.query_param("query", "Motörhead")
            .query_param_exists("query");
        then.status(200);
    });

    // Act: Send the request
    http_get(server.url("/search?query=Mot%C3%B6rhead")).unwrap();

    // Assert
    m.assert();
}

#[test]
fn url_param_unencoded_matching_test() {
    // Arrange
    let server = MockServer::start();

    let m = server.mock(|when, then| {
        when.query_param("query", "Motörhead")
            .query_param_exists("query");
        then.status(200);
    });

    // Act: Send the request
    httpget(&server.url("/search?query=Motörhead"))
        .send_string("")
        .unwrap();

    // Assert
    m.assert();
}
