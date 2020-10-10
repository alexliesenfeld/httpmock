extern crate httpmock;

use isahc::prelude::*;

use httpmock::Method::GET;
use httpmock::{Mock, MockServer};

#[test]
fn cookie_matching_test() {
    // Arrange
    let _ = env_logger::try_init();
    let server = MockServer::start();

    let mock = Mock::new()
        .expect_method(GET)
        .expect_path("/")
        .expect_cookie_exists("SESSIONID")
        .expect_cookie("SESSIONID", "298zf09hf012fh2")
        .return_status(200)
        .create_on(&server);

    // Act: Send the request and deserialize the response to JSON
    let response = Request::get(&format!("http://{}", server.address()))
        .header(
            "Cookie",
            "OTHERCOOKIE1=01234; SESSIONID=298zf09hf012fh2; OTHERCOOKIE2=56789; HttpOnly",
        )
        .body(())
        .unwrap()
        .send()
        .unwrap();

    // Assert
    mock.assert();
    assert_eq!(response.status(), 200);
}
