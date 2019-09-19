extern crate mocha;

use mocha::mock;
use mocha::Method::GET;

/// This test is supposed to make sure that mock can be stored, served and deleted.
#[test]
fn api_integration_test_set_and_drop_mock() {
    let m = mock(GET, "/health")
        .return_status(205)
        .return_header("Content-Type", "application/text")
        .return_header("X-Version", "0.0.1")
        .return_body("OK")
        .create();

    let response = reqwest::get("http://localhost:5000/health").expect("ERROR MAKING REQUEST");
    assert_eq!(response.status(), 205);
    assert_eq!(m.times_called(), 1);

    drop(m);

    let response = reqwest::get("http://localhost:5000/health").expect("ERROR MAKING REQUEST");
    assert_eq!(response.status(), 500);

}
