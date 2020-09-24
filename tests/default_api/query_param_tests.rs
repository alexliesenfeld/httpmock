extern crate httpmock;

use httpmock::MockServer;
use httpmock_macros::httpmock_example_test;
use isahc::get;

#[test]
#[httpmock_example_test] // Internal macro to make testing easier. Ignore it.
fn url_matching_test() {
    // Arrange
    let _ = env_logger::try_init();
    let mock_server = MockServer::start();

    let m = mock_server.mock(|when, then| {
        when.query_param("query", "Metallica")
            .query_param_exists("query");
        then.status(200);
    });

    // Act: Send the request and deserialize the response to JSON
    get(mock_server.url("/search?query=Metallica")).unwrap();

    // Assert
    assert_eq!(m.times_called(), 1);
}
