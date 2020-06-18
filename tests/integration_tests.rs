extern crate httpmock;

use httpmock::Method::{GET, POST};
use httpmock::{mock, with_mock_server, Mock, Regex};
use std::io::Read;

/// This test asserts that mocks can be stored, served and deleted as designed.
#[test]
#[with_mock_server]
fn simple_test() {
    let _ = env_logger::try_init();

    let search_mock = mock(GET, "/search")
        .expect_query_param("query", "metallica")
        .return_status(204)
        .create();

    let response = reqwest::blocking::get("http://localhost:5000/search?query=metallica").unwrap();

    assert_eq!(response.status(), 204);
    assert_eq!(search_mock.times_called(), 1);
}

/// Ensures that once explicitly deleting a mock, it will not be delivered by the server anymore.
#[test]
#[with_mock_server]
fn explicit_delete_test() {
    let _ = env_logger::try_init();

    let mut m = mock(GET, "/health").return_status(205).create();

    let response = reqwest::blocking::Client::new()
        .get("http://localhost:5000/health")
        .send()
        .unwrap();

    assert_eq!(response.status(), 205);
    assert_eq!(m.times_called(), 1);

    m.delete();

    let response = reqwest::blocking::get("http://localhost:5000/health").unwrap();
    assert_eq!(response.status(), 500);
}

/// Tests and demonstrates body matching.
#[test]
#[with_mock_server]
fn exact_body_match_test() {
    let _ = env_logger::try_init();

    #[derive(serde::Serialize, serde::Deserialize)]
    struct TestUser {
        name: String,
    }

    // Arranging the test by creating HTTP mocks.
    let m = mock(POST, "/users")
        .expect_header("Content-Type", "application/json")
        .expect_json_body(&TestUser {
            name: String::from("Fred"),
        })
        .return_status(201)
        .return_header("Content-Type", "application/json")
        .return_json_body(&TestUser {
            name: String::from("Hans"),
        })
        .create();

    // Simulates application that makes the request to the mock.
    let client = reqwest::blocking::Client::new();
    let mut response = client
        .post(&format!("http://{}/users", m.server_address()))
        .json(&TestUser {
            name: String::from("Fred"),
        })
        .header("Content-Type", "application/json")
        .send()
        .expect("request failed");

    // Extract the response body
    let mut response_body = String::new();
    response
        .read_to_string(&mut response_body)
        .expect("cannot read from response body");

    // Deserialize JSON response body
    let user: TestUser = serde_json::from_str(&response_body).expect("cannot deserialize JSON");

    // Assertions
    assert_eq!(response.status(), 201);
    assert_eq!(user.name, "Hans");
    assert_eq!(m.times_called(), 1);
}

/// Tests and demonstrates matching features.
#[test]
#[with_mock_server]
fn matching_features_test() {
    let _ = env_logger::try_init();

    #[derive(serde::Serialize, serde::Deserialize)]
    struct TransferItem {
        number: usize,
    }

    let m = Mock::new()
        .expect_path("/test")
        .expect_path_contains("test")
        .expect_query_param("myQueryParam", "überschall")
        .expect_query_param_exists("myQueryParam")
        .expect_path_matches(Regex::new(r#"test"#).unwrap())
        .expect_method(POST)
        .expect_header("Content-Type", "application/json")
        .expect_header_exists("User-Agent")
        .expect_body("{\"number\":5}")
        .expect_body_contains("number")
        .expect_body_matches(Regex::new(r#"(\d+)"#).unwrap())
        .expect_json_body(&TransferItem { number: 5 })
        .return_status(200)
        .create();

    let response = reqwest::blocking::Client::new()
        .post("http://localhost:5000/test?myQueryParam=%C3%BCberschall")
        .header("Content-Type", "application/json")
        .header("User-Agent", "rust-test")
        .json(&TransferItem { number: 5 })
        .send()
        .expect("error sending request to mock server");

    assert_eq!(response.status(), 200);
    assert_eq!(m.times_called(), 1);
}

/// Tests and demonstrates matching JSON body partials.
#[test]
#[with_mock_server]
fn body_partial_json_str_test() {
    let _ = env_logger::try_init();

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
    let m = mock(POST, "/users")
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
        .create();

    // Simulates application that makes the request to the mock.
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&format!("http://{}/users", m.server_address()))
        .json(&ParentStructure {
            child: ChildStructure {
                some_attribute: "Fred".to_string(),
            },
            some_other_value: "Flintstone".to_string(),
        })
        .send()
        .expect("request failed");

    // Assertions
    assert_eq!(response.status(), 201);
    assert_eq!(m.times_called(), 1);
}
