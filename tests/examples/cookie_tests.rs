use httpmock::prelude::*;
use reqwest::blocking::Client;

#[test]
fn cookie_matching_test() {
    // Arrange
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method("GET")
            .path("/")
            .cookie_exists("SESSIONID")
            .cookie("SESSIONID", "298zf09hf012fh2");
        then.status(200);
    });

    // Act: Send the request with cookies
    let client = Client::new();
    let response = client
        .get(&format!("http://{}", server.address()))
        .header(
            "Cookie",
            "OTHERCOOKIE1=01234; SESSIONID=298zf09hf012fh2; OTHERCOOKIE2=56789; HttpOnly",
        )
        .send()
        .unwrap();

    // Assert
    mock.assert();
    assert_eq!(response.status(), 200);
}
