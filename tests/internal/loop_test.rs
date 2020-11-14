extern crate httpmock;

use std::io::Read;

use isahc::{get, get_async, Body, RequestExt};
use regex::Replacer;

use httpmock::MockServer;

use crate::simulate_standalone_server;

#[test]
fn loop_with_standalone_test() {
    // Arrange

    // This starts up a standalone server in the background running on port 5000
    simulate_standalone_server();

    // Instead of creating a new MockServer using new(), we connect to an existing remote instance.
    let server = MockServer::connect("localhost:5000");

    for x in 0..1000 {
        let search_mock = server.mock(|when, then| {
            when.path(format!("/test/{}", x));
            then.status(202);
        });

        // Act: Send the HTTP request
        let response = get(server.url(&format!("/test/{}", x))).unwrap();

        // Assert
        search_mock.assert();
        assert_eq!(response.status(), 202);
    }
}

#[test]
fn loop_with_local_test() {
    // Arrange

    // Instead of creating a new MockServer using new(), we connect to an existing remote instance.
    let server = MockServer::start();

    for x in 0..1000 {
        let search_mock = server.mock(|when, then| {
            when.path(format!("/test/{}", x));
            then.status(202);
        });

        // Act: Send the HTTP request
        let response = get(server.url(&format!("/test/{}", x))).unwrap();

        // Assert
        search_mock.assert();
        assert_eq!(response.status(), 202);
    }
}
