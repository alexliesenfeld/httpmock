# HTTP mock server 
`mocha` is a simple and efficient HTTP mock server that can be used for local tests as
well as tests that span multiple systems. It provides a local (or remote) mock server and
 a library to create, verify and remove HTTP mocks.

 ## Usage
 If used without a dedicated (standalone) mock server instance, an HTTP mock server will
 automatically be created in the background of your tests. The local mock server is created
 in a separate thread that will be started when a test needs a mock server for the first time.
 It will be shut down at the end of the test run.

 Should you need to extend or change your tests to span multiple systems later (such as in
 system integration tests), you can switch the tests to use a standalone mock server by simply
 setting the address of the remote server using an environment variable. This way the remote
 server will be used for mocking and your mocks will be available to all participating systems.
 A standalone version of the HTTP mock server is available as an executable binary or a Docker
 image.

 ## Getting Started
 You can use a local mock server in your tests like shown in the following:
 ```rust
 extern crate mocha;

 use mocha::mock;
 use mocha::Method::GET;

 #[test]
 fn simple_test() {
    // Arrange the test by creating a mock
    let health_mock = mock(GET, "/health")
       .return_status(200)
       .return_header("Content-Type", "application/text")
       .return_header("X-Version", "0.0.1")
       .return_body("OK")
       .create();

    // Act (simulates your code)
    let response = reqwest::get("http://localhost:5000/health").unwrap();

    // Make some assertions
    assert_eq!(response.status(), 200);
    assert_eq!(health_mock.number_of_calls().unwrap(), 1);
 }
 ```
 As shown in the code snippet, a mock server is automatically created when the `mock` function
 is called. You can provide expected request attributes (such as headers, body content, etc.)
 and values that will be returned by the mock to the calling application using the
 `expect_xxx` and `return_xxx` methods, respectively. The `create` method will eventually
 make a request to the mock server (either local or remote) to create the mock at the server.

 You can use the mock object returned by the `create` method to fetch information about
 the mock from the mock server, such as the number of times this mock has been called.
 This object is useful for test assertions.

 A request is only considered to match a mock if the request contains all attributes required
 by the mock. If a request does not match any mock previously created, the mock server will
 respond with an empty response body and a status code `500 (Internal Server Error)`.
 
 ## License
 `mocha` is free software: you can redistribute it and/or modify it under the terms of the MIT Public License.
 
 This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the MIT Public License for more details.