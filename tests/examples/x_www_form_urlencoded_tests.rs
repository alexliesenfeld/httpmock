use httpmock::prelude::*;
use reqwest::blocking::Client;

#[test]
fn body_test_xxx_form_url_encoded() {
    // Arrange
    let server = MockServer::start();

    let m = server.mock(|when, then| {
        when.method(POST)
            .path("/example")
            .header("content-type", "application/x-www-form-urlencoded")
            .form_urlencoded_tuple("name", "Peter Griffin")
            .form_urlencoded_tuple("town", "Quahog")
            .form_urlencoded_tuple_exists("name")
            .form_urlencoded_tuple_exists("town");
        then.status(202);
    });

    let client = Client::new();
    let response = client
        .post(&server.url("/example"))
        .header("content-type", "application/x-www-form-urlencoded")
        .body("name=Peter%20Griffin&town=Quahog")
        .send()
        .unwrap();

    // Assert
    m.assert();
    assert_eq!(response.status(), 202);
}
