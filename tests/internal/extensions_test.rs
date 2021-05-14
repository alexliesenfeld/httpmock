extern crate httpmock;

use self::httpmock::prelude::*;
use self::httpmock::Mock;
use std::cell::RefCell;

// Test for issue https://github.com/alexliesenfeld/httpmock/issues/26
#[test]
fn wrapper_test() {
    // Assume we have some other structures that wrap a MockServer along with its mock objects
    struct MyMockWrapper {
        id: usize,
    }

    struct MyServerWrapper {
        server: MockServer,
        mocks: RefCell<Vec<MyMockWrapper>>,
    }

    // Start a mock server wrapped in another structure
    let sw = MyServerWrapper {
        server: MockServer::start(),
        mocks: RefCell::new(vec![]),
    };

    // Create a mock on the server and store its server ID for later use
    let mock = sw.server.mock(|when, then| {
        when.path("/test");
        then.status(200);
    });

    sw.mocks.borrow_mut().push(MyMockWrapper { id: mock.id });
    drop(mock);

    let mock = Mock::new(sw.mocks.borrow_mut().get(0).unwrap().id, &sw.server);
    mock.hits();
}
