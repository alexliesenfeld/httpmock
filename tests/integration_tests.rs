extern crate httpmock;

use isahc::prelude::*;
use isahc::{get};

use httpmock::Method::{GET, POST};
use httpmock::{Mock, MockServer, MockServerRequest, Regex};

use httpmock_macros::repeat_for_all_supported_executors;

/// This test asserts that mocks can be stored, served and deleted as designed.
#[test]
#[repeat_for_all_supported_executors]
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

/// Ensures that once explicitly deleting a mock, it will not be delivered by the server anymore.
#[test]
#[repeat_for_all_supported_executors]
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
    let response = get(&format!("http://{}/health", mock_server.address())).unwrap();

    // Assert
    assert_eq!(response.status(), 205);
    assert_eq!(m.times_called(), 1);

    // Delete the mock and send the request again
    m.delete();

    let response = get(&format!("http://{}/health", mock_server.address())).unwrap();

    // Assert that the request failed, because the mock has been deleted
    assert_eq!(response.status(), 500);
}

/// Tests and demonstrates body matching.
#[test]
#[repeat_for_all_supported_executors]
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
#[repeat_for_all_supported_executors]
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
#[repeat_for_all_supported_executors]
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
        .create_on(&mock_server);

    // Simulates application that makes the request to the mock.
    let uri = format!("http://{}/users", m.server_address());
    let response = Request::post(&uri)
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
