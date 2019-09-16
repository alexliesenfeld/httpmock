extern crate httpmock;

use httpmock::api::mock;
use httpmock::api::Method::GET;
use httpmock::api::Scheme::HTTP;

/// This test is supposed to make sure that mock can be stored, served and deleted.
#[test]
fn to_route_response_internal_server_error() {
    let _m = mock(GET, "/health")
        .expect_scheme(HTTP)
        .return_status(205)
        .return_header("Content-Type", "application/text")
        .return_header("X-Version", "0.0.1")
        .return_body("OK")
        .create();

    let r = reqwest::get("http://localhost:5000/health").expect("ERROR MAKING REQUEST");

    assert_eq!(205, r.status());
}
