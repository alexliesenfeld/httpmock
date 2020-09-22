extern crate httpmock;

use httpmock::Method::GET;
use httpmock::{MockServer};
use httpmock_macros::httpmock_example_test;
use isahc::prelude::*;

#[test]
#[httpmock_example_test] // Internal macro to make testing easier. Ignore it.
fn cookie_matching_test() {
    // Arrange
    let _ = env_logger::try_init();
    let mock_server = MockServer::start();

    let mock = mock_server.mock(|when, then| {
        when.method(GET)
            .path("/")
            .cookie_exists("SESSIONID")
            .cookie("SESSIONID", "298zf09hf012fh2");
        then.status(200);
    });

    // Act: Send the request and deserialize the response to JSON
    let response = Request::get(&format!("http://{}", mock_server.address()))
        .header(
            "Cookie",
            "OTHERCOOKIE1=01234; SESSIONID=298zf09hf012fh2; OTHERCOOKIE2=56789",
        )
        .body(())
        .unwrap()
        .send()
        .unwrap();

    // Assert
    assert_eq!(response.status(), 200);
    assert_eq!(mock.times_called(), 1);
}
