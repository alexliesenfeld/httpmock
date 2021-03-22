extern crate httpmock;

use isahc::{Request, RequestExt};

use httpmock::MockServer;

use crate::simulate_standalone_server;

#[test]
fn large_body_test() {
    // Arrange

    // This starts up a standalone server in the background running on port 5000
    simulate_standalone_server();

    // Instead of creating a new MockServer using new(), we connect to an existing remote instance.
    let server = MockServer::connect("localhost:5000");

    let search_mock = server.mock(|when, then| {
        when.path("/search").body("wow so large".repeat(10000000)); // 120 MB body
        then.status(202);
    });

    // Act: Send the HTTP request
    let response = Request::post(server.url("/search"))
        .body("wow so large".repeat(10000000)) // 120 MB body
        .unwrap()
        .send()
        .unwrap();

    // Assert
    search_mock.assert();
    assert_eq!(response.status(), 202);
}
