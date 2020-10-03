extern crate httpmock;

use isahc::prelude::*;

use httpmock::MockServer;
use httpmock_macros::httpmock_example_test;

#[test]
#[httpmock_example_test] // Internal macro to make testing easier. Ignore it.
fn temporary_redirect_test() {
    // Arrange
    let _ = env_logger::try_init();
    let server = MockServer::start();

    let redirect_mock = server.mock(|when, then| {
        when.path("/redirectPath");
        then.temporary_redirect("http://www.google.com");
    });

    // Act: Send the HTTP request with an HTTP client that DOES NOT FOLLOW redirects automatically!
    let mut response = isahc::get(server.url("/redirectPath")).unwrap();
    let body = response.text().unwrap();

    // Assert
    assert_eq!(redirect_mock.hits(), 1);

    // Attention!: Note that all of these values are automatically added to the response
    // (see details in mock builder method documentation).
    assert_eq!(response.status(), 302);
    assert_eq!(body, "Found");
    assert_eq!(
        response
            .headers()
            .get("Location")
            .unwrap()
            .to_str()
            .unwrap(),
        "http://www.google.com"
    );
}

#[test]
#[httpmock_example_test] // Internal macro to make testing easier. Ignore it.
fn permanent_redirect_test() {
    // Arrange
    let _ = env_logger::try_init();
    let server = MockServer::start();

    let redirect_mock = server.mock(|when, then| {
        when.path("/redirectPath");
        then.permanent_redirect("http://www.google.com");
    });

    // Act: Send the HTTP request with an HTTP client that DOES NOT FOLLOW redirects automatically!
    let mut response = isahc::get(server.url("/redirectPath")).unwrap();
    let body = response.text().unwrap();

    // Assert
    assert_eq!(redirect_mock.hits(), 1);

    // Attention!: Note that all of these values are automatically added to the response
    // (see details in mock builder method documentation).
    assert_eq!(response.status(), 301);
    assert_eq!(body, "Moved Permanently");
    assert_eq!(
        response
            .headers()
            .get("Location")
            .unwrap()
            .to_str()
            .unwrap(),
        "http://www.google.com"
    );
}