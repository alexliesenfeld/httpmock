use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use isahc::http::Request;
use isahc::prelude::Configurable;
use isahc::ResponseExt;

use crate::data::{
    ActiveMock, ClosestMatch, MockDefinition, MockIdentification, RequestRequirements,
};
use crate::server::web::handlers::{
    add_new_mock, delete_all_mocks, delete_history, delete_one_mock, read_one_mock, verify,
};
use crate::server::{Mismatch, MockServerState};

pub mod local;
pub mod standalone;

/// Type alias for [regex::Regex](../regex/struct.Regex.html).
pub type Regex = regex::Regex;

pub type InternalHttpClient = isahc::HttpClient;

/// Represents an HTTP method.
#[derive(Debug)]
pub enum Method {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}

impl FromStr for Method {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "GET" => Ok(Method::GET),
            "HEAD" => Ok(Method::HEAD),
            "POST" => Ok(Method::POST),
            "PUT" => Ok(Method::PUT),
            "DELETE" => Ok(Method::DELETE),
            "CONNECT" => Ok(Method::CONNECT),
            "OPTIONS" => Ok(Method::OPTIONS),
            "TRACE" => Ok(Method::TRACE),
            "PATCH" => Ok(Method::PATCH),
            _ => Err(format!("Invalid HTTP method {}", input)),
        }
    }
}

impl From<&str> for Method {
    fn from(value: &str) -> Self {
        value.parse().expect("Cannot parse HTTP method")
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

#[async_trait]
pub trait MockServerAdapter {
    fn host(&self) -> String;
    fn port(&self) -> u16;
    fn address(&self) -> &SocketAddr;
    async fn create_mock(&self, mock: &MockDefinition) -> Result<MockIdentification, String>;
    async fn fetch_mock(&self, mock_id: usize) -> Result<ActiveMock, String>;
    async fn delete_mock(&self, mock_id: usize) -> Result<(), String>;
    async fn delete_all_mocks(&self) -> Result<(), String>;
    async fn verify(&self, rr: &RequestRequirements) -> Result<Option<ClosestMatch>, String>;
    async fn delete_history(&self) -> Result<(), String>;
    async fn ping(&self) -> Result<(), String>;
}

async fn http_ping(
    server_addr: &SocketAddr,
    http_client: &InternalHttpClient,
) -> Result<(), String> {
    let request_url = format!("http://{}/__ping", server_addr);
    let request = Request::builder()
        .method("GET")
        .uri(request_url)
        .body("".to_string())
        .unwrap();

    let (status, _body) = match execute_request(request, http_client).await {
        Err(err) => return Err(format!("cannot send request to mock server: {}", err)),
        Ok(sb) => sb,
    };

    if status != 200 {
        return Err(format!(
            "Could not create mock. Mock server response: status = {}",
            status
        ));
    }

    Ok(())
}

async fn execute_request(
    req: Request<String>,
    http_client: &InternalHttpClient,
) -> Result<(u16, String), String> {
    let mut response = match http_client.send_async(req).await {
        Err(err) => return Err(format!("cannot send request to mock server: {}", err)),
        Ok(r) => r,
    };

    // Evaluate the response status
    let body = match response.text() {
        Err(err) => return Err(format!("cannot send request to mock server: {}", err)),
        Ok(b) => b,
    };

    Ok((response.status().as_u16(), body))
}

fn build_http_client() -> Arc<InternalHttpClient> {
    Arc::new(
        InternalHttpClient::builder()
            .tcp_keepalive(Duration::from_secs(60 * 60 * 24))
            .build()
            .expect("Cannot build HTTP client"),
    )
}
