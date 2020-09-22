extern crate httpmock;

use httpmock::Method::GET;
use httpmock::{Mock, MockServer};
use httpmock_macros::httpmock_example_test;
use isahc::prelude::*;

/// Tests and demonstrates cookie matching.
#[test]
#[httpmock_example_test] // Internal macro to make testing easier. Ignore it.
fn cookie_matching_test() {
    // Arrange
    let _ = env_logger::try_init();
    let mock_server = MockServer::start();

    let mock = Mock::new()
        .expect_method(GET)
        .expect_path("/")
        .expect_cookie_exists("SESSIONID")
        .expect_cookie("SESSIONID", "298zf09hf012fh2")
        .return_status(200)
        .create_on(&mock_server);

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
