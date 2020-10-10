extern crate httpmock;

use isahc::get;

use httpmock::Method::GET;
use httpmock::{Mock, MockServer};

#[test]
fn explicit_delete_mock_test() {
    // Arrange
    let _ = env_logger::try_init();
    let server = MockServer::start();

    let mut m = Mock::new()
        .expect_method(GET)
        .expect_path("/health")
        .return_status(205)
        .create_on(&server);

    // Act: Send the HTTP request
    let response = get(&format!(
        "http://{}:{}/health",
        server.host(),
        server.port()
    ))
    .unwrap();

    // Assert
    m.assert();
    assert_eq!(response.status(), 205);

    // Delete the mock and send the request again
    m.delete();

    let response = get(&format!("http://{}/health", server.address())).unwrap();

    // Assert that the request failed, because the mock has been deleted
    assert_eq!(response.status(), 404);
}
