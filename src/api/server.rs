use crate::api::spec::{Then, When};
#[cfg(feature = "remote")]
use crate::api::RemoteMockServerAdapter;
use crate::api::{LocalMockServerAdapter, MockServerAdapter};
use crate::common::data::{MockDefinition, MockServerHttpResponse, RequestRequirements};
use crate::common::util::{read_env, with_retry, Join};
use crate::server::{start_server, MockServerState};
use crate::Mock;
use async_object_pool::Pool;
use std::cell::Cell;
use std::future::pending;
use std::net::{SocketAddr, ToSocketAddrs};
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use tokio::task::LocalSet;

/// A mock server that is able to receive and respond to HTTP requests.
pub struct MockServer {
    pub(crate) server_adapter: Option<Arc<dyn MockServerAdapter + Send + Sync>>,
    pool: Arc<Pool<Arc<dyn MockServerAdapter + Send + Sync>>>,
}

impl MockServer {
    async fn from(
        server_adapter: Arc<dyn MockServerAdapter + Send + Sync>,
        pool: Arc<Pool<Arc<dyn MockServerAdapter + Send + Sync>>>,
    ) -> Self {
        let server = Self {
            server_adapter: Some(server_adapter),
            pool,
        };

        server.reset_async().await;

        return server;
    }

    /// Asynchronously connects to a remote mock server that is running in standalone mode using
    /// the provided address of the form <host>:<port> (e.g. "127.0.0.1:8080") to establish
    /// the connection.
    /// **Note**: This method requires the feature `remote` to be enabled.
    #[cfg(feature = "remote")]
    pub async fn connect_async(address: &str) -> Self {
        let addr = address
            .to_socket_addrs()
            .expect("Cannot parse address")
            .find(|addr| addr.is_ipv4())
            .expect("Not able to resolve the provided host name to an IPv4 address");

        let adapter = REMOTE_SERVER_POOL_REF
            .take_or_create(|| Arc::new(RemoteMockServerAdapter::new(addr)))
            .await;
        Self::from(adapter, REMOTE_SERVER_POOL_REF.clone()).await
    }

    /// Synchronously connects to a remote mock server that is running in standalone mode using
    /// the provided address of the form <host>:<port> (e.g. "127.0.0.1:8080") to establish
    /// the connection.
    /// **Note**: This method requires the feature `remote` to be enabled.
    #[cfg(feature = "remote")]
    pub fn connect(address: &str) -> Self {
        Self::connect_async(address).join()
    }

    /// Asynchronously connects to a remote mock server that is running in standalone mode using
    /// connection parameters stored in `HTTPMOCK_HOST` and `HTTPMOCK_PORT` environment variables.
    /// **Note**: This method requires the feature `remote` to be enabled.
    #[cfg(feature = "remote")]
    pub async fn connect_from_env_async() -> Self {
        let host = read_env("HTTPMOCK_HOST", "127.0.0.1");
        let port = read_env("HTTPMOCK_PORT", "5000")
            .parse::<u16>()
            .expect("Cannot parse environment variable HTTPMOCK_PORT to an integer");
        Self::connect_async(&format!("{}:{}", host, port)).await
    }

    /// Synchronously connects to a remote mock server that is running in standalone mode using
    /// connection parameters stored in `HTTPMOCK_HOST` and `HTTPMOCK_PORT` environment variables.
    /// **Note**: This method requires the feature `remote` to be enabled.
    #[cfg(feature = "remote")]
    pub fn connect_from_env() -> Self {
        Self::connect_from_env_async().join()
    }

    /// Starts a new `MockServer` asynchronously.
    ///
    /// Attention: This library manages a pool of `MockServer` instances in the background.
    /// Instead of always starting a new mock server, a `MockServer` instance is only created
    /// on demand if there is no free `MockServer` instance in the pool and the pool has not
    /// reached a maximum size yet. Otherwise, *THIS METHOD WILL BLOCK* the executing function
    /// until a free mock server is available.
    ///
    /// This allows to run many tests in parallel, but will prevent exhaust the executing
    /// machine by creating too many mock servers.
    ///
    /// A `MockServer` instance is automatically taken from the pool whenever this method is called.
    /// The instance is put back into the pool automatically when the corresponding
    /// 'MockServer' variable gets out of scope.
    pub async fn start_async() -> Self {
        let adapter = LOCAL_SERVER_POOL_REF
            .take_or_create(LOCAL_SERVER_ADAPTER_GENERATOR)
            .await;
        Self::from(adapter, LOCAL_SERVER_POOL_REF.clone()).await
    }

    /// Starts a new `MockServer` synchronously.
    ///
    /// Attention: This library manages a pool of `MockServer` instances in the background.
    /// Instead of always starting a new mock server, a `MockServer` instance is only created
    /// on demand if there is no free `MockServer` instance in the pool and the pool has not
    /// reached a maximum size yet. Otherwise, *THIS METHOD WILL BLOCK* the executing function
    /// until a free mock server is available.
    ///
    /// This allows to run many tests in parallel, but will prevent exhaust the executing
    /// machine by creating too many mock servers.
    ///
    /// A `MockServer` instance is automatically taken from the pool whenever this method is called.
    /// The instance is put back into the pool automatically when the corresponding
    /// 'MockServer' variable gets out of scope.
    pub fn start() -> MockServer {
        Self::start_async().join()
    }

    /// The hostname of the `MockServer`. By default, this is `127.0.0.1`.
    /// In standalone mode, the hostname will be the host where the standalone mock server is
    /// running.
    pub fn host(&self) -> String {
        self.server_adapter.as_ref().unwrap().host()
    }

    /// The TCP port that the mock server is listening on.
    pub fn port(&self) -> u16 {
        self.server_adapter.as_ref().unwrap().port()
    }

    /// Builds the address for a specific path on the mock server.
    ///
    /// **Example**:
    /// ```
    /// // Start a local mock server for exclusive use by this test function.
    /// let server = httpmock::MockServer::start();
    ///
    /// let expected_addr_str = format!("127.0.0.1:{}", server.port());
    ///
    /// // Get the address of the MockServer.
    /// let addr = server.address();
    ///
    /// // Ensure the returned URL is as expected
    /// assert_eq!(expected_addr_str, addr.to_string());
    /// ```
    pub fn address(&self) -> &SocketAddr {
        self.server_adapter.as_ref().unwrap().address()
    }

    /// Builds the URL for a specific path on the mock server.
    ///
    /// **Example**:
    /// ```
    /// // Start a local mock server for exclusive use by this test function.
    /// let server = httpmock::MockServer::start();
    ///
    /// let expected_url = format!("http://127.0.0.1:{}/hello", server.port());
    ///
    /// // Get the URL for path "/hello".
    /// let url = server.url("/hello");
    ///
    /// // Ensure the returned URL is as expected
    /// assert_eq!(expected_url, url);
    /// ```
    pub fn url<S: Into<String>>(&self, path: S) -> String {
        format!("http://{}{}", self.address(), path.into())
    }

    /// Builds the base URL for the mock server.
    ///
    /// **Example**:
    /// ```
    /// // Start a local mock server for exclusive use by this test function.
    /// let server = httpmock::MockServer::start();
    ///
    /// let expected_url = format!("http://127.0.0.1:{}", server.port());
    ///
    /// // Get the URL for path "/hello".
    /// let url = server.base_url();
    ///
    /// // Ensure the returned URL is as expected
    /// assert_eq!(expected_url, url);
    /// ```
    pub fn base_url(&self) -> String {
        self.url("")
    }

    /// Creates a [Mock](struct.Mock.html) object on the mock server.
    ///
    /// **Example**:
    /// ```
    /// use isahc::get;
    ///
    /// let server = httpmock::MockServer::start();
    ///
    /// let mock = server.mock(|when, then| {
    ///     when.path("/hello");
    ///     then.status(200);
    /// });
    ///
    /// get(server.url("/hello")).unwrap();
    ///
    /// mock.assert();
    /// ```
    pub fn mock<F>(&self, config_fn: F) -> Mock
    where
        F: FnOnce(When, Then),
    {
        self.mock_async(config_fn).join()
    }

    /// Creates a [Mock](struct.Mock.html) object on the mock server.
    ///
    /// **Example**:
    /// ```
    /// use isahc::{get_async};
    /// async_std::task::block_on(async {
    ///     let server = httpmock::MockServer::start();
    ///
    ///     let mock = server
    ///         .mock_async(|when, then| {
    ///             when.path("/hello");
    ///             then.status(200);
    ///         })
    ///         .await;
    ///
    ///     get_async(server.url("/hello")).await.unwrap();
    ///
    ///     mock.assert_async().await;
    /// });
    /// ```
    pub async fn mock_async<'a, F>(&'a self, spec_fn: F) -> Mock<'a>
    where
        F: FnOnce(When, Then),
    {
        let mut req = Rc::new(Cell::new(RequestRequirements::new()));
        let mut res = Rc::new(Cell::new(MockServerHttpResponse::new()));

        spec_fn(
            When {
                expectations: req.clone(),
            },
            Then {
                response_template: res.clone(),
            },
        );

        let response = self
            .server_adapter
            .as_ref()
            .unwrap()
            .create_mock(&MockDefinition {
                request: req.take(),
                response: res.take(),
            })
            .await
            .expect("Cannot deserialize mock server response");

        Mock {
            id: response.mock_id,
            server: self,
        }
    }

    /// Resets the mock server. More specifically, it deletes all [Mock](struct.Mock.html) objects
    /// from the mock server and clears its request history.
    ///
    /// **Example**:
    /// ```
    /// use isahc::get;
    /// let server = httpmock::MockServer::start();
    ///
    ///  let mock = server.mock(|when, then| {
    ///     when.path("/hello");
    ///     then.status(200);
    ///  });
    ///
    ///  let mut response = get(server.url("/hello")).unwrap();
    ///  assert_eq!(response.status(), 200);
    ///
    ///  server.reset();
    ///
    ///  let mut response = get(server.url("/hello")).unwrap();
    ///  assert_eq!(response.status(), 404);
    /// ```
    pub fn reset(&self) {
        self.reset_async().join()
    }

    /// Resets the mock server. More specifically, it deletes all [Mock](struct.Mock.html) objects
    /// from the mock server and clears its request history.
    ///
    /// **Example**:
    /// ```
    /// use isahc::get;
    /// async_std::task::block_on(async {
    ///     let server = httpmock::MockServer::start_async().await;
    ///
    ///     let mock = server.mock_async(|when, then| {
    ///        when.path("/hello");
    ///        then.status(200);
    ///     }).await;
    ///
    ///     let mut response = get(server.url("/hello")).unwrap();
    ///     assert_eq!(response.status(), 200);
    ///
    ///     server.reset_async().await;
    ///
    ///     let mut response = get(server.url("/hello")).unwrap();
    ///     assert_eq!(response.status(), 404);
    /// });
    /// ```
    pub async fn reset_async(&self) {
        if let Some(server_adapter) = &self.server_adapter {
            with_retry(5, || server_adapter.delete_all_mocks())
                .await
                .expect("Cannot reset mock server (task: delete mocks).");
            with_retry(5, || server_adapter.delete_history())
                .await
                .expect("Cannot reset mock server (task: delete request history).");
        }
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        let adapter = self.server_adapter.take().unwrap();
        self.pool.put(adapter).join();
    }
}

const LOCAL_SERVER_ADAPTER_GENERATOR: fn() -> Arc<dyn MockServerAdapter + Send + Sync> = || {
    let (addr_sender, addr_receiver) = tokio::sync::oneshot::channel::<SocketAddr>();
    let state = Arc::new(MockServerState::default());
    let server_state = state.clone();

    thread::spawn(move || {
        let server_state = server_state.clone();
        let srv = start_server(0, false, &server_state, Some(addr_sender), false, pending());

        let mut runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Cannot build local tokio runtime");

        LocalSet::new().block_on(&mut runtime, srv)
    });

    let addr = addr_receiver.join().expect("Cannot get server address");
    Arc::new(LocalMockServerAdapter::new(addr, state))
};

lazy_static! {
    static ref LOCAL_SERVER_POOL_REF: Arc<Pool<Arc<dyn MockServerAdapter + Send + Sync>>> = {
        let max_servers = read_env("HTTPMOCK_MAX_SERVERS", "25")
            .parse::<usize>()
            .expect("Cannot parse environment variable HTTPMOCK_MAX_SERVERS to an integer");
        Arc::new(Pool::new(max_servers))
    };
    static ref REMOTE_SERVER_POOL_REF: Arc<Pool<Arc<dyn MockServerAdapter + Send + Sync>>> =
        Arc::new(Pool::new(1));
}
