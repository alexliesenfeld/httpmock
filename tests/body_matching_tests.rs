extern crate httpmock;

use isahc::prelude::*;
use isahc::{get, get_async, HttpClientBuilder};

use httpmock::Method::{GET, POST};
use httpmock::{Mock, MockServer, MockServerRequest, Regex};
use httpmock_macros::test_executors;
use isahc::config::RedirectPolicy;
use std::fs::read_to_string;
use std::time::{Duration, SystemTime};

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
