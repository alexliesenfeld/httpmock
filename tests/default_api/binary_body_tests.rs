extern crate httpmock;

use httpmock::Method::GET;
use httpmock::{Mock, MockServer};
use httpmock_macros::httpmock_example_test;
use isahc::prelude::*;

#[test]
#[httpmock_example_test] // Internal macro to make testing easier. Ignore it.
fn binary_body_test() {
    assert_eq!(1,2);
}
