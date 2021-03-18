extern crate httpmock;

use isahc::get;

use httpmock::MockServer;

#[test]
fn url_matching_test() {
    // Arrange
    let server = MockServer::start();

    let m = server.mock(|when, then| {
        when.query_param("query", "Metallica")
            .query_param_exists("query");
        then.status(200);
    });

    // Act: Send the request and deserialize the response to JSON
    get(server.url("/search?query=Metallica")).unwrap();

    // Assert
    m.assert();
}

#[test]
fn urlencoded_params_test() {
    // Arrange
    let server = MockServer::start();

    let m = server.mock(|when, then| {
        when.method("GET")
            .path("/search")
            .query_param("query", "Motörhead");
        then.status(200);
    });

    // Act: Send the request and deserialize the response to JSON
    get(server.url("/search?query=Motörhead")).unwrap();

    // Assert
    m.assert();
}