use httpmock::prelude::*;

#[test]
fn my_custom_request_matcher_test() {
    // Arrange
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.is_true(|req| req.uri().path().ends_with("Test"));
        then.status(201);
    });

    // Act: Send the HTTP request using reqwest
    let response = reqwest::blocking::get(&server.url("/thisIsMyTest")).unwrap();

    // Assert
    mock.assert();
    assert_eq!(response.status(), 201);
}
