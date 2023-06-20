# Standalone Mode
Standalone mode in the httpmock library allows you to run a mock server independently, outside of the context of a specific test function. This mode enables the mock server to receive HTTP requests from various applications or services, not just the test functions where a mock server is typically created.

Here are the reasons why you might need standalone mode:

* **Integration testing**: Standalone mode is particularly useful for integration testing scenarios where you need to simulate the behavior of external services or APIs that your application interacts with. By running the mock server independently, you can accurately mimic the responses and behavior of these external services, enabling comprehensive integration testing.
* **API development and debugging**: During API development or debugging, standalone mode allows you to quickly set up a mock server that emulates the behavior of a real API. This allows you to test your application's interactions with the API and verify its functionality without relying on the actual API. It provides a controlled environment for iterating and refining your API implementation.
* **Microservices and distributed systems**: In a microservices or distributed system architecture, standalone mode enables you to mock individual services or components independently. Each microservice can run its own mock server, facilitating isolated testing of individual services without requiring the entire system to be running. This enhances the agility and efficiency of testing in distributed environments.
* **Service virtualization**: Standalone mode can be used for service virtualization, where you simulate the behavior of a third-party service that may not be readily available or accessible during development or testing. By running the mock server independently, you can replicate the responses and behavior of the actual service, allowing your application to interact with it as if it were live.
* **Load testing**: Standalone mode can also be leveraged for load testing scenarios. By running multiple instances of the mock server, you can simulate high volumes of requests and observe how your application handles the load. This helps identify potential bottlenecks or performance issues before deploying to a production environment.

In summary, standalone mode provides the flexibility to run a mock server independently, catering to a wider range of use cases beyond traditional test functions. It enables integration testing, API development and debugging, testing in microservices or distributed systems, service virtualization, and load testing. By decoupling the mock server from specific test functions, standalone mode empowers you to simulate and test interactions with various applications and services in a controlled and efficient manner.

## What are the changes that are required to use the standalone mode?
The only difference when using the standalone mode compare to library usage is how the connection to the mock server is
established. Instead of using the `MockServer::start` function to create a new mock server instance, you just need to 
use the `MockServer::connect` function to connect to your httpmock server.


## How Do I Start httpmock in Standalone Mode?

There is a [Docker image](start_standalone_server.md) that you can use to start your own httpmock server in standalone mode. 
You can then let your tests connect to this server (see example below). There is also the possibility to and run your own standalone httpmock 
binary (see [here](start_standalone_server.md)).

## Example test using Standalone Mode
```rust
#[test]
fn standalone_test() {
    // Arrange

    // This starts up a standalone server in the background running on port 5000
    simulate_standalone_server();

    // Instead of creating a new MockServer using new(), we connect to an existing remote instance.
    let server = MockServer::connect("localhost:5000");

    let search_mock = server.mock(|when, then| {
        when.path("/search").body("wow so large".repeat(1000000));
        then.status(202);
    });

    // Act: Send the HTTP request
    let response = Request::post(server.url("/search"))
        .body("wow so large".repeat(1000000))
        .unwrap()
        .send()
        .unwrap();

    // Assert
    search_mock.assert();
    assert_eq!(response.status(), 202);
}

```

## Detailed Instructions
1. Start the standalone mock server by either using a [Docker image](start_standalone_server.md) or [running a httpmock standalone binary](start_standalone_server.md).
2. Connect to the standalone server:
    * Instead of starting a new `MockServer` instance using `start()`, you can connect to the existing standalone server. 
    * You can use `MockServer::connect("localhost:5000")` method to establish a connection with the standalone server.
3. Define mock behavior:
    * Use the `mock()` or `mock_async()` methods of the `MockServer` instance to define the expected behavior for incoming HTTP requests.
    * The when closure allows you to specify conditions such as the request path, query parameters, or headers.
    * The then closure defines the response behavior, including the desired HTTP status code, response body, or headers.
4. Send HTTP requests to the standalone server
5. Assert and validate:
    * After sending the HTTP request, you can perform assertions to [verify that the expected behavior](../library_usage/getting_started.md) was triggered.
    * Use the [assert methods](../library_usage/getting_started.md) to validate the response status code, headers, or body.
    * You can also use the assert methods on the mock instance to ensure that the mock expectations were met.

Note: In the provided code, there are additional examples of using the standalone mode, 
[limitations and unsupported features](standalone_limitations.md). To gain a better understanding and cater to
your specific requirements, you can refer to the tests conducted by 
[httpmocks own tests](https://github.com/alexliesenfeld/httpmock/tree/master/tests). These tests showcase various 
scenarios that can be modified and applied according to your needs.

By following these steps, you can effectively utilize the standalone mode in the httpmock library to mock and test
HTTP requests in isolation.