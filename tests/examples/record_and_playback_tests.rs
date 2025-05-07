#[cfg(feature = "record")]
use httpmock::RecordingID;
use httpmock::prelude::*;
use reqwest::blocking::Client;

#[cfg(feature = "record")]
#[test]
fn record_with_forwarding_test() {
    let target_server = MockServer::start();
    target_server.mock(|when, then| {
        when.any_request();
        then.status(200).body("Hi from fake GitHub!");
    });

    let recording_server = MockServer::start();

    recording_server.forward_to(target_server.base_url(), |rule| {
        rule.filter(|when| {
            when.path("/hello");
        });
    });

    let recording_id = recording_server.record(|rule| {
        rule.record_response_delays(true)
            .record_request_headers(vec!["Accept", "Content-Type"])
            .filter(|when| {
                when.path("/hello");
            });
    });

    let github_client = Client::builder().build().unwrap();

    let response = github_client
        .get(format!("{}/hello", recording_server.base_url()))
        .send()
        .unwrap();
    assert_eq!(response.text().unwrap(), "Hi from fake GitHub!");

    let target_path = recording_server
        .record_save(&recording_id, "my_test_scenario")
        .unwrap();

    let playback_server = MockServer::start();

    playback_server.playback(target_path);

    let response = github_client
        .get(format!("{}/hello", playback_server.base_url()))
        .send()
        .unwrap();
    assert_eq!(response.text().unwrap(), "Hi from fake GitHub!");
}

// @example-start: record-proxy-github
#[cfg(all(feature = "proxy", feature = "experimental"))]
#[test]
fn record_with_proxy_test() {
    // Start a mock server to act as a proxy for the HTTP client
    let server = MockServer::start();

    // Configure the mock server to proxy all incoming requests
    server.proxy(|rule| {
        rule.filter(|when| {
            when.any_request(); // Intercept all requests
        });
    });

    // Set up recording on the mock server to capture all proxied
    // requests and responses
    let recording_id = server.record(|rule: RecordingRuleBuilder| {
        rule.filter(|when| {
            when.any_request(); // Record all requests
        });
    });

    // Create an HTTP client configured to route requests
    // through the mock proxy server
    let github_client = Client::builder()
        // Set the proxy URL to the mock server's URL
        .proxy(reqwest::Proxy::all(server.base_url()).unwrap())
        .build()
        .unwrap();

    // Send a GET request using the client, which will be proxied by the mock server
    let response = github_client.get(server.base_url()).send().unwrap();

    // Verify that the response matches the expected mock response
    assert_eq!(response.text().unwrap(), "This is a mock response");

    // Save the recorded HTTP interactions to a file for future reference or testing
    server
        .record_save(&recording_id, "my_scenario_name")
        .expect("could not save the recording");
}
// @example-end

// @example-start: record-forwarding-github
#[cfg(feature = "record")]
#[test]
fn record_github_api_with_forwarding_test() {
    // Let's create our mock server for the test
    let server = MockServer::start();

    // We configure our server to forward the request to the target
    // host instead of answering with a mocked response. The 'when'
    // variable lets you configure rules under which forwarding
    // should take place.
    server.forward_to("https://api.github.com", |rule| {
        rule.filter(|when| {
            when.any_request(); // Ensure all requests are forwarded.
        });
    });

    let recording_id = server.record(|rule| {
        rule
            // Specify which headers to record.
            // Only the headers listed here will be captured and stored
            // as part of the recorded mock. This selective recording is
            // necessary because some headers may vary between requests
            // and could cause issues when replaying the mock later.
            // For instance, headers like 'Authorization' or 'Date' may
            // change with each request.
            .record_request_header("User-Agent")
            .filter(|when| {
                when.any_request(); // Ensure all requests are recorded.
            });
    });

    // Now let's send an HTTP request to the mock server. The request
    // will be forwarded to the GitHub API, as we configured before.
    let client = Client::new();

    let response = client
        .get(server.url("/repos/torvalds/linux"))
        // GitHub requires us to send a user agent header
        .header("User-Agent", "httpmock-test")
        .send()
        .unwrap();

    // Since the request was forwarded, we should see a GitHub API response.
    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(true, response.text().unwrap().contains("\"private\":false"));

    // Save the recording to
    // "target/httpmock/recordings/github-torvalds-scenario_<timestamp>.yaml".
    server
        .record_save(&recording_id, "github-torvalds-scenario")
        .expect("cannot store scenario on disk");
}
// @example-end

// @example-start: playback-forwarding-github
#[cfg(feature = "record")]
#[test]
fn playback_github_api() {
    // Start a mock server for the test
    let server = MockServer::start();

    // Configure the mock server to forward requests to the target
    // host (GitHub API) instead of responding with a mock. The 'rule'
    // parameter allows you to define conditions under which forwarding
    // should occur.
    server.forward_to("https://api.github.com", |rule| {
        rule.filter(|when| {
            when.any_request(); // Forward all requests.
        });
    });

    // Set up recording to capture all forwarded requests and responses
    let recording_id = server.record(|rule| {
        rule.filter(|when| {
            when.any_request(); // Record all requests and responses.
        });
    });

    // Send an HTTP request to the mock server, which will be forwarded
    // to the GitHub API
    let client = Client::new();
    let response = client
        .get(server.url("/repos/torvalds/linux"))
        // GitHub requires a User-Agent header
        .header("User-Agent", "httpmock-test")
        .send()
        .unwrap();

    // Assert that the response from the forwarded request is as expected
    assert_eq!(response.status().as_u16(), 200);
    assert!(response.text().unwrap().contains("\"private\":false"));

    // Save the recorded interactions to a file
    let target_path = server
        .record_save(&recording_id, "github-torvalds-scenario")
        .expect("Failed to save the recording to disk");

    // Start a new mock server instance for playback
    let playback_server = MockServer::start();

    // Load the recorded interactions into the new mock server
    playback_server.playback(target_path);

    // Send a request to the playback server and verify the response
    // matches the recorded data
    let response = client
        .get(playback_server.url("/repos/torvalds/linux"))
        .send()
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);
    assert!(response.text().unwrap().contains("\"private\":false"));
}
// @example-end

#[cfg(feature = "record")]
#[test]
fn record_from_struct() {
    // Demonstrates storing the server and recording in a struct that can be
    // constructed once and carried around, potentially with a client, until
    // both go out-of-scope.
    struct RecordingServer {
        server: MockServer,
        recording: RecordingID,
    }
    impl RecordingServer {
        fn new(target_url: String) -> Self {
            let server = MockServer::start();

            server.forward_to(target_url, |rule| {
                rule.filter(|when| {
                    when.path("/hello");
                });
            });

            let recording = server.record(|rule| {
                rule.record_response_delays(true)
                    .record_request_headers(vec!["Accept", "Content-Type"])
                    .filter(|when| {
                        when.path("/hello");
                    });
            });

            Self { server, recording }
        }
    }
    // Write the recording when the server goes out-of-scope.
    impl Drop for RecordingServer {
        fn drop(&mut self) {
            self.server
                .record_save(&self.recording, "record_from_struct")
                .expect("Failed to save the recording");
        }
    }

    let target_server = MockServer::start();
    target_server.mock(|when, then| {
        when.any_request();
        then.status(200).body("Hi from fake GitHub!");
    });

    let recording_server = RecordingServer::new(target_server.base_url());

    let github_client = Client::builder().build().unwrap();
    let response = github_client
        .get(format!("{}/hello", recording_server.server.base_url()))
        .send()
        .unwrap();
    assert_eq!(response.text().unwrap(), "Hi from fake GitHub!");
}
