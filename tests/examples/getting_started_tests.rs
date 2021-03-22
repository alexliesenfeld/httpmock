extern crate httpmock;

use isahc::{get, get_async};

use self::httpmock::Mock;
use httpmock::Method::GET;
use httpmock::MockServer;

#[test]
fn getting_started_test() {
    // Start a lightweight mock server.
    let server = MockServer::start();

    // Create a mock on the server.
    let hello_mock = server.mock(|when, then| {
        when.method("GET")
            .path("/translate")
            .query_param("word", "hello");
        then.status(200)
            .header("Content-Type", "text/html; charset=UTF-8")
            .body("Привет");
    });

    // Send an HTTP request to the mock server. This simulates your code.
    let response = get(server.url("/translate?word=hello")).unwrap();

    // Ensure the specified mock was called.
    hello_mock.assert();

    // Ensure the mock server did respond as specified.
    assert_eq!(response.status(), 200);
}

#[async_std::test]
async fn async_getting_started_test() {
    // Start a local mock server for exclusive use by this test function.
    let server = MockServer::start_async().await;

    // Create a mock on the mock server. The mock will return HTTP status code 200 whenever
    // the mock server receives a GET-request with path "/hello".
    let hello_mock = Mock::new()
        .expect_method(GET)
        .expect_path("/hello")
        .return_status(200)
        .create_on_async(&server)
        .await;

    // Send an HTTP request to the mock server. This simulates your code.
    let url = format!("http://{}/hello", server.address());
    let response = get_async(&url).await.unwrap();

    // Ensure the specified mock responded exactly one time.
    hello_mock.assert_async().await;
    // Ensure the mock server did respond as specified above.
    assert_eq!(response.status(), 200);
}
