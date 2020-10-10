extern crate httpmock;

use std::io::Read;

use isahc::{get, get_async, Body, RequestExt};
use regex::Replacer;

use httpmock::MockServer;

use crate::simulate_standalone_server;

#[test]
fn large_body_test() {
    // This starts up a standalone server in the background running on port 5000
    simulate_standalone_server();

    // Arrange
    let _ = env_logger::try_init();

    // Instead of creating a new MockServer using new(), we connect to an existing remote instance.
    let server = MockServer::connect("localhost:5000");

    let search_mock = server.mock(|when, then| {
        when.path("/search").body("wow so large".repeat(10000000)); // 120 MB body
        then.status(202);
    });

    // Act: Send the HTTP request
    let response = isahc::prelude::Request::post(server.url("/search"))
        .body("wow so large".repeat(10000000)) // 120 MB body
        .unwrap()
        .send()
        .unwrap();

    // Assert
    search_mock.assert();
    assert_eq!(response.status(), 202);
}
