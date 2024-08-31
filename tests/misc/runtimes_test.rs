use crate::with_standalone_server;
use httpmock::prelude::*;
use reqwest::Client;

#[test]
fn all_runtimes_test() {
    return; // TODO: This needs to be fixed. New HTTP client requires tokio runtime!

    with_standalone_server();

    // Tokio
    assert_eq!(
        tokio::runtime::Runtime::new().unwrap().block_on(test_fn()),
        202
    );

    // Actix
    assert_eq!(actix_rt::Runtime::new().unwrap().block_on(test_fn()), 202);

    // async_std
    assert_eq!(async_std::task::block_on(test_fn()), 202);
}

async fn test_fn() -> u16 {
    // We will create this mock server to simulate a real service (e.g., GitHub, AWS, etc.).
    let server3 = MockServer::start_async().await;
    server3
        .mock_async(|when, then| {
            when.any_request();
            then.status(202).body("Hi from fake GitHub!");
        })
        .await;

    let server2 = MockServer::connect_async("localhost:5050").await;
    server2
        .forward_to_async(server3.base_url(), |rule| {
            rule.filter(|when| {
                when.any_request(); // We want all requests to be proxied.
            });
        })
        .await;

    // Let's create our mock server for the test
    let server1 = MockServer::start_async().await;

    // We configure our server to proxy the request to the target host instead of
    // answering with a mocked response. The 'when' variable lets you configure
    // rules under which requests are proxied.
    server1
        .proxy_async(|rule| {
            rule.filter(|when| {
                when.any_request(); // We want all requests to be proxied.
            });
        })
        .await;

    // The following will send a request to the mock server. The request will be forwarded
    // to the target host, as we configured before.
    let client = Client::builder()
        .proxy(reqwest::Proxy::all(server1.base_url()).unwrap()) // Configure to use a proxy server
        .build()
        .unwrap();

    // Since the request was forwarded, we should see the target host's response.
    let response = client.get(server2.url("/get")).send().await.unwrap();
    let status_code = response.status().as_u16();

    assert_eq!("Hi from fake GitHub!", response.text().await.unwrap());
    assert_eq!(status_code, 202);

    status_code
}
