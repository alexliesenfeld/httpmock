extern crate httpmock;

use std::cell::RefCell;
use std::io::Read;
use std::rc::Rc;

use isahc::{get, get_async, Body, RequestExt};
use owning_ref::OwningRef;
use regex::Replacer;

use httpmock::MockServer;

use crate::simulate_standalone_server;

use self::httpmock::{Mock, MockRef};

#[test]
fn loop_with_standalone_test() {
    // Arrange

    // This starts up a standalone server in the background running on port 5000
    simulate_standalone_server();

    // Instead of creating a new MockServer using new(), we connect to an existing remote instance.
    let server = MockServer::connect("localhost:5000");

    for x in 0..1000 {
        let search_mock = server.mock(|when, then| {
            when.path(format!("/test/{}", x));
            then.status(202);
        });

        // Act: Send the HTTP request
        let response = get(server.url(&format!("/test/{}", x))).unwrap();

        // Assert
        search_mock.assert();
        assert_eq!(response.status(), 202);
    }
}

#[test]
fn loop_with_local_test() {
    // Arrange

    // Instead of creating a new MockServer using new(), we connect to an existing remote instance.
    let server = MockServer::start();

    let mock = my_server.mock(
        when.path("/test")
            .path_contains("test")
            .query_param("myQueryParam", "überschall"),
        then.status(202),
    );

    for x in 0..1000 {
        let search_mock = server.mock(|when, then| {
            when.path(format!("/test/{}", x));
            then.status(202);
        });

        // Act: Send the HTTP request
        let response = get(server.url(&format!("/test/{}", x))).unwrap();

        // Assert
        search_mock.assert();

        assert_eq!(response.status(), 202);
    }
}

struct CustomMockRef {
    id: usize,
    /* probably also some custom fields here */
}

struct MyWrapper {
    server: MockServer,
    mocks: RefCell<Vec<CustomMockRef>>,
}

#[test]
fn wrapper_test() {
    let w = MyWrapper {
        server: MockServer::start(),
        mocks: RefCell::new(vec![]),
    };

    let mock = w.server.mock(|when, then| {
        when.path("/test");
        then.status(200);
    });

    w.mocks.borrow_mut().push(CustomMockRef { id: mock.id });

    let mock: MockRef = MockRef::new(mock.id, &w.server);
    mock.hits();
}
