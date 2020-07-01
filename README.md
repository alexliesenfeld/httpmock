<p align="center"><img height="150" src="https://raw.githubusercontent.com/alexliesenfeld/httpmock/master/banner.png"></p>
<p align="center"><b>HTTP mocking library for Rust</b></p>
<div align="center">
    
[![Build Status](https://dev.azure.com/alexliesenfeld/httpmock/_apis/build/status/alexliesenfeld.httpmock?branchName=multiserver)](https://dev.azure.com/alexliesenfeld/httpmock/_build/latest?definitionId=2&branchName=multiserver)
[![Coverage](https://codecov.io/github/alexliesenfeld/httpmock/coverage.svg?branch=multiserver)](https://codecov.io/gh/alexliesenfeld/httpmock/)
[![crates.io](https://img.shields.io/crates/d/httpmock.svg)](https://docs.rs/httpmock)
[![crates.io](https://img.shields.io/docker/cloud/build/alexliesenfeld/httpmock)](https://hub.docker.com/r/alexliesenfeld/httpmock)
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

---

`httpmock` is a Rust library that allows you to mock HTTP services in your tests. 

## Features

* Provides a full-blown HTTP mock server with HTTP/1.1 and HTTP/2 support.
* Fully asynchronous core with a synchornous and asynchronous API.
* Support for all major asynchronous executors and runtimes.
* Wide range of built-in request matchers and support for custom request matchers.
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

// ...

#[test]
fn simple_test() {
    // Arrange: Create a mock on a test local mock server 
    let mock_server = MockServer::start();

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

## API Usage

Each test usually creates its own local `MockServer` that runs a lightweight HTTP server by using `MockServer::new()`. Each local `MockServer` runs on its own random port so that tests do not conflict with each other.

You can use the `Mock` structure to specify and create a mock on the `MockServer`. The `Mock` structure provides you all supported mocking functionality.

### Request Matching and HTTP Responses
Other than many other libraries, `httpmock` does not require you to learn a DSL-like API to specify `Mock` behaviour. Instead, `httpmock` provides you a fluent builder-like API that clearly separates request matching and response attributes by using the following naming scheme:

- All methods starting with `expect` place a requirement on the HTTP request (e.g. `expect_method`, `expect_path`, or `expect_body`).
- All methods starting with `return` define what the mock server will return in response to a matching HTTP request (e.g. `return_status`, `return_body`, etc.).  

This way, users can benefit from IDE autocompletion to find request matchers and response attributes without even looking into documentation. 

An HTTP request is only considered to match a mock if it matches all of the mocks request requirements. If a request does not match at least one mock, the server will respond with an error message and HTTP status code 404 (Not Found).

### Sync / Async
Note that the blocking API (as presented in the `Getting Started` section) can be used in both, a synchronous and an asynchronous environment. Usually this should be the preferred style of using `httpmock` because it keeps tests simple and you don't need to change the style of usage when switching from a synchronous to an asynchronous environment or vice versa. If you absolutely need to schedule awaiting operations manually, then there are `async` counterparts for every potentially blocking operation that you can use (e.g.: `MockServer::start_async().await`, or `Mock::new().create_on_async(&mock_server).await`). 

## Parallelism
To balance execution speed and resource consumption, `MockServer`s are kept in a server pool internally. This allows to run multiple tests in parallel without overwhelming the executing machine by creating too many HTTP servers. A test will be blocked if it tries to use a `MockServer` (e.g. by calling `MockServer::new()`) while the server pool is empty (i.e. all servers are occupied by other tests). To avoid TCP port binding issues, `MockServers` are never recreated but recycled/resetted. The pool is filled on demand up to a predefined maximum number of 20 servers. You can change this number by setting the environment variable `HTTPMOCK_MAX_SERVERS`. 


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

### Examples
Fore more examples on how to use a remote server, please refer to
[this crates test directory](https://github.com/alexliesenfeld/httpmock/blob/master/tests/standalone_tests.rs ).

## License
`httpmock` is free software: you can redistribute it and/or modify it under the terms of the MIT Public License.
 
This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied 
warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the MIT Public License for more details.
