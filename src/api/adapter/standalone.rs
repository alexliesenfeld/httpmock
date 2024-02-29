use std::borrow::Borrow;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use crate::api::MockServerAdapter;
use async_trait::async_trait;
use isahc::config::Configurable;
use isahc::{AsyncReadResponseExt, Request, ResponseExt};

pub type InternalHttpClient = isahc::HttpClient;

use crate::common::data::{ActiveMock, ClosestMatch, MockDefinition, MockRef, RequestRequirements};

#[derive(Debug)]
pub struct RemoteMockServerAdapter {
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

    async fn create_mock(&self, mock: &MockDefinition) -> Result<MockRef, String> {
        // Check if the request can be sent via HTTP
        self.validate_mock(mock).expect("Cannot create mock");

        // Serialize to JSON
        let json = match serde_json::to_string(mock) {
            Err(err) => return Err(format!("cannot serialize mock object to JSON: {}", err)),
            Ok(json) => json,
        };

        // Send the request to the mock server
        let request_url = format!("http://{}/__httpmock__/mocks", &self.address());
        let request = Request::builder()
            .method("POST")
            .uri(request_url)
            .header("content-type", "application/json")
            .body(json)
            .unwrap();

        let (status, body) = match execute_request(request, &self.http_client).await {
            Err(err) => return Err(format!("cannot send request to mock server: {}", err)),
            Ok(sb) => sb,
        };

        // Evaluate the response status
        if status != 201 {
            return Err(format!(
                "Could not create mock. Mock server response: status = {}, message = {}",
                status, body
            ));
        }

        // Create response object
        let response: serde_json::Result<MockRef> = serde_json::from_str(&body);
        if let Err(err) = response {
            return Err(format!("Cannot deserialize mock server response: {}", err));
        }

        Ok(response.unwrap())
    }

    async fn fetch_mock(&self, mock_id: usize) -> Result<ActiveMock, String> {
        // Send the request to the mock server
        let request_url = format!("http://{}/__httpmock__/mocks/{}", &self.address(), mock_id);
        let request = Request::builder()
            .method("GET")
            .uri(request_url)
            .body("".to_string())
            .unwrap();

        let (status, body) = match execute_request(request, &self.http_client).await {
            Err(err) => return Err(format!("Cannot send request to mock server: {}", err)),
            Ok(r) => r,
        };

        // Evaluate response status code
        if status != 200 {
            return Err(format!(
                "Could not create mock. Mock server response: status = {}, message = {}",
                status, body
            ));
        }

        // Create response object
        let response: serde_json::Result<ActiveMock> = serde_json::from_str(&body);
        if let Err(err) = response {
            return Err(format!("Cannot deserialize mock server response: {}", err));
        }

        Ok(response.unwrap())
    }

    async fn delete_mock(&self, mock_id: usize) -> Result<(), String> {
        // Send the request to the mock server
        let request_url = format!("http://{}/__httpmock__/mocks/{}", &self.address(), mock_id);
        let request = Request::builder()
            .method("DELETE")
            .uri(request_url)
            .body("".to_string())
            .unwrap();

        let (status, body) = match execute_request(request, &self.http_client).await {
            Err(err) => return Err(format!("Cannot send request to mock server: {}", err)),
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
        let request_url = format!("http://{}/__httpmock__/mocks", &self.address());
        let request = Request::builder()
            .method("DELETE")
            .uri(request_url)
            .body("".to_string())
            .unwrap();

        let (status, body) = match execute_request(request, &self.http_client).await {
            Err(err) => return Err(format!("Cannot send request to mock server: {}", err)),
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

    async fn verify(&self, mock_rr: &RequestRequirements) -> Result<Option<ClosestMatch>, String> {
        // Serialize to JSON
        let json = match serde_json::to_string(mock_rr) {
            Err(err) => return Err(format!("Cannot serialize mock object to JSON: {}", err)),
            Ok(json) => json,
        };

        // Send the request to the mock server
        let request_url = format!("http://{}/__httpmock__/verify", &self.address());
        let request = Request::builder()
            .method("POST")
            .uri(request_url)
            .header("content-type", "application/json")
            .body(json)
            .unwrap();

        let (status, body) = match execute_request(request, &self.http_client).await {
            Err(err) => return Err(format!("Cannot send request to mock server: {}", err)),
            Ok(sb) => sb,
        };

        // Evaluate the response status
        if status == 404 {
            return Ok(None);
        }

        if status != 200 {
            return Err(format!(
                "Could not execute verification (status = {}, message = {})",
                status, body
            ));
        }

        // Create response object
        let response: serde_json::Result<ClosestMatch> = serde_json::from_str(&body);
        if let Err(err) = response {
            return Err(format!("cannot deserialize mock server response: {}", err));
        }

        Ok(Some(response.unwrap()))
    }

    async fn delete_history(&self) -> Result<(), String> {
        // Send the request to the mock server
        let request_url = format!("http://{}/__httpmock__/history", &self.address());
        let request = Request::builder()
            .method("DELETE")
            .uri(request_url)
            .body("".to_string())
            .unwrap();

        let (status, body) = match execute_request(request, &self.http_client).await {
            Err(err) => return Err(format!("Cannot send request to mock server: {}", err)),
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

    async fn proxy(&self, req: RequestRequirements) -> Result<(), String> {
        todo!()
    }

    async fn record(&self, req: RequestRequirements) -> Result<(), String> {
        todo!()
    }

    async fn ping(&self) -> Result<(), String> {
        http_ping(&self.addr, self.http_client.borrow()).await
    }

}

async fn http_ping(
    server_addr: &SocketAddr,
    http_client: &InternalHttpClient,
) -> Result<(), String> {
    let request_url = format!("http://{}/__httpmock__/ping", server_addr);
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
    let body = match response.text().await {
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
