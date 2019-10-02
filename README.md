<p align="center"><img height="150" src="https://raw.githubusercontent.com/alexliesenfeld/httpmock/master/banner.png"></p>
<p align="center">
    <a href="https://crates.io/crates/httpmock"><img src="https://img.shields.io/crates/v/httpmock.svg"></a>
    <a href="https://docs.rs/httpmock"><img src="https://docs.rs/httpmock/badge.svg"></a>
</p>
<p align="center"><b>HTTP mocking library for Rust</b></p>

---
`httpmock` is an easy-to-use library that allows you to mock HTTP endpoints in your tests.

This crate contains two major components:

* a **mock server** that is automatically started in the background of your tests, and
* a **test library** to create HTTP mocks on the server.

All interaction with the mock server happens through the provided library. Therefore, you do
not need to interact with the mock server directly (but you certainly can!).

By default, an HTTP mock server instance will be started in the background of
your tests. It will be created when your tests need the mock server for the first
time and will be shut down at the end of the test run. The mock server is executed in a
separate thread, so it does not conflict with your tests.

The mock server can also be started in **standalone mode** (more information below).

# Getting Started
Add `httpmock` to `Cargo.toml`:

```toml
[dev-dependencies]
httpmock = "0.3.0
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
and cleaning up the mock server state, so that each test can start with
a clean mock server. The annotation also sequentializes tests that are marked with it, so
they do not conflict with each other when using the mock server.

If you try to create a mock without having annotated your test function
with the `with_mock_server` annotation,
you will receive a panic at runtime pointing you to this problem.

# Usage
The main point of interaction with the mock server happens via `Mock`.
It provides you all mocking functionality that is supported by the mock server.

The expected style of usage is to
* create a `Mock` object using the
`Mock::create` method
(or `Mock::new` for slightly more control)
* Set all your mock requirements using `expect_xxx`-methods, such as headers, body content, etc.
These methods describe what attributes an HTTP request needs to have to be considered a
"match" for the mock you are creating.
* use `return_xxx`-methods to describe what the mock server should return when it receives
an HTTP request that matches the mock. If the server does not find any matching mocks for an
HTTP request, it will return a response with an empty body and an HTTP status code 500.
* create the mock using the `Mock::create` method. If you do
not call this method when you complete configuring it, it will not be created at the mock
server and your test will not receive the expected response.
* using the mock object returned by by the `Mock::create` method
to assert that a mock has been called by your code under test (please refer to any example).

# Responses
An HTTP request made by your application is only considered to match a mock if the request
fulfills all specified mock requirements. If a request does not match any mock currently stored
on the mock server, it will respond with an empty response body and an HTTP status code 500
(Internal Server Error).

# Examples
Fore more examples, please refer to
[this crates test directory](https://github.com/alexliesenfeld/httpmock/blob/master/tests/integration_tests.rs ).

# Debugging
`httpmock` logs against the `log` crate. If you use the `env_logger` backend, you can activate
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
You can use this crate to provide both, an HTTP mock server for your local tests,
but also a standalone mock server that is reachable for other applications as well. This can be
useful if you are running integration tests that span multiple applications.

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
