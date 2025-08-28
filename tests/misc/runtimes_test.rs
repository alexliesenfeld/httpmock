
#[test]
fn all_runtimes_test() {
    use crate::with_standalone_server;

    with_standalone_server();

    // Tokio
    assert_eq!(
        tokio::runtime::Runtime::new().unwrap().block_on(test_fn()),
        202
    );

    // Actix
    assert_eq!(actix_rt::Runtime::new().unwrap().block_on(test_fn()), 202);

    // async_std
    assert_eq!(smol::block_on(test_fn()), 202);
}

#[cfg(all(feature = "proxy", feature = "remote"))]
async fn test_fn() -> u16 {
    use httpmock::prelude::*;
    use isahc::{http::Uri, prelude::*, HttpClient};

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
    // Build client with proxy
    let proxy_server_uri: Uri = server1.base_url().parse().unwrap();

    let client = HttpClient::builder()
        .proxy(proxy_server_uri)
        .build()
        .unwrap();

    // Forwarded request
    let mut res = client.get_async("https://httpbin.org/ip").await.unwrap();
    println!("{}", res.text().await.unwrap());

    // Request to server2
    let mut response = client.get_async(server2.url("/get")).await.unwrap();
    let status_code = response.status().as_u16();

    assert_eq!("Hi from fake GitHub!", response.text().await.unwrap());
    assert_eq!(status_code, 202);

    status_code
}

#[cfg(feature = "remote")]
#[cfg(not(feature = "proxy"))]
async fn test_fn() -> u16 {
    use httpmock::prelude::*;
    use isahc::HttpClient;

    let server = MockServer::connect_async("localhost:5050").await;
    let mock = server.mock_async(|when, then| {
        when.path("/get");
        then.status(202);
    }).await;

    let client = HttpClient::new().unwrap();
    let response = client.get_async(server.url("/get")).await.unwrap();

    mock.assert_async().await;

    response.status().as_u16()
}

#[cfg(not(any(feature = "proxy", feature = "remote")))]
async fn test_fn() -> u16 {
    use httpmock::prelude::*;
    use isahc::HttpClient;

    let server = MockServer::start_async().await;
    let mock = server.mock_async(|when, then| {
        when.path("/get");
        then.status(202);
    }).await;

    let client = HttpClient::new().unwrap();
    let response = client.get_async(server.url("/get")).await.unwrap();

    mock.assert_async().await;

    response.status().as_u16()
}
