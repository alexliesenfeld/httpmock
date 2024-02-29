use httpmock::prelude::*;
use isahc::prelude::*;

#[test]
fn proxy_test() {
    // Arrange
    let server = MockServer::start();

    server.proxy(|when| {
        when.host("www.google.com");
    });

    let recorder = server.record(|when| {
        when.host("www.apple.com");
    });

    // recorder.download_to("~/httpmock-recordings");
    // recorder.print_specs();

    let client = isahc::HttpClient::builder()
        .proxy(Some(server.base_url().parse().unwrap()))
        .build()
        .unwrap();

    // Act
    let mut response = client.get("https://www.google.com").unwrap();

    assert_eq!(response.status(), 200);
}
