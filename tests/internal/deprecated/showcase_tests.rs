extern crate httpmock;

use isahc::{prelude::*, Request};
use serde_json::json;

use self::httpmock::HttpMockRequest;
use httpmock::Method::POST;
use httpmock::{Mock, MockServer, Regex};

#[test]
fn showcase_test() {
    // This is a temporary type that we will use for this test
    #[derive(serde::Serialize, serde::Deserialize)]
    struct TransferItem {
        number: usize,
    }

    // Arrange
    let _ = env_logger::try_init();
    let server = MockServer::start();

    let m = Mock::new()
        .expect_method(POST)
        .expect_path("/test")
        .expect_path_contains("test")
        .expect_query_param("myQueryParam", "überschall")
        .expect_query_param_exists("myQueryParam")
        .expect_path_matches(Regex::new(r#"test"#).unwrap())
        .expect_header("content-type", "application/json")
        .expect_header_exists("User-Agent")
        .expect_body("{\"number\":5}")
        .expect_body_contains("number")
        .expect_body_matches(Regex::new(r#"(\d+)"#).unwrap())
        .expect_json_body(json!({ "number": 5 }))
        .expect_match(|req: &HttpMockRequest| req.path.contains("es"))
        .return_status(200)
        .create_on(&server);

    // Act: Send the HTTP request
    let uri = format!(
        "http://{}/test?myQueryParam=%C3%BCberschall",
        server.address()
    );
    let response = Request::post(&uri)
        .header("content-type", "application/json")
        .header("User-Agent", "rust-test")
        .body(serde_json::to_string(&TransferItem { number: 5 }).unwrap())
        .unwrap()
        .send()
        .unwrap();

    // Assert
    m.assert();
    assert_eq!(response.status(), 200);
}
