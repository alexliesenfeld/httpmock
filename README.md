# HTTP mock server 
A simple-to-use HTTP mock server that can be used for mocking HTTP calls in your tests. This
 crate can be used for both, local tests as well as tests that span multiple systems.
 It provides an API to create mocks on a local or remote mock server.

 If used without a dedicated (standalone) mock server instance, an HTTP mock server will
 automatically be created in the background of your tests. The local mock server is created
 in a separate thread. It will be started when your tests need one for the first time.
 It will be shut down at the end of the test run.

 To use this crate in standalone mode you can just use the binary or start it using cargo
 (`cargo run`).

 # Getting Started
 You can use a local mock server in your tests like shown in the following:
 ```rust
 extern crate httpmock;

 use httpmock::Method::GET;
 use httpmock::{mock, with_mock_server};

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
 In the snippet, a mock server is automatically created when the test launches. This is ensured
 by the `httpmock::with_mock_server`
 annotation, which wraps the test inside an initializer function performing multiple
 preparation steps, such as starting a server (if none yet exists) or clearing the server
 from old mocks. It also sequentializes tests that involve a mock server.

 If you try to create a mock without having annotated you test function
 with the `httpmock::with_mock_server` annotation,
 you will receive a panic at runtime pointing you to this problem.
 You can provide expected request attributes (such as headers, body content, etc.)
 and values that will be returned by the mock to the calling application using the
 `expect_xxx` and `return_xxx` methods, respectively. The
 `Mock::create` method will eventually make a request to the
 mock server (either local or remote) to create the mock at the server.

 You can use the mock object returned by the `Mock::create`
 method to fetch information about it from the mock server. This might be the number of
 times this mock has been called. You might use this information in your test assertions.

 An HTTP request made by your application is only considered to match a mock if the request
 fulfills all specified requirements. If a request does not match any mock, the mock server will
 respond with an empty response body and a status code 500 (Internal Server Error).

 By default, if a server port is not provided using an environment variable (`MOCHA_SERVER_PORT`),
 the port 5000 will be used. If another server host is explicitely set
 using an environment variable (`MOCHA_SERVER_HOST`), then this API will use the remote server
 managing mocks.

 # Examples
 Please refer to the
 [tests of this crate](https://github.com/alexliesenfeld/httpmock/blob/master/tests/integration_tests.rs )
 for more examples.

 # Debugging
 `httpmock` logs against the `log` crate. If you use the `env_logger` backend, you can activate
 debug logging by setting `RUST_LOG` environment variable to `debug` and then calling
 `env_logger::try_init()`:
 ```rust
 #[test]
 fn your_test() {
     let _ = env_logger::try_init();
     // ...
 }
 ```
  
 
 ## License
 `httpmock` is free software: you can redistribute it and/or modify it under the terms of the MIT Public License.
 
 This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the MIT Public License for more details.