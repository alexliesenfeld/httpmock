extern crate httpmock;

use isahc::prelude::*;
use isahc::{get, get_async, HttpClientBuilder};

use httpmock::Method::{GET, POST};
use httpmock::{Mock, MockServer, MockServerRequest, Regex};
use httpmock_macros::test_executors;
use isahc::config::RedirectPolicy;
use std::fs::read_to_string;
use std::time::{Duration, SystemTime};

/// This test asserts that mocks can be stored, served and deleted as designed.
#[test]
#[test_executors] // Internal macro that executes this test in different async executors. Ignore it.
fn simple_test() {
    // Arrange
    let _ = env_logger::try_init();
    let mock_server = MockServer::start();

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

/// This test shows how to use multiple mock servers in one test.
#[test]
#[test_executors] // Internal macro that executes this test in different async executors. Ignore it.
fn multiple_mock_servers_test() {
    // Arrange
    let _ = env_logger::try_init();
    let mock_server1 = MockServer::start();
    let mock_server2 = MockServer::start();

    let redirect_mock = Mock::new()
        .expect_path("/redirectTest")
        .return_status(302)
        .return_header(
            "Location",
            &format!("http://{}/finalTarget", mock_server2.address()),
        )
        .create_on(&mock_server1);

    let target_mock = Mock::new()
        .expect_path("/finalTarget")
        .return_status(200)
        .create_on(&mock_server2);

    // Act: Send the HTTP request
    let http_client = HttpClientBuilder::new()
        .redirect_policy(RedirectPolicy::Follow)
        .build()
        .unwrap();

    let response = http_client
        .get(&format!("http://{}/redirectTest", mock_server1.address()))
        .unwrap();

    // Assert
    assert_eq!(response.status(), 200);
    assert_eq!(redirect_mock.times_called(), 1);
    assert_eq!(target_mock.times_called(), 1);
}

/// Demonstrates how to use async structures
#[async_std::test]
async fn simple_test_async() {
    // Arrange
    let _ = env_logger::try_init();
    let mock_server = MockServer::start_async().await;

    let search_mock = Mock::new()
        .expect_path_contains("/search")
        .return_status(202)
        .create_on_async(&mock_server)
        .await;

    // Act: Send the HTTP request

    // mock_server.url will create a full URL for the path on the server. In this case it will be
    // http://localhost:<port>/search
    let url = mock_server.url("/search");
    let response = get_async(url).await.unwrap();

    // Assert
    assert_eq!(response.status(), 202);
    assert_eq!(search_mock.times_called_async().await, 1);
}

/// Ensures that once explicitly deleting a mock, it will not be delivered by the server anymore.
#[test]
#[test_executors] // Internal macro that executes this test in different async executors. Ignore it.
fn explicit_delete_test() {
    // Arrange
    let _ = env_logger::try_init();
    let mock_server = MockServer::start();

    let mut m = Mock::new()
        .expect_method(GET)
        .expect_path("/health")
        .return_status(205)
        .create_on(&mock_server);

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

/// Tests and demonstrates body matching.
#[test]
#[test_executors] // Internal macro that executes this test in different async executors. Ignore it.
fn exact_body_match_test() {
    // This is a temporary type that we will use for this test
    #[derive(serde::Serialize, serde::Deserialize)]
    struct TestUser {
        name: String,
    }

    // Arrange
    let _ = env_logger::try_init();
    let mock_server = MockServer::start();

    let m = Mock::new()
        .expect_method(POST)
        .expect_path("/users")
        .expect_header("Content-Type", "application/json")
        .expect_json_body(&TestUser {
            name: String::from("Fred"),
        })
        .return_status(201)
        .return_header("Content-Type", "application/json")
        .return_json_body(&TestUser {
            name: String::from("Hans"),
        })
        .create_on(&mock_server);

    // Act: Send the request and deserialize the response to JSON
    let mut response = Request::post(&format!("http://{}/users", mock_server.address()))
        .header("Content-Type", "application/json")
        .body(
            serde_json::to_string(&TestUser {
                name: String::from("Fred"),
            })
            .unwrap(),
        )
        .unwrap()
        .send()
        .unwrap();

    let user: TestUser =
        serde_json::from_str(&response.text().unwrap()).expect("cannot deserialize JSON");

    // Assert
    assert_eq!(response.status(), 201);
    assert_eq!(user.name, "Hans");
    assert_eq!(m.times_called(), 1);
}

/// Tests and demonstrates matching features.
#[test]
#[test_executors] // Internal macro that executes this test in different async executors. Ignore it.
fn matching_features_test() {
    // This is a temporary type that we will use for this test
    #[derive(serde::Serialize, serde::Deserialize)]
    struct TransferItem {
        number: usize,
    }

    // Arrange
    let _ = env_logger::try_init();
    let mock_server = MockServer::start();

    let m = Mock::new()
        .expect_method(POST)
        .expect_path("/test")
        .expect_path_contains("test")
        .expect_query_param("myQueryParam", "überschall")
        .expect_query_param_exists("myQueryParam")
        .expect_path_matches(Regex::new(r#"test"#).unwrap())
        .expect_header("Content-Type", "application/json")
        .expect_header_exists("User-Agent")
        .expect_body("{\"number\":5}")
        .expect_body_contains("number")
        .expect_body_matches(Regex::new(r#"(\d+)"#).unwrap())
        .expect_json_body(&TransferItem { number: 5 })
        .expect_match(|req: MockServerRequest| req.path.contains("es"))
        .return_status(200)
        .create_on(&mock_server);

    // Act: Send the HTTP request
    let uri = format!(
        "http://{}/test?myQueryParam=%C3%BCberschall",
        mock_server.address()
    );
    let response = Request::post(&uri)
        .header("Content-Type", "application/json")
        .header("User-Agent", "rust-test")
        .body(serde_json::to_string(&TransferItem { number: 5 }).unwrap())
        .unwrap()
        .send()
        .unwrap();

    // Assert
    assert_eq!(response.status(), 200);
    assert_eq!(m.times_called(), 1);
}

/// Tests and demonstrates matching JSON body partials.
#[test]
#[test_executors] // Internal macro that executes this test in different async executors. Ignore it.
fn body_partial_json_str_test() {
    let _ = env_logger::try_init();
    let mock_server = MockServer::start();

    // This is the structure that needs to be included in the request
    #[derive(serde::Serialize, serde::Deserialize)]
    struct ChildStructure {
        some_attribute: String,
    }

    // This is a parent structure that carries the included structure
    #[derive(serde::Serialize, serde::Deserialize)]
    struct ParentStructure {
        some_other_value: String,
        child: ChildStructure,
    }

    // Arranging the test by creating HTTP mocks.
    let m = Mock::new()
        .expect_method(POST)
        .expect_path("/users")
        .expect_json_body_partial(
            r#"
            {
                "child" : {
                    "some_attribute" : "Fred"
                }
            }
        "#,
        )
        .return_status(201)
        .return_body(r#"{"result":"success"}"#)
        .create_on(&mock_server);

    // Simulates application that makes the request to the mock.
    let uri = format!("http://{}/users", m.server_address());
    let mut response = Request::post(&uri)
        .header("Content-Type", "application/json")
        .header("User-Agent", "rust-test")
        .body(
            serde_json::to_string(&ParentStructure {
                child: ChildStructure {
                    some_attribute: "Fred".to_string(),
                },
                some_other_value: "Flintstone".to_string(),
            })
            .unwrap(),
        )
        .unwrap()
        .send()
        .unwrap();

    // Assertions
    assert_eq!(response.status(), 201);
    assert_eq!(m.times_called(), 1);
}

/// This test asserts that mocks can be stored, served and deleted as designed.
#[test]
#[test_executors] // Internal macro that executes this test in different async executors. Ignore it.
fn delay_test() {
    // Arrange
    let _ = env_logger::try_init();
    let start_time = SystemTime::now();
    let delay = Duration::from_secs(5);

    let mock_server = MockServer::start();

    let search_mock = Mock::new()
        .expect_path("/delay")
        .return_with_delay(delay)
        .create_on(&mock_server);

    // Act: Send the HTTP request
    let response = get(mock_server.url("/delay")).unwrap();

    // Assert
    assert_eq!(response.status(), 200);
    assert_eq!(search_mock.times_called(), 1);
    assert_eq!(start_time.elapsed().unwrap() > delay, true);
}
