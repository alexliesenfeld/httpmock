extern crate httpmock;

use isahc::get;

use httpmock::{Mock, MockServer};

#[test]
fn url_matching_test() {
    // Arrange
    let _ = env_logger::try_init();
    let server = MockServer::start();

    let m = Mock::new()
        .expect_query_param("query", "Metallica")
        .expect_query_param_exists("query")
        .return_status(200)
        .create_on(&server);

    // Act: Send the request and deserialize the response to JSON
    get(server.url("/search?query=Metallica")).unwrap();

    // Assert
    m.assert();
}
