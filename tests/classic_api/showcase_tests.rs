extern crate httpmock;

use isahc::prelude::*;
use serde_json::json;

use httpmock::Method::POST;
use httpmock::{Mock, MockServer, MockServerRequest, Regex};
use httpmock_macros::httpmock_example_test;

#[test]
#[httpmock_example_test] // Internal macro to make testing easier. Ignore it.
fn showcase_test() {
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
        .expect_json_body(json!({ "number": 5 }))
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
