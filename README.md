<p align="center"><img height="150" src="https://raw.githubusercontent.com/alexliesenfeld/httpmock/master/banner.png"></p>
<p align="center"><b>HTTP mocking library for Rust</b></p>
<div align="center">
    
[![Build Status](https://dev.azure.com/alexliesenfeld/httpmock/_apis/build/status/alexliesenfeld.httpmock?branchName=master)](https://dev.azure.com/alexliesenfeld/httpmock/_build/latest?definitionId=2&branchName=multiserver)
[![Coverage](https://codecov.io/github/alexliesenfeld/httpmock/coverage.svg?branch=multiserver)](https://crates.io/crates/httpmock)
[![License](https://img.shields.io/github/license/alexliesenfeld/httpmock.svg)](LICENSE)
[![crates.io](https://img.shields.io/crates/v/httpmock.svg)](https://cloud.docker.com/repository/docker/dessalines/lemmy/)
[![docs.rs status](https://docs.rs/httpmock/badge.svg)](https://docs.rs/httpmock)
[![docker pulls](https://img.shields.io/docker/pulls/alexliesenfeld/httpmock.svg)](https://cloud.docker.com/repository/docker/alexliesenfeld/httpmock/)

</div>

<p align="center">
    <a href="https://docs.rs/httpmock/">Documentation</a>
    ·
    <a href="https://github.com/alexliesenfeld/httpmock/issues">Report Bug</a>
    ·
    <a href="https://github.com/alexliesenfeld/httpmock/issues">Request Feature</a>
</p>

---
`httpmock` is a Rust crate that allows you to mock HTTP responses in your tests. It contains two major components:

* a **mock server** that is automatically started in the background of your tests, and
* a **test library** to create HTTP mocks on the server.

All interaction with the mock server happens through the provided library. Therefore, you do
not need to interact with the mock server directly.

By default, an HTTP mock server instance will be started in the background of
your tests. It will be created when your tests need the mock server for the first
time and will be shut down at the end of the test run. The mock server is executed in a
separate thread, so it does not conflict with your tests.

The mock server can also be started in **standalone mode** (more information below).

# Getting Started
Add `httpmock` to `Cargo.toml`:

```toml
[dev-dependencies]
httpmock = "0.3.5"
```

You can then use `httpmock` in your tests like shown in the following example:
```rust
extern crate httpmock;

use httpmock::Method::GET;
use httpmock::{mock, with_mock_server};

#[test]
#[with_mock_server]
fn simple_test() {
   let search_mock = mock(GET, "/search")
       .expect_query_param("query", "metallica")
       .return_status(204)
       .create();

   let response = reqwest::get("http://localhost:5000/search?query=metallica").unwrap();

   assert_eq!(response.status(), 204);
   assert_eq!(search_mock.times_called(), 1);
}
```
In the above example, a mock server is automatically created when the test launches.
This is ensured by the `with_mock_server`
annotation. It wraps the test with an initializer function that is performing several important
preparation steps, such as starting the mock server if none yet exists
and cleaning up old mock server state, so that each test can start with
a clean server. The annotation also sequentializes tests, so
they do not conflict with each other when using the mock server.

If you try to create a mock without having annotated your test function
with the `with_mock_server` annotation,
you will receive a panic at runtime pointing you to this problem.

# Usage
Interaction with the mock server happens via the `Mock` structure.
It provides you all mocking functionality that is supported by the mock server.

The expected style of usage is as follows:
* Create a `Mock` object using the
`Mock::create` method
(or `Mock::new` for slightly more control).
* Set your mock requirements using the provided `expect`-methods, such as `expect_header`, `expect_body`, etc. These
methods describe what attributes an HTTP request needs to have to be considered a "match" for the mock you are creating.
* use the provided `return`-methods to describe what the mock server should return when it receives
an HTTP request that matches all mock requirements. Some example `return`-methods are `return_status` and `return_body`.
If the server does not find any matching mocks for an incoming HTTP request, it will return a response with an empty
body and HTTP status code 500.
* create the mock using the `Mock::create` method. If you do
not call this method when you are finished configuring it, it will not be created at the mock
server and your test will not receive the expected response.
* using the mock object returned by the `Mock::create` method
to assert that a mock has been called by your code under test (please refer to any example).

# Responses
For any HTTP request sent to the mock server by your application, the request is only
considered to match a mock if it fulfills all of the mocks request requirements.
If a request does not match any mock, the server will respond with an empty response body
and an HTTP status code 500 (Internal Server Error).

# Examples
Fore more examples, please refer to
[this crates test directory](https://github.com/alexliesenfeld/httpmock/blob/master/tests/integration_tests.rs ).

# Debugging
`httpmock` logs against the `log` crate. For example, if you use the `env_logger` logging backend, you can activate
debug logging by setting `RUST_LOG` environment variable to `debug` and then calling
`env_logger::try_init()`:
```rust
#[test]
#[with_mock_server]
fn your_test() {
    let _ = env_logger::try_init();
    // ...
}
```

# Standalone Mode
You can use `httpmock` to provide a standalone mock server that is available to multiple
applications. This can be useful if you are running integration tests that involve
multiple applications and you want to mock only a subset of them.

To activate standalone mode, you need to do the following steps:
* Start the mock server in standalone mode by running `cargo run --features="standalone" --release` from the sources
(or by using a binary that you can build with `cargo build --features="standalone" --release`).
* On the host that is executing the tests, provide a host name by setting the environment variable
`HTTPMOCK_HOST`. If set, tests are assuming a mock server is being executed elsewhere,
so no local mock server will be started for your tests anymore. Instead, this library will be using
the remote server to create mocks.

By default, if a server port is not provided by the environment variable
`HTTPMOCK_PORT`, port `5000` will be used.

## Exposing the mock server to the network
If you want to expose the server to machines other than localhost, you need to provide the
`--expose` parameter:
* using cargo: `cargo run --features="standalone" --release -- --expose`
* using the binary: `httpmock --expose`

## Docker container
As an alternative to building the mock server yourself, you can use the Docker image from
the sources to run a mock server in standalone mode:
```shell
docker build -t httpmock .
docker run -it --rm -p 5000:5000 --name httpmock httpmock
```

To enable extended logging, you can run the docker container with the `RUST_LOG` environment
variable set to the log level of your choice:
```shell
docker run -it --rm -e RUST_LOG=httpmock=debug -p 5000:5000 --name httpmock httpmock
```
Please refer to the [log](https://docs.rs/crate/log) and [env_logger](https://docs.rs/crate/env_logger) crates
for more information about logging.

# License
`httpmock` is free software: you can redistribute it and/or modify it under the terms of the MIT Public License.
 
This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied 
warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the MIT Public License for more details.
