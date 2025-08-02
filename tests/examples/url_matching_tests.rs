use httpmock::prelude::*;
use regex::Regex;

#[test]
fn url_matching_test() {
    // Arrange
    let server = MockServer::start();

    let m = server.mock(|when, then| {
        when.path("/appointments/20200922")
            .path_includes("appointments")
            .path_matches(Regex::new(r"\d{8}$").unwrap());
        then.status(201);
    });

    // Act: Send the request
    reqwest::blocking::get(server.url("/appointments/20200922")).unwrap();

    // Assert
    m.assert();
}
