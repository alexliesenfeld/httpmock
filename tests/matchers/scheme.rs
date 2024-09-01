use crate::matchers::expect_fails_with;
use httpmock::MockServer;
use reqwest::blocking::get;

#[test]
fn scheme_tests() {
    // Arrange
    let server = MockServer::start();

    let m = server.mock(|when, then| {
        when.scheme("http");
        then.status(200);
    });

    // Act
    let response = get(&server.base_url()).unwrap();

    // Assert
    m.assert();
    assert_eq!(response.status(), 200);
}

#[test]
fn scheme_failure() {
    expect_fails_with(
        || {
            // Arrange
            let server = MockServer::start();

            let m = server.mock(|when, then| {
                when.scheme("https");
                then.status(200);
            });

            // Act
            get(&server.base_url()).unwrap();

            m.assert()
        },
        vec![
            "Scheme Mismatch",
            "Expected",
            "scheme equals",
            "https",
            "Received",
            "http",
        ],
    )
}

#[test]
fn scheme_not_tests() {
    // Arrange
    let server = MockServer::start();

    let m = server.mock(|when, then| {
        when.scheme_not("https");
        then.status(200);
    });

    // Act
    let response = get(&server.base_url()).unwrap();

    // Assert
    m.assert();
    assert_eq!(response.status(), 200);
}

#[test]
fn scheme_not_failure() {
    expect_fails_with(
        || {
            // Arrange
            let server = MockServer::start();

            let m = server.mock(|when, then| {
                when.scheme_not("http");
                then.status(200);
            });

            // Act
            get(&server.base_url()).unwrap();

            m.assert()
        },
        vec![
            "Scheme Mismatch",
            "Expected",
            "scheme not equal to",
            "http",
            "Received",
            "http",
        ],
    )
}
