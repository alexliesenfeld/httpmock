use std::borrow::Borrow;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use async_std::io::WriteExt;

use async_trait::async_trait;

use async_std::net::TcpStream;
use async_std::prelude::*;

use crate::api::adapter::{MockServerAdapter};

use crate::common::data::{ActiveMock, ClosestMatch, MockDefinition, MockRef, RequestRequirements};
use crate::server::web::handlers::{
    add_new_mock, delete_all_mocks, delete_history, delete_one_mock, read_one_mock, verify,
};
use crate::server::MockServerState;

pub struct LocalMockServerAdapter {
    pub addr: SocketAddr,
    local_state: Arc<MockServerState>,
}

impl LocalMockServerAdapter {
    pub fn new(addr: SocketAddr, local_state: Arc<MockServerState>) -> Self {
        LocalMockServerAdapter {
            addr,
            local_state,
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

    async fn create_mock(&self, mock: &MockDefinition) -> Result<MockRef, String> {
        let id = add_new_mock(&self.local_state, mock.clone(), false)?;
        Ok(MockRef::new(id))
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

    async fn verify(&self, mock_rr: &RequestRequirements) -> Result<Option<ClosestMatch>, String> {
        verify(&self.local_state, mock_rr)
    }

    async fn delete_history(&self) -> Result<(), String> {
        delete_history(&self.local_state);
        Ok(())
    }

    async fn ping(&self) -> Result<(), String> {
        let addr = self.addr.to_string();

        let mut stream = TcpStream::connect(&addr).await
            .map_err(|err| format!("Cannot connect to mock server: {}", err))?;

        let request = format!(
            "GET /__httpmock__/ping HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            addr
        );

        stream.write_all(request.as_bytes()).await
            .map_err(|err| format!("Cannot send request to mock server: {}", err))?;

        let mut buf = vec![0u8; 1024];
        stream.read(&mut buf).await
            .map_err(|err| format!("Cannot read response from mock server: {}", err))?;

        let response = String::from_utf8_lossy(&buf);
        if !response.contains("200 OK") {
            return Err(format!("Unexpected mock server response. Expected '{}' to contain '200 OK'", response))
        }

        Ok(())
    }
}
