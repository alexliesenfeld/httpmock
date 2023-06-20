# Getting Started
This crate in Rust allows you to create mocks for HTTP requests and responses.
Follow the steps below to create mocks using this crate:

1. Add the httpmock crate as a dependency in your Cargo.toml file:
```toml
[dependencies]
httpmock = "0.6"
```
2. Import the necessary modules in your Rust code:
```rust
use httpmock::{MockServer, Method, Mock, MockRef, Regex};
```
3. Create a MockServer instance. This server will handle all the mock requests and responses:
```rust
let server = MockServer::start();
```
4. Define a mock by creating a Mock instance. You can specify the HTTP method, URL pattern,
request headers, request body, and response details:
```rust
let mock = Mock::new()
    .expect_method(Method::GET)
    .expect_path("/api/users")
    .expect_query_param("page", "1")
    .return_status(200)
    .return_header("Content-Type", "application/json")
    .return_body("{\"message\": \"Mock response\"}")
    .create_on(&server);
```
In this example, the mock is set up to expect a GET request with the URL pattern "/api/users" and
the query parameter "page" with the value "1". It will return a 200 status code, set the "Content-Type"
header to "application/json", and respond with a JSON body.
5. Make requests to the mock server using the reqwest crate or any other HTTP client library:
```rust
let url = server.url("/api/users?page=1");
let response = reqwest::blocking::get(&url).unwrap();
```
Here, we're using the reqwest crate to send a GET request to the mock server. The response will be the one defined in the mock.
6. Verify the mock was called and perform assertions on it if needed:
```rust
assert_eq!(mock.times_called(), 1);
```
You can check the number of times the mock was called using the times_called method and perform assertions on it or other properties.
Remember to clean up the mock server after you're done by calling server.stop() to avoid port conflicts.
That's how you can create mocks using the httpmock crate in Rust. 

# Full Code Files

### `Cargo.toml`
```toml
[package]
name = "my-httpmock-project"
version = "0.1.0"
edition = "2021"

[dev-dependencies]
httpmock = "0.6"
reqwest = "0.11"
```



### test.rs
```rust
use httpmock::{MockServer, Method, Mock, Regex};
use reqwest::blocking::get;

#[test]
fn test_mocked_request() {
    // Create a MockServer instance
    let server = MockServer::start();

    // Define a mock
    let mock = Mock::new()
        .expect_method(Method::GET)
        .expect_path("/api/users")
        .expect_query_param("page", "1")
        .return_status(200)
        .return_header("Content-Type", "application/json")
        .return_body("{\"message\": \"Mock response\"}")
        .create_on(&server);

    // Send a request to the mock server
    let url = server.url("/api/users?page=1");
    let response = get(&url).unwrap();

    // Verify the mock was called and perform assertions
    assert_eq!(mock.times_called(), 1);
    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(
        response.headers().get("Content-Type").unwrap(),
        "application/json"
    );
    assert_eq!(response.text().unwrap(), "{\"message\": \"Mock response\"}");
}
```

