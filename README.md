<div align="center">
<h1>httpmock</h1>
</div>

<p align="center">HTTP mocking library for Rust.</p>
<div align="center">
    
[![Build Status](https://dev.azure.com/alexliesenfeld/httpmock/_apis/build/status/alexliesenfeld.httpmock?branchName=master)](https://dev.azure.com/alexliesenfeld/httpmock/_build/latest?definitionId=2&branchName=master)
[![codecov](https://codecov.io/gh/alexliesenfeld/httpmock/branch/master/graph/badge.svg)](https://codecov.io/gh/alexliesenfeld/httpmock)
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
* A fully asynchronous core with synchronous and asynchronous APIs.
* Compatible with all major asynchronous executors and runtimes.
* Built-in request matchers with support for custom request matchers.
* Parallel test execution by default.
* A standalone mode with an accompanying [Docker image](https://hub.docker.com/r/alexliesenfeld/httpmock).

## Getting Started
Add `httpmock` to `Cargo.toml`:

```toml
[dev-dependencies]
httpmock = "0.4.2"
```

You can then use `httpmock` in your tests like shown in the example below:
```rust
extern crate httpmock;

use httpmock::Method::{GET};
use httpmock::{Mock, MockServer, MockServerRequest, Regex};

#[test]
fn example_test() {
    // Start a local mock server for exclusive use by this test function.
    let mock_server = MockServer::start();

    // Create a mock on the mock server. The mock will return HTTP status code 200 whenever
    // the mock server receives a GET-request with path "/hello".
    let hello_mock = Mock::new()
        .expect_method(GET)
        .expect_path("/hello")
        .return_status(200)
        .create_on(&mock_server);

    // Send an HTTP request to the mock server. This simulates your code.
    // The mock_server variable is being used to generate a mock server URL for path "/hello".
    let response = get(mock_server.url("/hello")).unwrap();

    // Ensure the mock server did respond as specified above.
    assert_eq!(response.status(), 200);
    // Ensure the specified mock responded exactly one time.
    assert_eq!(hello_mock.times_called(), 1);
}
```

## API Usage

Each test usually creates its own local `MockServer` using `MockServer::start()`. This creates a lightweight HTTP
server that runs on its own port. This way tests do not conflict with each other.

You can use the `Mock`  structure to specify and create mocks on the mock server. It provides you all supported mocking 
functionality.

### Request Matching and Responses
Other than many other libraries `httpmock` does not require you to learn a DSL-like API to
specify mock behaviour. Instead, `httpmock` provides you a fluent builder API that
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

## Examples
Fore more examples, please refer to
[this crates test directory](https://github.com/alexliesenfeld/httpmock/blob/master/tests ).

## Debugging
`httpmock` logs against the `log` crate. For example, if you use the `env_logger` backend, you can activate debug 
logging by setting the `RUST_LOG` environment variable to `httpmock=debug`. 

## Standalone Mode
You can use `httpmock` to run a standalone mock server that is available to multiple applications. This can be useful 
if you are running integration tests that involve both, real and mocked applications. 

### Docker
Although you can build the mock server in standalone mode yourself, it is easiest to use the Docker image 
from the accompanying [Docker image](https://hub.docker.com/r/alexliesenfeld/httpmock). Please refer to the 
documentation on Docker repository. 

## License
`httpmock` is free software: you can redistribute it and/or modify it under the terms of the MIT Public License.
 
This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied 
warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the MIT Public License for more details.
