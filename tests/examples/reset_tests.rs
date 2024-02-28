use httpmock::prelude::*;
use isahc::get;

#[async_std::test]
async fn reset_server_test() {
    // Start a lightweight mock server.
    let server = MockServer::start();

    // Create a mock on the server that will be reset later
    server.mock(|when, then| {
        when.method("GET")
            .path("/translate")
            .query_param("word", "hello");
        then.status(500)
            .header("content-type", "text/html; charset=UTF-8")
            .body("Привет");
    });

    // Delete all previously created mocks
    server.reset_async().await;

    // Create a new mock that will replace the previous one
    let hello_mock = server.mock(|when, then| {
        when.method("GET")
            .path("/translate")
            .query_param("word", "hello");
        then.status(200)
            .header("content-type", "text/html; charset=UTF-8")
            .body("Привет");
    });

    // Send an HTTP request to the mock server. This simulates your code.
    let response = get(server.url("/translate?word=hello")).unwrap();

    // Ensure the specified mock was called.
    hello_mock.assert();

    // Ensure the mock server did respond as specified.
    assert_eq!(response.status(), 200);
}

