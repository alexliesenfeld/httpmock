extern crate httpmock;

use isahc::prelude::*;
use serde_json::json;

use httpmock::Method::POST;
use httpmock::{MockServer, MockServerRequest, Regex};
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

    let m = mock_server.mock(|when, then| {
        when.method(POST)
            .path("/test")
            .path_contains("test")
            .query_param("myQueryParam", "überschall")
            .query_param_exists("myQueryParam")
            .path_matches(Regex::new(r#"test"#).unwrap())
            .header("Content-Type", "application/json")
            .header_exists("User-Agent")
            .body("{\"number\":5}")
            .body_contains("number")
            .body_matches(Regex::new(r#"(\d+)"#).unwrap())
            .json_body(json!({ "number": 5 }))
            .matches(|req: MockServerRequest| req.path.contains("es"));
        then.status(200);
    });

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
