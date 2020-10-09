<div align="center">
<h1>httpmock</h1>
</div>

<p align="center">HTTP mocking library for Rust.</p>
<div align="center">
    
[![Build Status](https://dev.azure.com/alexliesenfeld/httpmock/_apis/build/status/alexliesenfeld.httpmock?branchName=master)](https://dev.azure.com/alexliesenfeld/httpmock/_build/latest?definitionId=2&branchName=master)
[![codecov](https://codecov.io/gh/alexliesenfeld/httpmock/branch/master/graph/badge.svg)](https://codecov.io/gh/alexliesenfeld/httpmock)
[![crates.io](https://img.shields.io/crates/d/httpmock.svg)](https://crates.io/crates/httpmock)
[![Rust](https://img.shields.io/badge/rust-1.43.0%2B-blue.svg?maxAge=3600)](https://github.com/rust-lang/regex)
[![Docker](https://img.shields.io/docker/cloud/build/alexliesenfeld/httpmock)](https://hub.docker.com/r/alexliesenfeld/httpmock)

</div>

<p align="center">
    <a href="https://docs.rs/httpmock/">Documentation</a>
    ·
    <a href="https://crates.io/crates/httpmock">Crate</a>
    ·
    <a href="https://github.com/alexliesenfeld/httpmock/issues">Report Bug</a>
    ·
    <a href="https://github.com/alexliesenfeld/httpmock/issues">Request Feature</a>
    ·
    <a href="https://github.com/alexliesenfeld/httpmock/blob/develop/RELEASES.md">Changelog</a>
</p>

## Features

* Simple, expressive, fluent API.
* Many built-in helpers for easy request matching.
* Parallel test execution.
* Extensible request matching.
* Fully asynchronous core with synchronous and asynchronous APIs.
* Debugging support.
* Network delay simulation.
* Standalone mode with an accompanying [Docker image](https://hub.docker.com/r/alexliesenfeld/httpmock).
* Support for [Regex](https://docs.rs/regex/) matching, JSON, [serde](https://crates.io/crates/serde), cookies, and more.


## Getting Started
Add `httpmock` to `Cargo.toml`:

```toml
[dev-dependencies]
httpmock = "0.5.0-beta.1"
```
You can then use `httpmock` as follows:
```rust
use httpmock::MockServer;
use httpmock::Method::GET;

// Start a lightweight mock server.
let server = MockServer::start();

// Create a mock on the server.
let hello_mock = server.mock(|when, then| {
    when.method(GET)
        .path("/translate")
        .query_param("word", "hello");
    then.status(200)
        .header("Content-Type", "text/html; charset=UTF-8")
        .body("Привет");
});

// Send an HTTP request to the mock server. This simulates your code.
let response = isahc::get(server.url("/translate?word=hello")).unwrap();

// Ensure the mock server did respond as specified.
assert_eq!(response.status(), 200);
// Ensure the specified mock was called exactly one time.
assert_eq!(hello_mock.hits(), 1);
```

The above example will spin up a lightweight HTTP mock server and configure it to respond to all `GET` requests 
to path `/translate` with query parameter `word=hello`. The corresponding HTTP response will contain the text body 
`Привет`.

# Usage
See the [reference docs](https://docs.rs/httpmock/) for detailed API documentation.

## Examples
You can find examples in the 
[`httpmock` test directory](https://github.com/alexliesenfeld/httpmock/blob/master/tests/). 
The [reference docs](https://docs.rs/httpmock/) also contain _**a lot**_ of examples. 

## License
`httpmock` is free software: you can redistribute it and/or modify it under the terms of the MIT Public License.
 
This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied 
warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the MIT Public License for more details.
