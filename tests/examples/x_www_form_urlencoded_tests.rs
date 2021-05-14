use httpmock::prelude::*;
use isahc::{prelude::*, Request};

#[test]
fn body_test() {
    // Arrange
    let server = MockServer::connect("127.0.0.1:5000");

    let m = server.mock(|when, then| {
        when.method(POST)
            .path("/example")
            .header("content-type", "application/x-www-form-urlencoded")
            .x_www_form_urlencoded_tuple("name", "Peter Griffin")
            .x_www_form_urlencoded_tuple("town", "Quahog")
            .x_www_form_urlencoded_key_exists("name")
            .x_www_form_urlencoded_key_exists("town");
        then.status(202);
    });

    let response = Request::post(server.url("/example"))
        .header("content-type", "application/x-www-form-urlencoded")
        .body("name=Peter%20Griffin&town=Quahog")
        .unwrap()
        .send()
        .unwrap();

    // Assert
    m.assert();
    assert_eq!(response.status(), 202);
}
