extern crate httpmock;
#[macro_use]
extern crate lazy_static;

use std::sync::Mutex;
use std::thread::{spawn, JoinHandle};

use isahc::{get, get_async};
use tokio::task::LocalSet;

use httpmock::standalone::start_standalone_server;
use httpmock::{HttpMockConfig, Mock, MockServer};
use httpmock_macros::test_executors;
use actix_rt::Builder;

/// This test asserts that mocks can be stored, served and deleted as designed.
// Ignore this "test_executors" macro. It runs tests in multiple async runtimes for quality assurance.
#[test_executors]
#[test]
fn simple_standalone_test() {
    // This starts up a standalone server in the background running on port 5000
    simulate_standalone_server();

    // Arrange
    let _ = env_logger::try_init();

    // Instead of creating a new MockServer using new(), we connect to an existing remote instance.
    let mock_server = MockServer::connect("localhost:5000");

    let search_mock = Mock::new()
        .expect_path_contains("/search")
        .expect_query_param("query", "metallica")
        .return_status(202)
        .create_on(&mock_server);

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

/// Demonstrates how to use async structures
#[async_std::test]
async fn simple_standalone_test_async() {
    // This starts up a standalone server in the background running on port 5000
    simulate_standalone_server();

    // Arrange
    let _ = env_logger::try_init();

    // Instead of creating a new MockServer using connect_from_env_async(), we connect by
    // reading the host and port from the environment (HTTPMOCK_HOST / HTTPMOCK_PORT) or
    // falling back to defaults (localhost on port 5000)
    let mock_server = MockServer::connect_from_env_async().await;

    let mut search_mock = Mock::new()
        .expect_path_contains("/search")
        .expect_query_param("query", "metallica")
        .return_status(202)
        .create_on_async(&mock_server)
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

/// This test asserts that mocks can be stored, served and deleted as designed.
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
    let _ = Mock::new().expect_match(|_| true).create_on(&mock_server);
}

/// The rest of this file is only required to simulate that a standalone mock server is
/// running somewhere else. The tests above will is.
fn simulate_standalone_server() {
    println!("test");
    let _ = STANDALONE_SERVER.lock().unwrap_or_else(|e| e.into_inner());
}

lazy_static! {
    static ref STANDALONE_SERVER: Mutex<JoinHandle<Result<(), String>>> = Mutex::new(spawn(|| {
        let srv = start_standalone_server(HttpMockConfig::new(5000, false));
        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        LocalSet::new().block_on(&mut runtime, srv)
    }));
}
