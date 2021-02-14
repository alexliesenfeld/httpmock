extern crate httpmock;

use isahc::{get, get_async};
use serde_json::json;

use self::httpmock::URLEncodedExtension;
use httpmock::Method::{GET, POST};
use httpmock::{Mock, MockServer};

#[test]
fn getting_started_test() {
    // Start a lightweight mock server.
    let server = MockServer::start();

    let a = String::new();

    let x = a.url_encoded();

    // Create a mock on the server.
    let hello_mock = server.mock(|when, then| {
        when.method("GET")
            .path("/translate")
            //.query_param_new("word", url_encoded("peter"))
            .query_param_new("word", &x)
            .query_param_new("word", x)
            .query_param_new("peter", "griffin".url_encoded());
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
