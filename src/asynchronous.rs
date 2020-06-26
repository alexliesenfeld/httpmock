use crate::api::{LocalMockServerAdapter, MockServerAdapter, RemoteMockServerAdapter};
use crate::pool::ItemPool;
use crate::server::data::{MockServerHttpRequest, MockServerState};
use crate::server::{start_server, HttpMockConfig};
use crate::util::{read_env, with_retry, Join};
use crate::{Method, Mock};
use isahc::prelude::Configurable;
use std::net::{SocketAddr, ToSocketAddrs};
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::task::LocalSet;
use crate::new_pool::Pool;

pub(crate) type InternalHttpClient = isahc::HttpClient;

pub struct MockServer {
    server_adapter: Option<Arc<dyn MockServerAdapter + Send + Sync>>,
}

impl MockServer {
    async fn from(server_adapter: Arc<dyn MockServerAdapter + Send + Sync>) -> Self {
        // TODO: use with_retry
        server_adapter
            .ping()
            .await
            .expect("Cannot ping mock server.");
        // TODO: use with_retry
        server_adapter
            .delete_all_mocks()
            .expect("Cannot reset mock server.");
        Self { server_adapter: Some(server_adapter) }
    }

    pub async fn new_remote_from_address(addr: SocketAddr) -> Self {
        // TODO: Remove this await and return Furture instead
        return Self::from(Arc::new(RemoteMockServerAdapter::new(addr))).await;
    }

    pub async fn new_remote() -> Self {
        let host = read_env("HTTPMOCK_HOST", "127.0.0.1");
        let port = read_env("HTTPMOCK_PORT", "5000")
            .parse::<u16>()
            .expect("Cannot parse port from environment variable HTTPMOCK_PORT");

        let addr = format!("{}:{}", host, port)
            .to_socket_addrs()
            .expect("Cannot parse mock server address")
            .next()
            .expect("Cannot find mock server address in user input");

        // TODO: Remove this await and return Future instead
        return Self::from(Arc::new(RemoteMockServerAdapter::new(addr))).await;
    }

    pub async fn new() -> Self {
        let adapter2 = LOCAL_SERVER_POOL.take(LOCAL_SERVER_ADAPTER_GENERATOR).await;
        Self::from(adapter2).await
    }

    pub fn new_mock(&self) -> Mock {
        Mock::new(self.server_adapter.clone().unwrap())
    }

    pub fn mock(&self, method: Method, path: &str) -> Mock {
        Mock::new(self.server_adapter.clone().unwrap())
            .expect_method(method)
            .expect_path(path)
    }

    pub fn host(&self) -> String {
        self.server_adapter.as_ref().unwrap().host()
    }

    pub fn port(&self) -> u16 {
        self.server_adapter.as_ref().unwrap().port()
    }

    pub fn address(&self) -> &SocketAddr {
        self.server_adapter.as_ref().unwrap().address()
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        let adapter = self.server_adapter.take().unwrap();
        LOCAL_SERVER_POOL.put(adapter).join();
    }
}

const LOCAL_SERVER_ADAPTER_GENERATOR: fn() -> Arc<dyn MockServerAdapter + Send + Sync> = || {
    let (addr_sender, addr_receiver) = tokio::sync::oneshot::channel::<SocketAddr>();
    let state = Arc::new(MockServerState::new());
    let server_state = state.clone();

    thread::spawn(move || {
        let config = HttpMockConfig::new(0, false);
        let server_state = server_state.clone();

        let srv = start_server(config, &server_state, None, Some(addr_sender));

        let mut runtime = tokio::runtime::Builder::new()
            .enable_all()
            .basic_scheduler()
            .build()
            .expect("Cannot build local tokio runtime");

        return LocalSet::new().block_on(&mut runtime, srv);
    });

    // TODO: replace this join by await
    let addr = addr_receiver.join().expect("Cannot get server address");
    return Arc::new(LocalMockServerAdapter::new(addr, state));
};

lazy_static! {
    static ref LOCAL_SERVER_POOL: Arc<Pool<Arc<dyn MockServerAdapter + Send + Sync>>> = {
        let max_servers = read_env("HTTPMOCK_MAX_SERVERS", "1")
            .parse::<usize>()
            .expect("Cannot parse environment variable HTTPMOCK_MAX_SERVERS to an integer");
        return Arc::new(Pool::new(max_servers));
    };
}
