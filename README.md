# HTTP mock server 
A simple-to-use HTTP mock server that can be used for local tests as
well as tests that span multiple systems. It provides a local (or remote) mock server and
a library to create, verify and remove HTTP mocks.
# Usage
 If used without a dedicated (standalone) mock server instance, an HTTP mock server will
 automatically be created in the background of your tests. The local mock server is created
 in a separate thread that will be started when a test needs a mock server for the first time.
 It will be shut down at the end of the test run.

 Should you need to extend or change your tests to span multiple systems later (such as in
 system integration tests), you can switch the tests to use a standalone mock server by simply
 setting the address of the remote server using an environment variable. This way the remote
 server will be used for mocking and your mocks will be available to all participating systems.
 A standalone version of the HTTP mock server is available as an executable binary.

 ## Getting Started
 You can use a local mock server in your tests like shown in the following:
 ```rust
 extern crate httpmock;

use httpmock::Method::GET;
use httpmock::mock;
use httpmock_macros::with_mock_server;

#[test]
#[with_mock_server]
fn simple_test() {
    let health_mock = mock(GET, "/search")
        .expect_query_param("query", "metallica")
        .return_status(204)
        .create();

    let response = reqwest::get("http://localhost:5000/search?query=metallica").unwrap();

    assert_eq!(response.status(), 204);
    assert_eq!(health_mock.times_called(), 1);
 }
 ```
 In the snippet, a mock server is automatically created when the test launches.
 This is ensured by the [with_mock_server](httpmock_macros::with_mock_server) annotation, which
 wraps the test inside an initializer function performing multiple preparation steps, such as
 starting a server (if none yet exists) or clearing the server from old mocks (cleaning also
 happens after the test ended). It also sequentializes tests that involve a mock server.
 If you try to create a mock without having annotated you test function
 with the [with_mock_server](httpmock_macros::with_mock_server) annotation, you will receive a
 panic at runtime pointing you to this problem.
 You can provide expected request attributes (such as headers, body content, etc.)
 and values that will be returned by the mock to the calling application using the
`expect_xxx` and `return_xxx` methods, respectively. The ´create´ method will eventually
 make a request to the mock server (either local or remote) to create the mock at the server.

 You can use the mock object returned by the ´create´ method to fetch information about
 it from the mock server. This might be the number of times this mock has been called.
 You might use this information in your test assertions.

 An HTTP request made by your application is only considered to match a mock if the request
 fulfills all specified requirements. If a request does not match any mock, the mock server will
 respond with an empty response body and a status code 500 (Internal Server Error).

 By default, if a server port is not provided using an environment variable (MOCHA_SERVER_PORT),
 the port 5000 will be used. If another server host is explicitely set
 using an environment variable (MOCHA_SERVER_HOST), then this API will use the remote server
 managing mocks.
 
 
 ## Debugging
 `httpmock` logs against the `log` crate. If you use the `env_logger` backend, you can activate debug logging 
 by setting `RUST_LOG` environment variable to `debug` and then calling `env_logger::try_init()`  in your tests and like this:

In your test:
```rust
#[test]
fn your_test() {
    let _ = env_logger::try_init();
    // ...
}
```

Starting your tests from command line:
```
RUST_LOG=httpmock=debug cargo test
```
  
 
 ## License
 `httpmock` is free software: you can redistribute it and/or modify it under the terms of the MIT Public License.
 
 This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the MIT Public License for more details.