<div align="center">
<img height="120" src="https://raw.githubusercontent.com/alexliesenfeld/httpmock/master/banner.png">
<h1>httpmock</h1>
</div>

<p align="center">HTTP mocking library for Rust.</p>
<div align="center">
    
[![Build Status](https://dev.azure.com/alexliesenfeld/httpmock/_apis/build/status/alexliesenfeld.httpmock?branchName=master)](https://dev.azure.com/alexliesenfeld/httpmock/_build/latest?definitionId=2&branchName=master)
[![Coverage](https://codecov.io/github/alexliesenfeld/httpmock/coverage.svg?branch=master)](https://codecov.io/gh/alexliesenfeld/httpmock/)
[![crates.io](https://img.shields.io/crates/d/httpmock.svg)](https://crates.io/crates/httpmock)
[![Docker](https://img.shields.io/docker/cloud/build/alexliesenfeld/httpmock)](https://hub.docker.com/r/alexliesenfeld/httpmock)
[![License](https://img.shields.io/github/license/alexliesenfeld/httpmock.svg)](LICENSE)
	
</div>

<p align="center">
    <a href="https://docs.rs/httpmock/">Documentation</a>
    ·
    <a href="https://crates.io/crates/httpmock">Crate</a>
    ·
    <a href="https://github.com/alexliesenfeld/httpmock/issues">Report Bug</a>
    ·
    <a href="https://github.com/alexliesenfeld/httpmock/issues">Request Feature</a>
</p>

## Features

* Provides a full-blown HTTP mock server with HTTP/1 and HTTP/2 support.
* Fully asynchronous core with a synchornous and asynchronous API.
* Compatible with all major asynchronous executors and runtimes.
* Built-in request matchers with support for custom request matchers.
* Parallel test execution by default.
* Standalone mode with an accompanying [Docker image](https://hub.docker.com/r/alexliesenfeld/httpmock).

## Getting Started
Add `httpmock` to `Cargo.toml`:

```toml
[dev-dependencies]
httpmock = "0.4.0"
```

You can then use `httpmock` in your tests like shown in the example below:
```rust
extern crate httpmock;

use httpmock::Method::{GET};
use httpmock::{Mock, MockServer, MockServerRequest, Regex};

#[test]
fn example_test() {
    // Arrange: Create a mock on a local mock server 
    let mock_server = MockServer::start();

    let search_mock = Mock::new()
        .expect_method(GET)         
        .expect_path("/search")
        .return_status(200)
        .create_on(&mock_server);

    // Act: Send an HTTP request to the mock server (simulates your software)
    let url = format!("http://{}/search", mock_server.address());
    let response = isahc::get(&url).unwrap();

    // Assert: Ensure there was a response from the mock server
    assert_eq!(response.status(), 200);
    assert_eq!(search_mock.times_called(), 1);
}
```

## API Usage

Each test usually creates its own local `MockServer` using `MockServer::start()`. This creates a lightweight HTTP
server that runs on its own random port. This way tests do not conflict with each other.

You can use the `Mock`  structure to specify and create mocks on the mock server. It provides you all supported mocking 
functionality.

### Request Matching and Responses
Other than many other libraries `httpmock` does not require you to learn a DSL-like API to
specify mock behaviour. Instead, `httpmock` provides you a fluent builder-like API that
clearly separates request matching and response attributes by using the following naming scheme:

- All `Mock` methods that start with `expect` in their name set a requirement
for HTTP requests (e.g. `Mock::expect_method`, `Mock::expect_path`, or `Mock::expect_body`).
- All `Mock` methods that start with `return` in their name define what the
mock server will return in response to an HTTP request that matched all mock requirements (e.g.
`Mock::return_status`, `Mock::return_body`, etc.).

With this naming scheme users can benefit from IDE autocompletion to find request matchers and
response attributes mostly without even looking into documentation.

If a request does not match at least one mock, the server will respond with
an error message and HTTP status code 404 (Not Found).

### Sync / Async

The internal implementation of `httpmock` is fully asynchronous. It provides you a synchronous and an asynchronous API 
though. If you want to schedule awaiting operations manually, then you can use the `async` variants that exist for every 
potentially blocking operation. For example, there is `MockServer::start_async` as an asynchronous 
counterpart to `MockServer::start` and `Mock::create_on_async` for `Mock::create_on`. 

## Parallelism
To balance execution speed and resource consumption, `MockServer`s are kept in a server pool internally. This allows to run multiple tests in parallel without overwhelming the executing machine by creating too many HTTP servers. A test will be blocked if it tries to use a `MockServer` (e.g. by calling `MockServer::new()`) while the server pool is empty (i.e. all servers are occupied by other tests). To avoid TCP port binding issues, `MockServers` are never recreated but recycled/resetted. The pool is filled on demand up to a predefined maximum number of 25 servers. You can change this number by setting the environment variable `HTTPMOCK_MAX_SERVERS`. 


## Examples
Fore more examples, please refer to
[this crates test directory](https://github.com/alexliesenfeld/httpmock/blob/master/tests/integration_tests.rs ).

## Debugging
`httpmock` logs against the `log` crate. For example, if you use the `env_logger` backend, you can activate debug logging by setting the `RUST_LOG` environment variable to `httpmock=debug`.

## Standalone Mode
You can use `httpmock` to run a standalone mock server that is available to multiple applications. This can be useful if you are running integration tests that involve both, real and mocked applications. 

### Docker
Altough you can build the mock server in standalone mode yourself, it is easiest to use the Docker image from the accompanying [Docker image](https://hub.docker.com/r/alexliesenfeld/httpmock). Please refer to the documentation on Docker repository. 

### API Usage
To be able to use a standalone server from your tests, you need to change how an instance of the `MockServer` structure is created. Instead of using `MockServer::new()`, you need to connect to a remote server by using one of the `connect` methods (such as `MockServer::connect("localhost:5000")` or `MockServer::connect_from_env()`). Therefore, tests that use a local mock server do only differ in one line of code from tests that use a remote server. Otherwise, both variants are identical. 

```Rust
#[test]
fn simple_test() {
    // Arrange: Create a mock on a test local mock server 
    let mock_server = MockServer::connect("some-host:5000");

    let search_mock = Mock::new()
        .expect_method(GET)         
        .expect_path("/search")
        .return_status(200)
        .create_on(&mock_server);

    // Act: Send an HTTP request to the mock server (simulates your software)
    let url = format!("http://{}/search", mock_server.address())).unwrap();
    let response = http_get(&url).unwrap();

    // Assert: Ensure there was a response from the mock server
    assert_eq!(response.status(), 200);
    assert_eq!(search_mock.times_called(), 1);
}
```

### Parallelism
Tests that use a remote mock server are executed sequentially by default. This is in contrast to tests that use a local mock server. Sequential execution is achieved by blocking all tests from further execution whenever a test requires to connect to a busy mock server. 

### Limitations
At this time, it is not possible to use custom request matchers in combination with remote
mock servers. It is planned to add this functionality in future though.

### Examples
Fore more examples on how to use a remote server, please refer to
[this crates test directory](https://github.com/alexliesenfeld/httpmock/blob/master/tests/standalone_tests.rs ).

## License
`httpmock` is free software: you can redistribute it and/or modify it under the terms of the MIT Public License.
 
This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied 
warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the MIT Public License for more details.
