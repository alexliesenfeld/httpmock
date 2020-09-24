extern crate httpmock;

use isahc::get;

use httpmock::Method::GET;
use httpmock::MockServer;
use httpmock_macros::httpmock_example_test;

#[test]
#[httpmock_example_test] // Internal macro that executes this test in different async executors. Ignore it.
fn explicit_delete_mock_test() {
    // Arrange
    let _ = env_logger::try_init();
    let mock_server = MockServer::start();

    let mut m = mock_server.mock(|when, then| {
        when.method(GET).path("/health");
        then.status(205);
    });

    // Act: Send the HTTP request
    let response = get(&format!(
        "http://{}:{}/health",
        mock_server.host(),
        mock_server.port()
    ))
    .unwrap();

    // Assert
    assert_eq!(response.status(), 205);
    assert_eq!(m.times_called(), 1);

    // Delete the mock and send the request again
    m.delete();

    let response = get(&format!("http://{}/health", mock_server.address())).unwrap();

    // Assert that the request failed, because the mock has been deleted
    assert_eq!(response.status(), 404);
}