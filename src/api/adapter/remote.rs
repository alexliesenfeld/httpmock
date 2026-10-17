use std::{net::SocketAddr, sync::Arc};

use async_trait::async_trait;
use bytes::Bytes;
use http::{Request, StatusCode};
#[cfg(feature = "record")]
use serde::Deserialize;
use serde::de::DeserializeOwned;

#[cfg(feature = "proxy")]
use crate::common::data::{ActiveForwardingRule, ActiveProxyRule, ForwardingRuleConfig, ProxyRuleConfig};
#[cfg(feature = "record")]
use crate::common::data::{ActiveRecording, RecordingRuleConfig};
#[cfg(feature = "record")]
use crate::common::static_mock::StaticMockDefinition;
use crate::{
    api::{
        MockServerAdapter,
        adapter::{
            ServerAdapterError,
            ServerAdapterError::{
                InvalidMockDefinitionError, JsonDeserializationError, JsonSerializationError, UpstreamError,
            },
        },
    },
    common::{
        data::{ActiveMock, ClosestMatch, MockDefinition, MockServerHttpResponse, RequestRequirements},
        http::HttpClient,
    },
};

pub(in crate::api) struct RemoteMockServerAdapter {
    addr: SocketAddr,
    http_client: Arc<dyn HttpClient + Send + Sync + 'static>,
}

impl RemoteMockServerAdapter {
    pub(in crate::api) fn new(addr: SocketAddr, http_client: Arc<dyn HttpClient + Send + Sync + 'static>) -> Self {
        Self { addr, http_client }
    }

    fn validate_request_requirements(&self, requirements: &RequestRequirements) -> Result<(), ServerAdapterError> {
        match requirements.is_true {
            Some(_) => Err(InvalidMockDefinitionError(
                "Anonymous function request matchers are not supported when using a remote mock server".to_string(),
            )),
            None => Ok(()),
        }
    }

    fn validate_response(&self, response: &MockServerHttpResponse) -> Result<(), ServerAdapterError> {
        match response.respond_with {
            Some(_) => Err(InvalidMockDefinitionError(
                "Dynamic responders are not supported by remote/standalone servers".to_string(),
            )),
            None => Ok(()),
        }
    }

    async fn validate_template_support(&self) -> Result<(), ServerAdapterError> {
        // Older servers ignore unknown mock fields. Check support before sending a template
        // so registration cannot succeed while silently losing the requested behavior.
        let capabilities: serde_json::Value = self
            .request_json(
                "GET",
                "capabilities",
                None,
                StatusCode::OK,
                "check response template support",
            )
            .await
            .map_err(|error| InvalidMockDefinitionError(format!("Cannot verify remote template support: {error}")))?;

        if capabilities.get("response_templates").and_then(|value| value.as_bool()) != Some(true) {
            return Err(InvalidMockDefinitionError(
                "Remote server does not support response templates; enable its 'template' Cargo feature".to_string(),
            ));
        }

        Ok(())
    }

    /// Builds a request against the `__httpmock__` API. When `json_body` is
    /// `Some`, the payload is sent with a JSON content type.
    fn build_request(
        &self,
        method: &str,
        path: &str,
        json_body: Option<String>,
    ) -> Result<Request<Bytes>, ServerAdapterError> {
        let mut builder = Request::builder()
            .method(method)
            .uri(format!("http://{}/__httpmock__/{}", self.addr, path));

        let body = match json_body {
            Some(json) => {
                builder = builder.header("content-type", "application/json");
                Bytes::from(json)
            }
            None => Bytes::new(),
        };

        builder.body(body).map_err(|e| UpstreamError(e.to_string()))
    }

    /// Sends a request and returns the response body as a string, failing with
    /// an [`UpstreamError`] when the status does not match `expected_status`.
    /// `context` names the operation for diagnostic error messages.
    async fn send_checked(
        &self,
        request: Request<Bytes>,
        expected_status: StatusCode,
        context: &str,
    ) -> Result<String, ServerAdapterError> {
        let (status, body) = self.do_request(request).await?;

        if status != expected_status.as_u16() {
            return Err(UpstreamError(format!(
                "Could not {}. Expected response status {} but was {} (response body = '{}')",
                context,
                expected_status.as_u16(),
                status,
                body
            )));
        }

        Ok(body)
    }

    /// Sends a JSON request and deserializes the response body.
    async fn request_json<T: DeserializeOwned>(
        &self,
        method: &str,
        path: &str,
        json_body: Option<String>,
        expected_status: StatusCode,
        context: &str,
    ) -> Result<T, ServerAdapterError> {
        let request = self.build_request(method, path, json_body)?;
        let body = self.send_checked(request, expected_status, context).await?;
        serde_json::from_str(&body).map_err(JsonDeserializationError)
    }

    /// Sends a request that is expected to have no meaningful response body,
    /// only verifying the response status.
    async fn request_empty(
        &self,
        method: &str,
        path: &str,
        json_body: Option<String>,
        expected_status: StatusCode,
        context: &str,
    ) -> Result<(), ServerAdapterError> {
        let request = self.build_request(method, path, json_body)?;
        self.send_checked(request, expected_status, context).await?;
        Ok(())
    }

    async fn do_request(&self, req: Request<Bytes>) -> Result<(u16, String), ServerAdapterError> {
        let (code, body_bytes) = self.do_request_raw(req).await?;

        let body = String::from_utf8(body_bytes.to_vec()).map_err(|e| UpstreamError(e.to_string()))?;

        Ok((code, body))
    }

    async fn do_request_raw(&self, req: Request<Bytes>) -> Result<(u16, Bytes), ServerAdapterError> {
        let response = self
            .http_client
            .send(req)
            .await
            .map_err(|e| UpstreamError(e.to_string()))?;

        Ok((response.status().as_u16(), response.body().clone()))
    }
}

#[async_trait]
impl MockServerAdapter for RemoteMockServerAdapter {
    fn host(&self) -> String {
        self.addr.ip().to_string()
    }

    fn port(&self) -> u16 {
        self.addr.port()
    }

    fn address(&self) -> &SocketAddr {
        &self.addr
    }

    async fn reset(&self) -> Result<(), ServerAdapterError> {
        self.request_empty("DELETE", "state", None, StatusCode::NO_CONTENT, "reset the mock server")
            .await
    }

    async fn create_mock(&self, mock: &MockDefinition) -> Result<ActiveMock, ServerAdapterError> {
        self.validate_request_requirements(&mock.request)?;
        self.validate_response(&mock.response)?;

        if mock.response.has_templates() {
            self.validate_template_support().await?;
        }

        let json = serde_json::to_string(mock).map_err(JsonSerializationError)?;

        self.request_json("POST", "mocks", Some(json), StatusCode::CREATED, "create mock")
            .await
    }

    async fn fetch_mock(&self, mock_id: usize) -> Result<ActiveMock, ServerAdapterError> {
        self.request_json(
            "GET",
            &format!("mocks/{}", mock_id),
            None,
            StatusCode::OK,
            "fetch mock from the mock server",
        )
        .await
    }

    async fn delete_mock(&self, mock_id: usize) -> Result<(), ServerAdapterError> {
        self.request_empty(
            "DELETE",
            &format!("mocks/{}", mock_id),
            None,
            StatusCode::NO_CONTENT,
            "delete mock from the mock server",
        )
        .await
    }

    async fn verify(&self, requirements: &RequestRequirements) -> Result<Option<ClosestMatch>, ServerAdapterError> {
        let json = serde_json::to_string(requirements).map_err(JsonSerializationError)?;

        let request = self.build_request("POST", "verify", Some(json))?;
        let (status, body) = self.do_request(request).await?;

        if status == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if status != StatusCode::OK.as_u16() {
            return Err(UpstreamError(format!(
                "Could not verify mock. Expected response status 200 but was {} (response body = '{}')",
                status, body
            )));
        }

        let response: ClosestMatch = serde_json::from_str(&body).map_err(JsonDeserializationError)?;

        Ok(Some(response))
    }

    #[cfg(feature = "proxy")]
    async fn create_forwarding_rule(
        &self,
        config: ForwardingRuleConfig,
    ) -> Result<ActiveForwardingRule, ServerAdapterError> {
        self.validate_request_requirements(&config.request_requirements)?;

        let json = serde_json::to_string(&config).map_err(JsonSerializationError)?;

        self.request_json(
            "POST",
            "forwarding_rules",
            Some(json),
            StatusCode::CREATED,
            "create forwarding rule",
        )
        .await
    }

    #[cfg(feature = "proxy")]
    async fn delete_forwarding_rule(&self, id: usize) -> Result<(), ServerAdapterError> {
        self.request_empty(
            "DELETE",
            &format!("forwarding_rules/{}", id),
            None,
            StatusCode::NO_CONTENT,
            "delete forwarding rule from the mock server",
        )
        .await
    }

    #[cfg(feature = "proxy")]
    async fn create_proxy_rule(&self, config: ProxyRuleConfig) -> Result<ActiveProxyRule, ServerAdapterError> {
        self.validate_request_requirements(&config.request_requirements)?;

        let json = serde_json::to_string(&config).map_err(JsonSerializationError)?;

        self.request_json(
            "POST",
            "proxy_rules",
            Some(json),
            StatusCode::CREATED,
            "create proxy rule",
        )
        .await
    }

    #[cfg(feature = "proxy")]
    async fn delete_proxy_rule(&self, id: usize) -> Result<(), ServerAdapterError> {
        self.request_empty(
            "DELETE",
            &format!("proxy_rules/{}", id),
            None,
            StatusCode::NO_CONTENT,
            "delete proxy rule from the mock server",
        )
        .await
    }

    #[cfg(feature = "record")]
    async fn create_recording(&self, config: RecordingRuleConfig) -> Result<ActiveRecording, ServerAdapterError> {
        self.validate_request_requirements(&config.request_requirements)?;

        let json = serde_json::to_string(&config).map_err(JsonSerializationError)?;

        self.request_json(
            "POST",
            "recordings",
            Some(json),
            StatusCode::CREATED,
            "create recording",
        )
        .await
    }

    #[cfg(feature = "record")]
    async fn delete_recording(&self, id: usize) -> Result<(), ServerAdapterError> {
        self.request_empty(
            "DELETE",
            &format!("recordings/{}", id),
            None,
            StatusCode::NO_CONTENT,
            "delete recording from the mock server",
        )
        .await
    }

    #[cfg(feature = "record")]
    async fn export_recording(&self, id: usize) -> Result<Option<Bytes>, ServerAdapterError> {
        let request = self.build_request("GET", &format!("recordings/{}", id), None)?;

        let (status, body) = self.do_request_raw(request).await?;

        if status == StatusCode::NOT_FOUND {
            return Ok(None);
        } else if status != StatusCode::OK.as_u16() {
            return Err(UpstreamError(format!(
                "Could not fetch mock from the mock server. Expected response status 200 but was {}",
                status
            )));
        }

        Ok(Some(body))
    }

    #[cfg(feature = "record")]
    async fn create_mocks_from_recording<'a>(
        &self,
        recording_file_content: &'a str,
    ) -> Result<Vec<usize>, ServerAdapterError> {
        let definitions = serde_yaml::Deserializer::from_str(recording_file_content)
            .map(StaticMockDefinition::deserialize)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| InvalidMockDefinitionError(error.to_string()))?;

        // Register through the same API as builder-created mocks, including template support
        // checks. The recordings collection endpoint creates recording rules, not playback mocks.
        let mut ids = Vec::with_capacity(definitions.len());
        for definition in definitions {
            let definition: MockDefinition = definition
                .try_into()
                .map_err(|error: crate::common::data::Error| InvalidMockDefinitionError(error.to_string()))?;
            ids.push(self.create_mock(&definition).await?.id);
        }
        Ok(ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct UnsupportedTemplateServer {
        status: StatusCode,
        body: &'static str,
    }

    #[async_trait]
    impl HttpClient for UnsupportedTemplateServer {
        async fn send(&self, request: Request<Bytes>) -> Result<http::Response<Bytes>, crate::common::http::Error> {
            // Simulate an old or feature-disabled server. Any registration request is a test
            // failure: unsupported templates must be rejected before a mock reaches the server.
            assert_eq!(request.method(), "GET");
            assert_eq!(request.uri().path(), "/__httpmock__/capabilities");
            Ok(http::Response::builder()
                .status(self.status)
                .body(Bytes::from(self.body))
                .unwrap())
        }
    }

    #[tokio::test]
    async fn remote_templates_require_server_support() {
        for (status, body) in [
            (StatusCode::NOT_FOUND, "old server"),
            (StatusCode::OK, r#"{"response_templates":false}"#),
            (StatusCode::OK, "{}"),
        ] {
            let adapter = RemoteMockServerAdapter::new(
                "127.0.0.1:5050".parse().unwrap(),
                Arc::new(UnsupportedTemplateServer { status, body }),
            );
            let definition = MockDefinition {
                request: RequestRequirements::default(),
                response: MockServerHttpResponse {
                    body_template: Some("{{ request.path }}".to_string()),
                    ..Default::default()
                },
            };

            assert!(matches!(
                adapter.create_mock(&definition).await,
                Err(InvalidMockDefinitionError(_))
            ));

            // YAML playback uses a separate endpoint and must perform the same check.
            #[cfg(feature = "record")]
            assert!(matches!(
                adapter
                    .create_mocks_from_recording("when: {}\nthen:\n  body_template: '{{ request.path }}'")
                    .await,
                Err(InvalidMockDefinitionError(_))
            ));
        }
    }
}
