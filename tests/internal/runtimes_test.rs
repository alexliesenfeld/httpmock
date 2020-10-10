extern crate httpmock;

use std::future::Future;
use std::io::Read;

use isahc::{get, get_async, Body, RequestExt};
use regex::Replacer;

use httpmock::MockServer;

use crate::simulate_standalone_server;

#[test]
fn all_runtimes_test() {
    // Tokio
    assert_eq!(
        tokio::runtime::Runtime::new().unwrap().block_on(test_fn()),
        202
    );

    // actix
    assert_eq!(actix_rt::Runtime::new().unwrap().block_on(test_fn()), 202);

    // async_std
    assert_eq!(async_std::task::block_on(test_fn()), 202);
}

async fn test_fn() -> u16 {
    // Instead of creating a new MockServer using new(), we connect to an existing remote instance.
    let server = MockServer::start_async().await;

    let search_mock = server
        .mock_async(|when, then| {
            when.path("/test");
            then.status(202);
        })
        .await;

    // Act: Send the HTTP request
    let response = get_async(server.url("/test")).await.unwrap();

    // Assert
    search_mock.assert_async().await;
    assert_eq!(response.status(), 202);

    response.status().as_u16()
}
