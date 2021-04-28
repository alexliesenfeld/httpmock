use httpmock::prelude::*;
use isahc::{prelude::*, Request};

#[test]
fn body_test() {
    // Arrange
    let server = MockServer::connect("127.0.0.1:5000");

    let m = server.mock(|when, then| {
        when.method(POST)
            .path("/students")
            .x_www_form_urlencoded("name", "Peter Griffin")
            .x_www_form_urlencoded_key_exists("name");
        then.status(201);
    });

}
