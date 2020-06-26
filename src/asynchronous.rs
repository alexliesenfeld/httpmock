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
    server_adapter: Arc<Arc<dyn MockServerAdapter + Send + Sync>>,
    server_adapter2: Option<Box<dyn MockServerAdapter + Send + Sync>>,
}

impl MockServer {
    async fn from(server_adapter: Arc<Arc<dyn MockServerAdapter + Send + Sync>>, server_adapter2: Box<dyn MockServerAdapter + Send + Sync>) -> Self {
        // TODO: use with_retry
        server_adapter
            .ping()
            .await
            .expect("Cannot ping mock server.");
        // TODO: use with_retry
        server_adapter
            .delete_all_mocks()
            .expect("Cannot reset mock server.");
        Self { server_adapter, server_adapter2: Some(server_adapter2) }
    }

    pub async fn new_remote_from_address(addr: SocketAddr) -> Self {
        // TODO: Remove this await and return Furture instead
        return Self::from(Arc::new(Arc::new(RemoteMockServerAdapter::new(addr))), Box::new(RemoteMockServerAdapter::new(addr))).await;
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
        return Self::from(Arc::new(Arc::new(RemoteMockServerAdapter::new(addr))), Box::new(RemoteMockServerAdapter::new(addr))).await;
    }

    pub async fn new() -> Self {
        let adapter = LOCAL_SERVER_POOL
            .get_or_create_from(LOCAL_SERVER_ADAPTER_GENERATOR)
            .await;

        let adapter2 = LOCAL_SERVER_POOL2.take(LOCAL_SERVER_ADAPTER_GENERATOR2).await;

        // TODO: remove this await and return future instead
        Self::from(adapter, adapter2).await
    }

    pub fn new_mock(&self) -> Mock {
        Mock::new(self.server_adapter.clone())
    }

    pub fn mock(&self, method: Method, path: &str) -> Mock {
        Mock::new(self.server_adapter.clone())
            .expect_method(method)
            .expect_path(path)
    }

    pub fn host(&self) -> String {
        self.server_adapter.host()
    }

    pub fn port(&self) -> u16 {
        self.server_adapter.port()
    }

    pub fn address(&self) -> &SocketAddr {
        self.server_adapter.address()
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        LOCAL_SERVER_POOL
            .put_back(self.server_adapter.clone())
            .join();
        let ssa = self.server_adapter2.take().unwrap();
        LOCAL_SERVER_POOL2.put(ssa).join();
    }
}

const LOCAL_SERVER_ADAPTER_GENERATOR2: fn() -> Box<dyn MockServerAdapter + Send + Sync> = || {
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
    return Box::new(LocalMockServerAdapter::new(addr, state));
};


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
    static ref LOCAL_SERVER_POOL: Arc<ItemPool<Arc<dyn MockServerAdapter + Send + Sync>>> = {
        let max_servers = read_env("HTTPMOCK_MAX_SERVERS", "1")
            .parse::<usize>()
            .expect("Cannot parse environment variable HTTPMOCK_MAX_SERVERS to an integer");
        return Arc::new(ItemPool::<Arc<dyn MockServerAdapter + Send + Sync>>::new(
            max_servers,
        ));
    };
    static ref LOCAL_SERVER_POOL2: Arc<Pool<Box<dyn MockServerAdapter + Send + Sync>>> = {
        let max_servers = read_env("HTTPMOCK_MAX_SERVERS", "1")
            .parse::<usize>()
            .expect("Cannot parse environment variable HTTPMOCK_MAX_SERVERS to an integer");
        return Arc::new(Pool::new(max_servers));
    };
    static ref LOCAL_CLIENT_POOL: Arc<ItemPool<Arc<InternalHttpClient>>> = {
        let max_clients = read_env("HTTPMOCK_MAX_LOCAL_CLIENTS", "30")
            .parse::<usize>()
            .expect("Cannot parse environment variable HTTPMOCK_MAX_LOCAL_CLIENTS to an integer");
        return Arc::new(ItemPool::<Arc<InternalHttpClient>>::new(max_clients));
    };
}
