extern crate lazy_static;

use crate::handlers::mocks::SetMockRequest;
use crate::handlers::{HttpMockRequest, HttpMockResponse};
use crate::{start_server, HttpMockConfig};

use crate::api::Scheme::{HTTP, HTTPS};
pub use crate::handlers::Scheme;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt;
use std::fmt::format;
use std::io::Read;
use std::sync::{LockResult, Mutex, MutexGuard, PoisonError};
use std::thread::{self, JoinHandle};

lazy_static! {
    pub static ref SERVER: Mutex<JoinHandle<()>> = {
        let server_thread = thread::spawn(move || {
            let config = HttpMockConfig::builder()
                .port(5000 as u16)
                .workers(3 as usize)
                .build();

            start_server(config);
        });
        return Mutex::new(server_thread);
    };
}

thread_local!(
    static SERVER_GUARD: RefCell<LockResult<MutexGuard<'static, JoinHandle<()>>>> =
        RefCell::new(SERVER.lock());
);

#[derive(Debug)]
pub struct Mock {
    server_host: String,
    server_port: u16,
    client: reqwest::Client,
    mock: SetMockRequest,
}

impl Mock {
    pub fn server_port(&self) -> u16 {
        self.server_port
    }

    pub fn server_host(&self) -> &String {
        &self.server_host
    }

    pub fn path(mut self, path: &str) -> Self {
        self.mock.request.path = Some(path.to_string());
        self
    }

    pub fn method(mut self, method: Method) -> Self {
        self.mock.request.method = Some(method.as_str().to_string());
        self
    }

    pub fn return_status(mut self, status: usize) -> Self {
        self.mock.response.status = status as u16;
        self
    }

    pub fn return_body(mut self, contents: &str) -> Self {
        self.mock.response.body = Some(contents.to_string());
        self
    }

    pub fn expect_body(mut self, contents: &str) -> Self {
        self.mock.request.body = Some(contents.to_string());
        self
    }

    pub fn return_header(mut self, key: &str, value: &str) -> Self {
        if self.mock.response.headers.is_none() {
            self.mock.response.headers = Some(BTreeMap::new());
        }
        self.mock
            .response
            .headers
            .as_mut()
            .unwrap()
            .insert(key.to_string(), value.to_string());
        self
    }

    pub fn expect_scheme(mut self, scheme: Scheme) -> Self {
        self.mock.request.scheme = Some(scheme);
        self
    }

    pub fn expect_header(mut self, key: &str, value: &str) -> Self {
        if self.mock.request.headers.is_none() {
            self.mock.request.headers = Some(BTreeMap::new());
        }

        self.mock
            .request
            .headers
            .as_mut()
            .unwrap()
            .insert(key.to_string(), value.to_string());

        self
    }

    pub fn create(&self) -> &Self {
        SERVER_GUARD.with(|_| {}); // Prevents tests run in parallel

        let json = serde_json::to_string(&self.mock).expect("cannot serialize request");
        let request_url = format!("http://{}/__mocks", &self.server_address());
        let mut res = self
            .client
            .post(request_url.as_str())
            .body(json)
            .send()
            .expect("Mock server error");

        if res.status() != 201 {
            let mut buf = String::new();
            res.read_to_string(&mut buf)
                .expect("Failed to read response");
            let err_msg = format!(
                "Could not create mock (status = {}, message = {})",
                res.status(),
                buf
            );
            panic!(err_msg);
        }

        self
    }

    pub fn number_of_calls(&self) -> Result<usize, String> {
        Ok(5 as usize)
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}

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

impl Method {
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::GET => "GET",
            Method::HEAD => "HEAD",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::DELETE => "DELETE",
            Method::CONNECT => "CONNECT",
            Method::OPTIONS => "OPTIONS",
            Method::TRACE => "TRACE",
            Method::PATCH => "PATCH",
        }
    }
}

pub fn mock(method: Method, path: &str) -> Mock {
    Mock {
        server_port: 5000,
        server_host: "localhost".to_string(),
        client: reqwest::Client::new(),
        mock: SetMockRequest {
            request: HttpMockRequest {
                scheme: None,
                method: Some(method.as_str().to_string()),
                path: Some(String::from(path)),
                headers: None,
                body: None,
            },
            response: HttpMockResponse {
                status: 200,
                status_message: None,
                headers: None,
                body: None,
            },
        },
    }
}
