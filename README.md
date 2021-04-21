<div align="center">
<h1>httpmock</h1>
</div>

<p align="center">HTTP mocking library for Rust.</p>
<div align="center">
    
[![Build Status](https://dev.azure.com/alexliesenfeld/httpmock/_apis/build/status/alexliesenfeld.httpmock?branchName=master)](https://dev.azure.com/alexliesenfeld/httpmock/_build/latest?definitionId=2&branchName=master)
[![codecov](https://codecov.io/gh/alexliesenfeld/httpmock/branch/master/graph/badge.svg)](https://codecov.io/gh/alexliesenfeld/httpmock)
[![crates.io](https://img.shields.io/crates/d/httpmock.svg)](https://crates.io/crates/httpmock)
[![Rust](https://img.shields.io/badge/rust-1.46%2B-blue.svg?maxAge=3600)](https://github.com/rust-lang/rust/blob/master/RELEASES.md#version-1460-2020-08-27)
[![Mentioned in Awesome](https://camo.githubusercontent.com/e5d3197f63169393ee5695f496402136b412d5e3b1d77dc5aa80805fdd5e7edb/68747470733a2f2f617765736f6d652e72652f6d656e74696f6e65642d62616467652e737667)](https://github.com/rust-unofficial/awesome-rust#testing)
	
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
    <a href="https://github.com/alexliesenfeld/httpmock/blob/master/CHANGELOG.md">Changelog</a>
</p>

## Features

* Simple, expressive, fluent API.
* Many built-in helpers for easy request matching.
* Parallel test execution.
* Extensible request matching.
* Fully asynchronous core with synchronous and asynchronous APIs.
* [Advanced verification and debugging support](https://dev.to/alexliesenfeld/rust-http-testing-with-httpmock-2mi0#verification).
* [Network delay simulation](https://github.com/alexliesenfeld/httpmock/blob/master/tests/examples/delay_tests.rs).
* Support for [Regex](https://docs.rs/regex/) matching, JSON, [serde](https://crates.io/crates/serde), cookies, and more.
* Standalone mode with an accompanying [Docker image](https://hub.docker.com/r/alexliesenfeld/httpmock).
* Support for [mock specification based on YAML files](https://github.com/alexliesenfeld/httpmock/tree/develop#file-based-mock-specification).

## Getting Started
Add `httpmock` to `Cargo.toml`:

```toml
[dev-dependencies]
httpmock = "0.5"
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

// Ensure the specified mock was called exactly one time (or fail with a detailed error description).
hello_mock.assert();
// Ensure the mock server did respond as specified.
assert_eq!(response.status(), 200);
```

The above example will spin up a lightweight HTTP mock server and configure it to respond to all `GET` requests 
to path `/translate` with query parameter `word=hello`. The corresponding HTTP response will contain the text body 
`Привет`.

# Usage
See the [reference docs](https://docs.rs/httpmock/) for detailed API documentation.

## Examples
You can find examples in the 
[`httpmock` test directory](https://github.com/alexliesenfeld/httpmock/blob/master/tests/). 
The [reference docs](https://docs.rs/httpmock/) also contain _**a lot**_ of examples. There is an [online tutorial](https://dev.to/alexliesenfeld/rust-http-testing-with-httpmock-2mi0) as well. 

## Standalone Mock Server
You can use `httpmock` to run a standalone mock server that is executed in a separate process. There is a 
[Docker image](https://hub.docker.com/r/alexliesenfeld/httpmock) available at Dockerhub to get started quickly.

The standalone mode allows you to mock HTTP based APIs for many API clients, not only the ones 
inside your Rust tests, but also completely different programs running on remote hosts. 
This is especially useful if you want to use `httpmock` in system or end-to-end tests that require mocked services 
(such as REST APIs, data stores, authentication providers, etc.).

Please refer to [the docs](https://docs.rs/httpmock/0.5.8/httpmock/#standalone-mode) for more information

### File Based Mock Specification
For convenience, the standalone mode also allows you to use YAML files for mock specification, so you do not need to
use Rust or any other programming language at all. The mock specification file schema is very similar to the `httpmock` 
Rust API, so it's easy to jump between the two. Please find an example mock specification file 
[here](https://github.com/alexliesenfeld/httpmock/blob/master/tests/resources/static_yaml_mock.yaml). 

Please refer to [the docs](https://github.com/alexliesenfeld/httpmock/blob/master/src/lib.rs#L185-L201) 
for more information.

## License
`httpmock` is free software: you can redistribute it and/or modify it under the terms of the MIT Public License.
 
This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied 
warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the MIT Public License for more details.
