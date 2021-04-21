use isahc::config::RedirectPolicy;
use isahc::prelude::*;
use isahc::HttpClientBuilder;
use httpmock::prelude::*;

#[test]
fn multi_server_test() {
    // Arrange
    let server1 = MockServer::start();
    let server2 = MockServer::start();

    let redirect_mock = server1.mock(|when, then| {
        when.path("/redirectTest");
        then.temporary_redirect(&server2.url("/finalTarget"));
    });

    let target_mock = server2.mock(|when, then| {
        when.path("/finalTarget");
        then.status(200);
    });

    // Act: Send the HTTP request with an HTTP client that automatically follows redirects!
    let http_client = HttpClientBuilder::new()
        .redirect_policy(RedirectPolicy::Follow)
        .build()
        .unwrap();

    let response = http_client.get(server1.url("/redirectTest")).unwrap();

    // Assert
    redirect_mock.assert();
    target_mock.assert();
    assert_eq!(response.status(), 200);
}
