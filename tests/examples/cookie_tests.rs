use isahc::{prelude::*, Request};
use httpmock::prelude::*;

#[test]
fn cookie_matching_test() {
    // Arrange
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/")
            .cookie_exists("SESSIONID")
            .cookie("SESSIONID", "298zf09hf012fh2");
        then.status(200);
    });

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
