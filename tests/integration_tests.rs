extern crate mocha;

use mocha::mock;
use mocha::Method::GET;

/// This test is supposed to make sure that mock can be stored, served and deleted.
#[test]
fn to_route_response_internal_server_error() {
    let _m = mock(GET, "/health")
        .return_status(205)
        .return_header("Content-Type", "application/text")
        .return_header("X-Version", "0.0.1")
        .return_body("OK")
        .create();

    let first_response =
        reqwest::get("http://localhost:5000/health").expect("ERROR MAKING REQUEST");

    drop(_m);

    let second_response =
        reqwest::get("http://localhost:5000/health").expect("ERROR MAKING REQUEST");

    assert_eq!(205, first_response.status());
    assert_eq!(500, second_response.status());
}
