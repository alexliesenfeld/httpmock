extern crate httpmock;

use std::io::Read;

use isahc::{get, get_async, Body};

use httpmock::{Mock, MockServer};

use crate::simulate_standalone_server;

#[test]
fn standalone_test() {
    // This starts up a standalone server in the background running on port 5000
    simulate_standalone_server();

    // Arrange
    let _ = env_logger::try_init();

    // Instead of creating a new MockServer using new(), we connect to an existing remote instance.
    let server = MockServer::connect("localhost:5000");

    let mock = Mock::new()
        .expect_path_contains("/search")
        .expect_query_param("query", "metallica")
        .return_status(202)
        .create_on(&server);

    // Act: Send the HTTP request
    let response = get(&format!(
        "http://{}/search?query=metallica",
        server.address()
    ))
    .unwrap();

    // Assert
    mock.assert();
    assert_eq!(response.status(), 202);
}

/// Demonstrates how to use async structures
#[async_std::test]
async fn async_standalone_test() {
    // This starts up a standalone server in the background running on port 5000
    simulate_standalone_server();

    // Arrange
    let _ = env_logger::try_init();

    // Instead of creating a new MockServer using connect_from_env_async(), we connect by
    // reading the host and port from the environment (HTTPMOCK_HOST / HTTPMOCK_PORT) or
    // falling back to defaults (localhost on port 5000)
    let server = MockServer::connect_from_env_async().await;

    let mut mock = Mock::new()
        .expect_path_contains("/search")
        .expect_query_param("query", "metallica")
        .return_status(202)
        .create_on_async(&server)
        .await;

    // Act: Send the HTTP request
    let response = get_async(&format!(
        "http://{}/search?query=metallica",
        server.address()
    ))
    .await
    .unwrap();

    // Assert 1
    assert_eq!(response.status(), 202);
    assert_eq!(mock.hits_async().await, 1);

    // Act 2: Delete the mock and send a request to show that it is not present on the server anymore
    mock.delete();
    let response = get_async(&format!(
        "http://{}:{}/search?query=metallica",
        server.host(),
        server.port()
    ))
    .await
    .unwrap();

    // Assert: The mock was not found
    assert_eq!(response.status(), 404);
}

/// This test asserts that mocks can be stored, served and deleted as designed.
#[test]
#[should_panic]
fn unsupported_features() {
    let _ = env_logger::try_init();

    // This starts up a standalone server in the background running on port 5000
    simulate_standalone_server();

    // Instead of creating a new MockServer using connect_from_env(), we connect by reading the
    // host and port from the environment (HTTPMOCK_HOST / HTTPMOCK_PORT) or falling back to defaults
    let server = MockServer::connect_from_env();

    // Creating this mock will panic because expect_match is not supported when using
    // a remote mock server.
    let _ = Mock::new().expect_match(|_| true).create_on(&server);
}

#[test]
fn binary_body_standalone_test() {
    let _ = env_logger::try_init();

    // This starts up a standalone server in the background running on port 5000
    simulate_standalone_server();

    // Arrange
    let binary_content = b"\x80\x02\x03\xF0\x90\x80";

    let server = MockServer::connect_from_env();
    let m = Mock::new()
        .expect_path("/hello")
        .return_status(200)
        .return_body(binary_content)
        .create_on(&server);

    // Act
    let mut response = isahc::get(server.url("/hello")).unwrap();

    // Assert
    m.assert();
    assert_eq!(response.status(), 200);
    assert_eq!(body_to_vec(response.body_mut()), binary_content.to_vec());
}

fn body_to_vec(body: &mut Body) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    body.read_to_end(&mut buf).expect("Cannot read from body");
    buf
}
