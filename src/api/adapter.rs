use async_trait::async_trait;
use isahc::prelude::*;
use std::borrow::Borrow;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use crate::data::{ActiveMock, ClosestMatch, MockDefinition, MockIdentification};
use crate::server::web::handlers::{
    add_new_mock, delete_all_mocks, delete_history, delete_one_mock, find_closest_match,
    read_one_mock,
};
use crate::server::{Mismatch, MockServerState};
use std::str::FromStr;

/// Type alias for [regex::Regex](../regex/struct.Regex.html).
pub type Regex = regex::Regex;

pub(crate) type InternalHttpClient = isahc::HttpClient;

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

#[async_trait]
pub(crate) trait MockServerAdapter {
    fn host(&self) -> String;
    fn port(&self) -> u16;
    fn address(&self) -> &SocketAddr;
    async fn create_mock(&self, mock: &MockDefinition) -> Result<MockIdentification, String>;
    async fn fetch_mock(&self, mock_id: usize) -> Result<ActiveMock, String>;
    async fn delete_mock(&self, mock_id: usize) -> Result<(), String>;
    async fn delete_all_mocks(&self) -> Result<(), String>;
    async fn find_closest_match(&self, mock_id: usize) -> Result<Option<ClosestMatch>, String>;

    async fn delete_history(&self) -> Result<(), String>;
    async fn ping(&self) -> Result<(), String>;
}

#[derive(Debug)]
pub(crate) struct RemoteMockServerAdapter {
    addr: SocketAddr,
    http_client: Arc<InternalHttpClient>,
}

impl RemoteMockServerAdapter {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            http_client: build_http_client(),
        }
    }

    fn validate_mock(&self, mock: &MockDefinition) -> Result<(), String> {
        if mock.request.matchers.is_some() {
            return Err(
                "Anonymous function request matchers are not supported when using a remote mock server".to_string(),
            );
        }
        Ok(())
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

    async fn create_mock(&self, mock: &MockDefinition) -> Result<MockIdentification, String> {
        // Check if the request can be sent via HTTP
        self.validate_mock(mock).expect("Cannot create mock");

        // Serialize to JSON
        let json = match serde_json::to_string(mock) {
            Err(err) => return Err(format!("cannot serialize mock object to JSON: {}", err)),
            Ok(json) => json,
        };

        // Send the request to the mock server
        let request_url = format!("http://{}/__mocks", &self.address());
        let request = Request::builder()
            .method("POST")
            .uri(request_url)
            .header("Content-Type", "application/json")
            .body(json)
            .unwrap();

        let (status, body) = match execute_request(request, &self.http_client).await {
            Err(err) => return Err(format!("cannot send request to mock server: {}", err)),
            Ok(sb) => sb,
        };

        // Evaluate the response status
        if status != 201 {
            return Err(format!(
                "could not create mock. Mock server response: status = {}, message = {}",
                status, body
            ));
        }

        // Create response object
        let response: serde_json::Result<MockIdentification> = serde_json::from_str(&body);
        if let Err(err) = response {
            return Err(format!("cannot deserialize mock server response: {}", err));
        }

        Ok(response.unwrap())
    }

    async fn fetch_mock(&self, mock_id: usize) -> Result<ActiveMock, String> {
        // Send the request to the mock server
        let request_url = format!("http://{}/__mocks/{}", &self.address(), mock_id);
        let request = Request::builder()
            .method("GET")
            .uri(request_url)
            .body("".to_string())
            .unwrap();

        let (status, body) = match execute_request(request, &self.http_client).await {
            Err(err) => return Err(format!("cannot send request to mock server: {}", err)),
            Ok(r) => r,
        };

        // Evaluate response status code
        if status != 200 {
            return Err(format!(
                "could not create mock. Mock server response: status = {}, message = {}",
                status, body
            ));
        }

        // Create response object
        let response: serde_json::Result<ActiveMock> = serde_json::from_str(&body);
        if let Err(err) = response {
            return Err(format!("cannot deserialize mock server response: {}", err));
        }

        Ok(response.unwrap())
    }

    async fn delete_mock(&self, mock_id: usize) -> Result<(), String> {
        // Send the request to the mock server
        let request_url = format!("http://{}/__mocks/{}", &self.address(), mock_id);
        let request = Request::builder()
            .method("DELETE")
            .uri(request_url)
            .body("".to_string())
            .unwrap();

        let (status, body) = match execute_request(request, &self.http_client).await {
            Err(err) => return Err(format!("cannot send request to mock server: {}", err)),
            Ok(sb) => sb,
        };

        // Evaluate response status code
        if status != 202 {
            return Err(format!(
                "Could not delete mocks from server (status = {}, message = {})",
                status, body
            ));
        }

        Ok(())
    }

    async fn delete_all_mocks(&self) -> Result<(), String> {
        // Send the request to the mock server
        let request_url = format!("http://{}/__mocks", &self.address());
        let request = Request::builder()
            .method("DELETE")
            .uri(request_url)
            .body("".to_string())
            .unwrap();

        let (status, body) = match execute_request(request, &self.http_client).await {
            Err(err) => return Err(format!("cannot send request to mock server: {}", err)),
            Ok(sb) => sb,
        };

        // Evaluate response status code
        if status != 202 {
            return Err(format!(
                "Could not delete mocks from server (status = {}, message = {})",
                status, body
            ));
        }

        Ok(())
    }

    async fn find_closest_match(&self, mock_id: usize) -> Result<Option<ClosestMatch>, String> {
        Ok(None)
        // TODO
    }

    async fn delete_history(&self) -> Result<(), String> {
        // Send the request to the mock server
        let request_url = format!("http://{}/__history", &self.address());
        let request = Request::builder()
            .method("DELETE")
            .uri(request_url)
            .body("".to_string())
            .unwrap();

        let (status, body) = match execute_request(request, &self.http_client).await {
            Err(err) => return Err(format!("cannot send request to mock server: {}", err)),
            Ok(sb) => sb,
        };

        // Evaluate response status code
        if status != 202 {
            return Err(format!(
                "Could not delete history from server (status = {}, message = {})",
                status, body
            ));
        }

        Ok(())
    }

    async fn ping(&self) -> Result<(), String> {
        http_ping(&self.addr, self.http_client.borrow()).await
    }
}

pub(crate) struct LocalMockServerAdapter {
    pub addr: SocketAddr,
    local_state: Arc<MockServerState>,
    client: Arc<InternalHttpClient>,
}

impl LocalMockServerAdapter {
    pub fn new(addr: SocketAddr, local_state: Arc<MockServerState>) -> Self {
        let client = build_http_client();
        LocalMockServerAdapter {
            addr,
            local_state,
            client,
        }
    }
}

#[async_trait]
impl MockServerAdapter for LocalMockServerAdapter {
    fn host(&self) -> String {
        self.addr.ip().to_string()
    }

    fn port(&self) -> u16 {
        self.addr.port()
    }

    fn address(&self) -> &SocketAddr {
        &self.addr
    }

    async fn create_mock(&self, mock: &MockDefinition) -> Result<MockIdentification, String> {
        let id = add_new_mock(&self.local_state, mock.clone())?;
        Ok(MockIdentification::new(id))
    }

    async fn fetch_mock(&self, mock_id: usize) -> Result<ActiveMock, String> {
        match read_one_mock(&self.local_state, mock_id)? {
            Some(mock) => Ok(mock),
            None => Err("Cannot find mock".to_string()),
        }
    }

    async fn delete_mock(&self, mock_id: usize) -> Result<(), String> {
        let deleted = delete_one_mock(&self.local_state, mock_id)?;
        if deleted {
            Ok(())
        } else {
            Err("Mock could not deleted".to_string())
        }
    }

    async fn delete_all_mocks(&self) -> Result<(), String> {
        delete_all_mocks(&self.local_state);
        Ok(())
    }

    async fn find_closest_match(&self, mock_id: usize) -> Result<Option<ClosestMatch>, String> {
        find_closest_match(&self.local_state, mock_id)
    }

    async fn delete_history(&self) -> Result<(), String> {
        delete_history(&self.local_state);
        Ok(())
    }

    async fn ping(&self) -> Result<(), String> {
        http_ping(&self.addr, self.client.borrow()).await
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
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
