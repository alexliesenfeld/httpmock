#[macro_use]
extern crate lazy_static;

use crate::common::prepare_test_environment;
use log::debug;

mod common;

/// This test is supposed to make sure that mock can be stored, served and deleted.
#[test]
fn to_route_response_internal_server_error() {
    prepare_test_environment();

    let mut response = reqwest::get("http://localhost:5000/__admin/health").unwrap();
    debug!("Response: {:?}", response);
    debug!("Body: {:?}", response.text().unwrap());
}

#[test]
fn to_route_response_internal_server_error_2() {
    prepare_test_environment();

    let mut response = reqwest::get("http://localhost:5000/__admin/health").unwrap();
    debug!("Response: {:?}", response);
    debug!("Body: {:?}", response.text().unwrap());
}
