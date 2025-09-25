use httpmock::prelude::*;
use reqwest::blocking::{Client, ClientBuilder};
use reqwest::redirect::Policy;

#[cfg(feature = "proxy")]
#[test]
fn proxy_test() {
    env_logger::try_init().unwrap();

    // We will create this mock server to simulate a real service (e.g., GitHub, AWS, etc.).
    let target_server = MockServer::start();
    target_server.mock(|when, then| {
        when.any_request();
        then.status(200).body("Hi from fake GitHub!");
    });

    // Let's create our mock server for the test
    let proxy_server = MockServer::start();

    // We configure our server to proxy the request to the target host instead of
    // answering with a mocked response. The 'when' variable lets you configure
    // rules under which requests are allowed to be proxied. If you do not restrict,
    // any request will be proxied.
    proxy_server.proxy(|rule| {
        rule.filter(|when| {
            // Here we only allow to proxy requests to our target server.
            when.host(target_server.host()).port(target_server.port());
        });
    });

    // The following will send a request to the mock server. The request will be forwarded
    // to the target host, as we configured before.
    let client = Client::builder()
        .proxy(reqwest::Proxy::all(proxy_server.base_url()).unwrap()) // <<- Here we configure to use a proxy server
        .build()
        .unwrap();

    // Since the request was forwarded, we should see the target host's response.
    let response = client.get(target_server.url("/get")).send().unwrap();

    // Extract the status code before calling .text() which consumes the response
    let status_code = response.status().as_u16();
    let response_text = response.text().unwrap(); // Store the text response in a variable

    assert_eq!("Hi from fake GitHub!", response_text); // Use the stored text for comparison
    assert_eq!(status_code, 200); // Now compare the status code
}


#[cfg(all(feature = "proxy", feature = "https"))]
#[test]
fn testik() {
    let server = httpmock::MockServer::start();
    server.proxy(|rule| {
        rule.filter(|when| {
            when.any_request();
        });
    });

    let client = ClientBuilder::new()
        .proxy(reqwest::Proxy::all(server.base_url()).unwrap())
        .redirect(Policy::none())
        .build().unwrap();

    let response = client.get("https://yahoo.com/").send().unwrap();
    assert_eq!(response.status(), 301);

    let response = client.get("https://google.com/").send().unwrap();
    assert_eq!(response.status(), 301);
}