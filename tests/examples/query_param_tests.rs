extern crate httpmock;

use isahc::get;

use httpmock::MockServer;

#[test]
fn url_matching_test() {
    // Arrange
    let _ = env_logger::try_init();
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
