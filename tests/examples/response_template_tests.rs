//! Response templates run on the server for each matched request. These examples cover the
//! builder API locally and through the remote API, including JSON escaping and repeated values.

use httpmock::MockServer;
use reqwest::blocking::Client;
use serde_json::{Value, json};

// @example-start: templates
/// Render response headers and JSON from each request, preserving repeated values and JSON escaping.
#[test]
fn response_templates_use_request_data() {
    use httpmock::MockServer;
    use reqwest::blocking::Client;
    use serde_json::{Value, json};

    let server = MockServer::start();

    // Match user requests and build a JSON reply from their path, query, headers, and body.
    // `tojson` inserts a complete JSON value; do not put additional quotes around it.
    let source = String::from(
        r#"{
            "path": {{ request.path | tojson }},
            "method": {{ request.method | tojson }},
            "names": {{ request.query.name | tojson }},
            "tags": {{ request.headers["x-tag"] | tojson }},
            "user": {{ request.json.user | tojson }},
            "state": {% if request.json.active %}"active"{% else %}"inactive"{% endif %}
        }"#,
    );
    let mock = server.mock(|when, then| {
        when.method("POST").path_prefix("/users/");
        then.status(201)
            .header("content-type", "application/json")
            .header_template("x-request-id", r#"{{ request.headers["x-request-id"][0] }}"#)
            .body_template(&source);
    });

    // The builder owns its copy; the user's template string does not need to outlive the mock.
    drop(source);

    // Send two requests with different data, repeated query/header values, and text needing JSON escaping.
    let client = Client::new();
    for (id, active) in [(42, true), (43, false)] {
        let request_id = format!("request-{id}");
        let user = json!({ "name": "Alex \"the tester\"\n<&>", "id": id });
        let response = client
            .post(server.url(format!("/users/{id}")))
            .query(&[("name", "first & second"), ("name", "third")])
            .header("X-Request-Id", &request_id)
            .header("x-tag", "one")
            .header("x-tag", "two")
            .body(json!({ "user": user, "active": active }).to_string())
            .send()
            .unwrap();

        // Two different requests must produce different responses from the same compiled template.
        assert_eq!(response.status(), 201);
        assert_eq!(response.headers()["content-type"], "application/json");
        assert_eq!(response.headers()["x-request-id"], request_id);

        // Parsing the reply also checks that `tojson` escaped the user-supplied text correctly.
        let body: Value = serde_json::from_str(&response.text().unwrap()).unwrap();
        assert_eq!(body["path"], format!("/users/{id}"));
        assert_eq!(body["method"], "POST");
        assert_eq!(body["names"], json!(["first & second", "third"]));
        assert_eq!(body["tags"], json!(["one", "two"]));
        assert_eq!(body["user"], user);
        assert_eq!(body["state"], if active { "active" } else { "inactive" });
    }

    // Both requests must have matched the same mock.
    mock.assert_calls(2);
}
// @example-end

/// Send templates through the remote API and render them against each request on the server.
#[cfg(feature = "remote")]
#[test]
fn response_templates_use_request_data_on_remote_server() {
    // Use the shared standalone server to exercise template registration over HTTP.
    crate::with_standalone_server();
    let server = MockServer::connect("localhost:5050");

    // Match user requests and build a JSON reply from their path, query, headers, and body.
    // `tojson` inserts a complete JSON value; do not put additional quotes around it.
    let source = String::from(
        r#"{
            "path": {{ request.path | tojson }},
            "method": {{ request.method | tojson }},
            "names": {{ request.query.name | tojson }},
            "tags": {{ request.headers["x-tag"] | tojson }},
            "user": {{ request.json.user | tojson }},
            "state": {% if request.json.active %}"active"{% else %}"inactive"{% endif %}
        }"#,
    );
    let mock = server.mock(|when, then| {
        when.method("POST").path_prefix("/users/");
        then.status(201)
            .header("content-type", "application/json")
            .header_template("x-request-id", r#"{{ request.headers["x-request-id"][0] }}"#)
            .body_template(&source);
    });

    // The builder owns its copy; the user's template string does not need to outlive the mock.
    drop(source);

    // Vary the request data to check that the remote server renders each response independently.
    let client = Client::new();
    for (id, active) in [(42, true), (43, false)] {
        let request_id = format!("request-{id}");
        let user = json!({ "name": "Alex \"the tester\"\n<&>", "id": id });
        let response = client
            .post(server.url(format!("/users/{id}")))
            .query(&[("name", "first & second"), ("name", "third")])
            .header("X-Request-Id", &request_id)
            .header("x-tag", "one")
            .header("x-tag", "two")
            .body(json!({ "user": user, "active": active }).to_string())
            .send()
            .unwrap();

        // Two different requests must produce different responses from the same compiled template.
        assert_eq!(response.status(), 201);
        assert_eq!(response.headers()["content-type"], "application/json");
        assert_eq!(response.headers()["x-request-id"], request_id);

        // Repeated values and escaped JSON must survive remote registration and rendering.
        let body: Value = serde_json::from_str(&response.text().unwrap()).unwrap();
        assert_eq!(body["path"], format!("/users/{id}"));
        assert_eq!(body["method"], "POST");
        assert_eq!(body["names"], json!(["first & second", "third"]));
        assert_eq!(body["tags"], json!(["one", "two"]));
        assert_eq!(body["user"], user);
        assert_eq!(body["state"], if active { "active" } else { "inactive" });
    }

    // Both requests must have matched the same remote mock.
    mock.assert_calls(2);
}

/// Configure and verify a response template using the async API from Tokio.
#[tokio::test]
async fn response_templates_render_with_tokio() {
    // Register a template that echoes the matched request's path.
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.path("/async");
            then.body_template("{{ request.path }}");
        })
        .await;

    // Trigger rendering with an asynchronous HTTP request.
    let response = reqwest::Client::new().get(server.url("/async")).send().await.unwrap();

    // The response must contain the rendered path, rather than the template source.
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "/async");
    mock.assert_async().await;
}

/// Register and verify a remote response template using Tokio.
#[cfg(feature = "remote")]
#[tokio::test]
async fn response_templates_render_with_tokio_on_remote_server() {
    // Configure the standalone server through the async remote API.
    crate::with_standalone_server();
    let server = MockServer::connect_async("localhost:5050").await;

    let mock = server
        .mock_async(|when, then| {
            when.path("/async");
            then.body_template("{{ request.path }}");
        })
        .await;

    // Trigger rendering with an asynchronous HTTP request.
    let response = reqwest::Client::new().get(server.url("/async")).send().await.unwrap();

    // Verify that the remote server rendered the request path and recorded the match.
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "/async");
    mock.assert_async().await;
}

/// Use response templates from Actix without requiring callers to switch runtimes.
#[actix_rt::test]
async fn response_templates_render_with_actix() {
    // Register a template through the async API while running on Actix.
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.path("/async");
            then.body_template("{{ request.path }}");
        })
        .await;

    // Trigger rendering with an asynchronous HTTP request.
    let response = reqwest::Client::new().get(server.url("/async")).send().await.unwrap();

    // Check both the rendered body and the recorded match through the async API.
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "/async");
    mock.assert_async().await;
}

/// Register and verify a remote response template from Actix.
#[cfg(feature = "remote")]
#[actix_rt::test]
async fn response_templates_render_with_actix_on_remote_server() {
    // Configure the standalone server through the async remote API while running on Actix.
    crate::with_standalone_server();
    let server = MockServer::connect_async("localhost:5050").await;

    let mock = server
        .mock_async(|when, then| {
            when.path("/async");
            then.body_template("{{ request.path }}");
        })
        .await;

    // Trigger rendering with an asynchronous HTTP request.
    let response = reqwest::Client::new().get(server.url("/async")).send().await.unwrap();

    // Verify that rendering and remote call assertions work from this executor.
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "/async");
    mock.assert_async().await;
}

/// Use the async template API from smol without entering a Tokio runtime.
#[test]
fn response_templates_render_with_smol() {
    smol::block_on(async {
        // Register a template that echoes the request path, with smol polling httpmock's futures.
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.path("/async");
                then.body_template("{{ request.path }}");
            })
            .await;

        // Run the blocking HTTP client on another thread; the oneshot receiver needs no Tokio runtime.
        let url = server.url("/async");
        let (sender, receiver) = tokio::sync::oneshot::channel();
        std::thread::spawn(move || {
            let response = reqwest::blocking::get(url).unwrap();
            sender.send((response.status(), response.text().unwrap())).unwrap();
        });
        let (status, body) = receiver.await.unwrap();

        // Verify rendering and await the mock assertion on smol as well.
        assert_eq!(status, 200);
        assert_eq!(body, "/async");
        mock.assert_async().await;
    });
}

/// Register and verify a remote response template with smol polling httpmock's futures.
#[cfg(feature = "remote")]
#[test]
fn response_templates_render_with_smol_on_remote_server() {
    smol::block_on(async {
        // Configure the standalone server through the remote API from smol.
        crate::with_standalone_server();
        let server = MockServer::connect_async("localhost:5050").await;

        let mock = server
            .mock_async(|when, then| {
                when.path("/async");
                then.body_template("{{ request.path }}");
            })
            .await;

        // Run the blocking HTTP client on another thread; the oneshot receiver needs no Tokio runtime.
        let url = server.url("/async");
        let (sender, receiver) = tokio::sync::oneshot::channel();
        std::thread::spawn(move || {
            let response = reqwest::blocking::get(url).unwrap();
            sender.send((response.status(), response.text().unwrap())).unwrap();
        });
        let (status, body) = receiver.await.unwrap();

        // Verify rendering and await the remote mock assertion on smol as well.
        assert_eq!(status, 200);
        assert_eq!(body, "/async");
        mock.assert_async().await;
    });
}

/// Use defaults for absent request data and loop over repeated query parameters.
#[test]
fn templates_handle_optional_values_and_loops() {
    // Default missing names to an empty list and a missing JSON body to readable text.
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.path("/greet");
        then.body_template(
            "{% for name in request.query.name | default([]) %}Hello {{ name }}! {% endfor %}\
             {{ request.json | default('no JSON') }}\n",
        );
    });

    // Missing query data and a non-JSON body are allowed when the template supplies defaults.
    let client = Client::new();
    assert_eq!(
        client.get(server.url("/greet")).send().unwrap().text().unwrap(),
        "no JSON\n"
    );

    // The loop must visit each supplied name, and rendering must retain the final newline.
    let response = client.get(server.url("/greet?name=Alex&name=Falk")).send().unwrap();

    assert_eq!(response.text().unwrap(), "Hello Alex! Hello Falk! no JSON\n");
    mock.assert_calls(2);
}

/// Keep ordinary bodies literal and let the last body setter select literal or templated output.
#[test]
fn literal_bodies_stay_literal_and_last_body_setter_wins() {
    let server = MockServer::start();
    let literal = "{{ request.path }}";

    // `body` replaces an earlier template without interpreting template-like text.
    let mock = server.mock(|when, then| {
        when.path("/literal");
        then.body_template("discarded template").body(literal);
    });

    let response = reqwest::blocking::get(server.url("/literal")).unwrap();
    assert_eq!(response.text().unwrap(), literal);
    mock.assert();

    // `json_body` also replaces the template and preserves strings as literal JSON values.
    let mock = server.mock(|when, then| {
        when.path("/json");
        then.body_template("discarded template")
            .json_body(json!({ "literal": literal }));
    });

    let response = reqwest::blocking::get(server.url("/json")).unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&response.text().unwrap()).unwrap(),
        json!({ "literal": literal })
    );
    mock.assert();

    // Setting a template last replaces the literal body and enables rendering.
    let mock = server.mock(|when, then| {
        when.path("/template");
        then.body("discarded body").body_template("{{ request.path }}");
    });

    let response = reqwest::blocking::get(server.url("/template")).unwrap();
    assert_eq!(response.text().unwrap(), "/template");
    mock.assert();
}

/// Reject invalid template syntax when registering the mock, before any request arrives.
#[test]
#[should_panic(expected = "cannot compile response template")]
fn invalid_template_fails_during_setup() {
    let server = MockServer::start();

    // The incomplete `if` expression must make mock registration fail.
    server.mock(|when, then| {
        when.path("/invalid");
        then.body_template("{% if %}");
    });
}

/// Reject two competing ways of generating the same response.
#[test]
#[should_panic(expected = "response templates cannot be combined with respond_with")]
fn templates_cannot_be_combined_with_a_rust_responder() {
    let server = MockServer::start();

    // A mock cannot use both a response template and a Rust response callback.
    server.mock(|when, then| {
        when.path("/ambiguous");
        then.body_template("{{ request.path }}")
            .respond_with(|_| httpmock::HttpMockResponse::builder().status(200).build());
    });
}

/// Report a rendering error when a required request value is missing and has no default.
#[test]
fn missing_request_value_returns_a_rendering_error() {
    // This template requires a query parameter that the request below deliberately omits.
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.path("/missing");
        then.body_template("{{ request.query.required[0] }}");
    });

    // Match the mock without providing the template's required input.
    let response = reqwest::blocking::get(server.url("/missing")).unwrap();

    // Rendering must fail explicitly, while still counting the matched request.
    assert_eq!(response.status(), 500);
    assert!(response.text().unwrap().contains("cannot render response template"));
    mock.assert();
}

/// Reject a rendered header containing line breaks instead of emitting an invalid HTTP response.
#[test]
fn invalid_rendered_header_returns_an_error() {
    // Copy a JSON field into a response header to exercise validation after rendering.
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.path("/header");
        then.header_template("x-user", "{{ request.json.name }}");
    });

    // Supply line breaks through valid JSON; these are not valid within an HTTP header value.
    let response = Client::new()
        .post(server.url("/header"))
        .body(json!({ "name": "Alex\r\nInjected: header" }).to_string())
        .send()
        .unwrap();

    // Header validation must fail before the injected value can be sent to the client.
    assert_eq!(response.status(), 500);
    mock.assert();
}

/// Expose binary request bodies as base64 without losing non-UTF-8 bytes.
#[test]
fn binary_request_bodies_are_available_as_base64() {
    // Echo the base64 representation of the incoming request body.
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.path("/binary");
        then.body_template("{{ request.body_base64 }}");
    });

    // Base64 retains bytes that cannot be represented by the lossy text field `request.body`.
    let response = Client::new()
        .post(server.url("/binary"))
        .body(b"hi\xff".to_vec())
        .send()
        .unwrap();

    // The encoded response must retain the original invalid UTF-8 byte as well.
    assert_eq!(response.text().unwrap(), "aGn/");
    mock.assert();
}

/// Stop template execution when it exceeds the configured instruction budget.
#[test]
fn excessive_template_execution_returns_an_error() {
    // Nested loops exhaust the budget without allocating a large response body.
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.path("/expensive");
        then.body_template("{% for x in range(10000) %}{% for y in range(10000) %}{% endfor %}{% endfor %}");
    });

    // The template is valid at registration; its execution limit applies when a request matches.
    let response = reqwest::blocking::get(server.url("/expensive")).unwrap();

    // Return a rendering error identifying the exhausted MiniJinja execution budget ("fuel").
    assert_eq!(response.status(), 500);
    assert!(response.text().unwrap().contains("fuel"));
    mock.assert();
}

/// Load body and header templates from YAML and render them against an incoming request.
#[cfg(feature = "record")]
#[test]
fn response_templates_from_yaml() {
    let server = MockServer::start();

    // The YAML fixture greets a query parameter and echoes the request ID in a response header.
    let mocks = server.playback_from_yaml(include_str!("../resources/response_template.yaml"));

    // Supply the query and header values referenced by the YAML templates.
    let response = Client::new()
        .get(server.url("/greet?name=Alex"))
        .header("x-request-id", "yaml-example")
        .send()
        .unwrap();

    // Verify both rendered values and the call count of the mock created from YAML.
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["x-request-id"], "yaml-example");
    assert_eq!(response.text().unwrap(), "Hello Alex!");
    assert_eq!(mocks.ids.len(), 1);
    httpmock::Mock::new(mocks.ids[0], &server).assert();
}

/// Register YAML response templates remotely and render them on the standalone server.
#[cfg(feature = "record")]
#[test]
fn response_templates_from_yaml_on_remote_server() {
    // Load the YAML through the remote API, exercising serialization of both template fields.
    crate::with_standalone_server();
    let server = MockServer::connect("localhost:5050");

    // The YAML fixture greets a query parameter and echoes the request ID in a response header.
    let mocks = server.playback_from_yaml(include_str!("../resources/response_template.yaml"));

    // Supply the query and header values referenced by the YAML templates.
    let response = Client::new()
        .get(server.url("/greet?name=Alex"))
        .header("x-request-id", "yaml-example")
        .send()
        .unwrap();

    // Verify the remote server rendered both values and recorded the matching request.
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["x-request-id"], "yaml-example");
    assert_eq!(response.text().unwrap(), "Hello Alex!");
    assert_eq!(mocks.ids.len(), 1);
    httpmock::Mock::new(mocks.ids[0], &server).assert();
}
