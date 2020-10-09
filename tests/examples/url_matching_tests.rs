extern crate httpmock;

use self::httpmock::Mock;
use httpmock::{MockServer, Regex};
use httpmock_macros::httpmock_example_test;
use isahc::get;

#[test]
#[httpmock_example_test] // Internal macro to make testing easier. Ignore it.
fn url_matching_test() {
    // Arrange
    let _ = env_logger::try_init();
    let server = MockServer::start();

    let m = server.mock(|when, then| {
        when.path("/appointments/20200922")
            .path_contains("appointments")
            .path_matches(Regex::new(r"\d{4}\d{2}\d{2}$").unwrap());
        then.status(201);
    });

    // Act: Send the request and deserialize the response to JSON
    get(server.url("/appointments/20200922")).unwrap();

    // Assert
    m.assert();
}
