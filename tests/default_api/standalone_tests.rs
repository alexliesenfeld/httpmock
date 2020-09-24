extern crate httpmock;

use isahc::{get, get_async};

use crate::simulate_standalone_server;
use httpmock::MockServer;
use httpmock_macros::httpmock_example_test;

#[test]
#[httpmock_example_test] // Internal macro to make testing easier. Ignore it.
fn standalone_test() {
    // This starts up a standalone server in the background running on port 5000
    simulate_standalone_server();

    // Arrange
    let _ = env_logger::try_init();

    // Instead of creating a new MockServer using new(), we connect to an existing remote instance.
    let mock_server = MockServer::connect("localhost:5000");

    let search_mock = mock_server.mock(|when, then| {
        when.path_contains("/search")
            .query_param("query", "metallica");
        then.status(202);
    });

    // Act: Send the HTTP request
    let response = get(&format!(
        "http://{}/search?query=metallica",
        mock_server.address()
    ))
    .unwrap();

    // Assert
    assert_eq!(response.status(), 202);
    assert_eq!(search_mock.times_called(), 1);
}

#[async_std::test]
async fn async_standalone_test() {
    // This starts up a standalone server in the background running on port 5000
    simulate_standalone_server();

    // Arrange
    let _ = env_logger::try_init();

    // Instead of creating a new MockServer using connect_from_env_async(), we connect by
    // reading the host and port from the environment (HTTPMOCK_HOST / HTTPMOCK_PORT) or
    // falling back to defaults (localhost on port 5000)
    let mock_server = MockServer::connect_from_env_async().await;

    let mut search_mock = mock_server
        .mock_async(|when, then| {
            when.path_contains("/search")
                .query_param("query", "metallica");
            then.status(202);
        })
        .await;

    // Act: Send the HTTP request
    let response = get_async(&format!(
        "http://{}/search?query=metallica",
        mock_server.address()
    ))
    .await
    .unwrap();

    // Assert 1
    assert_eq!(response.status(), 202);
    assert_eq!(search_mock.times_called_async().await, 1);

    // Act 2: Delete the mock and send a request to show that it is not present on the server anymore
    search_mock.delete();
    let response = get_async(&format!(
        "http://{}:{}/search?query=metallica",
        mock_server.host(),
        mock_server.port()
    ))
    .await
    .unwrap();

    // Assert: The mock was not found
    assert_eq!(response.status(), 404);
}

#[test]
#[should_panic]
fn unsupported_features() {
    let _ = env_logger::try_init();

    // This starts up a standalone server in the background running on port 5000
    simulate_standalone_server();

    // Instead of creating a new MockServer using connect_from_env(), we connect by reading the
    // host and port from the environment (HTTPMOCK_HOST / HTTPMOCK_PORT) or falling back to defaults
    let mock_server = MockServer::connect_from_env();

    // Creating this mock will panic because expect_match is not supported when using
    // a remote mock server.
    let _ = mock_server.mock(|when, _then| {
        when.matches(|_| true);
    });
}