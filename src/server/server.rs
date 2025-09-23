use futures_util::{stream::StreamExt, FutureExt};
use http::{Request, StatusCode};
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::body::{Bytes, Incoming};
use std::{
    future::{pending, Future},
    net::SocketAddr,
    path::PathBuf,
    sync::Arc,
};

use hyper_util::server::conn::auto::Builder as ServerBuilder;

use crate::server;
use hyper::{http, service::service_fn, upgrade::on as upgrade_on, Method, Response};
use hyper_util::rt::tokio::TokioIo;
use thiserror::Error;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::oneshot::Sender,
    task::spawn,
};

use crate::server::{
    handler::Handler,
    server::Error::{
        BufferError, LocalSocketAddrError, PublishSocketAddrError, RouterError, SocketBindError,
    },
};

use std::io;

#[cfg(feature = "https")]
use rustls::ServerConfig;
#[cfg(feature = "https")]
use tokio_rustls::TlsAcceptor;

#[derive(Error, Debug)]
pub enum Error {
    #[error("cannot bind to socket addr {0}: {1}")]
    SocketBindError(SocketAddr, std::io::Error),
    #[error("cannot parse socket address: {0}")]
    SocketAddrParseError(#[from] std::net::AddrParseError),
    #[error("cannot obtain local error: {0}")]
    LocalSocketAddrError(std::io::Error),
    #[error("cannot send reserved TCP address to test thread {0}")]
    PublishSocketAddrError(SocketAddr),
    #[error("cannot create response: {0}")]
    ResponseConstructionError(http::Error),
    #[error("buffering error: {0}")]
    BufferError(hyper::Error),
    #[error("HTTP error: {0}")]
    HTTPError(#[from] http::Error),
    #[error("cannot process request: {0}")]
    RouterError(#[from] server::handler::Error),
    #[error("HTTPS error: {0}")]
    TlsError(String),
    #[error("Server configuration error: {0}")]
    ConfigurationError(String),
    #[error("Server I/O error: {0}")]
    IOError(io::Error),
    #[error("Server error: {0}")]
    ServerError(#[from] hyper::Error),
    #[error("Server error: {0}")]
    ServerConnectionError(Box<dyn std::error::Error + Send + Sync>),
    #[error("unknown data store error")]
    Unknown,
}

#[cfg(feature = "https")]
pub struct MockServerHttpsConfig {
    pub cert_resolver_factory: Arc<dyn CertificateResolverFactory + Send + Sync>,
}

pub struct MockServerConfig {
    pub static_port: Option<u16>,
    pub expose: bool,
    pub print_access_log: bool,
    #[cfg(feature = "https")]
    pub https: MockServerHttpsConfig,
}

/// The `MockServer` struct represents a mock server that can handle incoming HTTP requests.
pub struct MockServer<H>
where
    H: Handler + Send + Sync + 'static,
{
    handler: Box<H>,
    config: MockServerConfig,
}

impl<H> MockServer<H>
where
    H: Handler + Send + Sync + 'static,
{
    /// Creates a new `MockServer` instance with the given handler and configuration.
    ///
    /// # Parameters
    /// - `handler`: A boxed handler that implements the `Handler` trait.
    /// - `config`: The configuration settings for the mock server.
    ///
    /// # Returns
    /// A `Result` containing the new `MockServer` instance or an `Error` if creation fails.
    pub fn new(handler: Box<H>, config: MockServerConfig) -> Result<Self, Error> {
        Ok(MockServer { handler, config })
    }

    /// Starts the mock server asynchronously.
    pub async fn start(self) -> Result<(), Error> {
        self.start_with_signals(None, pending()).await
    }

    /// Starts the mock server asynchronously with support for handling external shutdown signals.
    ///
    /// # Parameters
    /// - `socket_addr_sender`: An optional `Sender` to send the server's socket address once it's bound.
    /// - `shutdown`: A future that resolves when the server should shut down.
    ///
    pub async fn start_with_signals<F>(
        self,
        socket_addr_sender: Option<Sender<SocketAddr>>,
        shutdown: F,
    ) -> Result<(), Error>
    where
        F: Future<Output = ()>,
    {
        let host = if self.config.expose {
            "0.0.0.0"
        } else {
            "127.0.0.1"
        };
        let addr: SocketAddr =
            format!("{}:{}", host, self.config.static_port.unwrap_or(0)).parse()?;
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| SocketBindError(addr, e))?;

        if let Some(sender) = socket_addr_sender {
            let addr = listener.local_addr().map_err(|e| LocalSocketAddrError(e))?;
            sender
                .send(addr)
                .map_err(|addr| PublishSocketAddrError(addr))?;
        }

        // ****************************************************************************************
        // SERVER START
        log::info!("Listening on {}", addr);
        self.run_accept_loop(listener, shutdown).await
    }

    pub async fn run_accept_loop<F>(self, listener: TcpListener, shutdown: F) -> Result<(), Error>
    where
        F: Future<Output = ()>,
    {
        let shutdown = shutdown.shared();
        let server = Arc::new(self);

        loop {
            tokio::select! {
                accepted = listener.accept() => {
                    match accepted {
                        Ok((tcp_stream, remote_address)) => {
                            let server = server.clone();
                            spawn(async move {
                               if let Err(err) = server.handle_tcp_stream(tcp_stream, remote_address).await {
                                    log::error!("{:?}", err);
                                }
                            });
                        },
                        Err(err) =>  {
                            log::error!("TCP error: {:?}", err);
                        },
                    };
                }
                _ = shutdown.clone() => {
                    break;
                }
            }
        }

        Ok(())
    }

    async fn service(
        self: Arc<Self>,
        req: Request<Incoming>,
    ) -> Result<Response<BoxBody<Bytes, hyper::Error>>, Error> {
        log::trace!("New HTTP request received: {}", req.uri());

        if req.method() == Method::CONNECT {
            #[cfg(feature = "https")]
            {
                // CONNECT handling (HTTP proxy tunneling):
                //
                // For a CONNECT request the client asks us to turn the HTTP connection into a
                // raw, bidirectional byte tunnel (e.g., for TLS-over-HTTP proxies).
                //
                // In Hyper, the upgrade to this raw mode only happens AFTER we return the
                // 200 Connection Established response so the server can flush it and switch
                // the underlying socket into “upgraded” mode. The future from
                // `hyper::upgrade::on(req)` (captured as `on_upgrade`) will only resolve once
                // that response is sent and the connection is upgraded.
                //
                // If we awaited `on_upgrade` inline here, we would deadlock/block the upgrade:
                //   - We would wait for the upgrade to complete, but Hyper can’t complete it
                //     until we first return 200.
                //
                // Therefore we must detach the upgrade handling from this request future. We
                // do that by spawning a background task which:
                //   1) awaits `on_upgrade` once Hyper has upgraded the connection, and
                //   2) runs the tunnel logic in `handle_connect_upgraded`.
                //
                // Meanwhile, we immediately return `200 Connection Established` (with an empty
                // body) to let Hyper proceed with the upgrade.

                let authority = req.uri().authority().map(|a| a.to_string());
                let on_upgrade = upgrade_on(req);
                let this = self.clone();

                spawn(async move {
                    match on_upgrade.await {
                        Ok(upgraded) => {
                            let io = TokioIo::new(upgraded);
                            if let Err(e) = this.serve_tls_connection(io, authority).await {
                                log::warn!("failed to server upgraded TLS connection: {:?}", e);
                            }
                        }
                        Err(err) => {
                            let e = crate::server::server::Error::ServerConnectionError(Box::new(err));
                            log::warn!("CONNECT upgraded handling failed: {:?}", e);
                        }
                    }
                });
            }

            // Respond with 200 Connection Established and empty body
            return Ok(Response::builder().status(StatusCode::OK).body(empty())?);
        }

        let mut req = match buffer_request(req).await {
            Ok(req) => req,
            Err(err) => {
                return error_response(StatusCode::INTERNAL_SERVER_ERROR, BufferError(err));
            }
        };


        // Normalize the request URI to absolute-form for both HTTP and HTTPS.
        //
        // Clients typically send origin-form ("/path") with a Host header; after a
        // CONNECT+TLS MITM this is also what the browser sends to us. We normalize to
        // absolute-form (scheme://authority/path?query) so that:
        // - matchers and the recorder can reliably read scheme/host/port from req.uri();
        // - we can avoid ad-hoc Host-header parsing throughout the codebase.
        //
        // See handler::proxy() for the inverse step where we convert back to
        // origin-form right before sending the request upstream, since most origin
        // servers (HTTP/1.1 and HTTP/2) expect origin-form on the wire.
        // TODO: Rather than normalizing to absolute-form here, we should consider
        //       enhancing the RequestMetadata to carry the scheme/authority separately
        if let Err(err) = normalize_absolute_uri(&mut req) {
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, err);
        }

        match self.handler.handle(req).await {
            Ok(response) => to_service_response(response),
            Err(err) => error_response(StatusCode::INTERNAL_SERVER_ERROR, RouterError(err)),
        }
    }

    async fn handle_tcp_stream(
        self: Arc<Self>,
        tcp_stream: TcpStream,
        remote_address: SocketAddr,
    ) -> Result<(), Error> {
        log::trace!("new TCP connection incoming");

        #[cfg(feature = "https")]
        {
            let mut peek_buffer = TcpStreamPeekBuffer::new(&tcp_stream);
            if is_encrypted(&mut peek_buffer, 0).await {
                log::trace!("TCP connection seems to be TLS encrypted");

                let tcp_address = tcp_stream.local_addr().map_err(|err| IOError(err))?;
                return self
                    .clone()
                    .serve_tls_connection(tcp_stream, Some(tcp_address.to_string()))
                    .await;
            }

            if log::max_level() >= log::LevelFilter::Trace {
                let peeked_str =
                    String::from_utf8_lossy(&peek_buffer.buffer().to_vec()).to_string();
                log::trace!(
                    "TCP connection seems NOT to be TLS encrypted (based on peeked data: {}",
                    peeked_str
                );
            }
        }

        log::trace!("TCP connection is not TLS encrypted");
        serve_connection(self.clone(), tcp_stream, "http").await
    }

    #[cfg(feature = "https")]
    async fn serve_tls_connection<S>(
        self: Arc<Self>,
        stream: S,
        authority: Option<String>,
    ) -> Result<(), Error>
    where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        // Build TLS acceptor inline for this connection
        let cert_resolver = self.config.https.cert_resolver_factory.build(authority);
        let mut server_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_cert_resolver(cert_resolver);

        #[cfg(feature = "http2")]
        {
            server_config.alpn_protocols =
                vec![b"h2".to_vec(), b"http/1.1".to_vec(), b"http/1.0".to_vec()];
        }
        #[cfg(not(feature = "http2"))]
        {
            server_config.alpn_protocols = vec![b"http/1.1".to_vec(), b"http/1.0".to_vec()];
        }

        let tls_acceptor = TlsAcceptor::from(Arc::new(server_config));

        let tls_stream = tls_acceptor
            .accept(stream)
            .await
            .map_err(|e| TlsError(format!("TLS accept failed: {:?}", e)))?;

        serve_connection(self, tls_stream, "https").await
    }

}

async fn serve_connection<H, S>(
    server: Arc<MockServer<H>>,
    stream: S,
    scheme: &'static str,
) -> Result<(), Error>
where
    H: Handler + Send + Sync + 'static,
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let mut server_builder = ServerBuilder::new(TokioExecutor::new());

    server_builder.http1().preserve_header_case(true);
    server_builder.http2();
    //.enable_connect_protocol();

    server_builder
        .serve_connection_with_upgrades(
            TokioIo::new(stream),
            service_fn(|mut req| {
                req.extensions_mut().insert(RequestMetadata::new(scheme));
                server.clone().service(req)
            }),
        )
        .await
        .map_err(|err| ServerConnectionError(err))
}

async fn handle_connect(
    req: Request<Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, Error> {
    if let Some(addr) = host_addr(req.uri()) {
        spawn(async move {
            match upgrade_on(req).await {
                Ok(upgraded) => {
                    if let Err(e) = tunnel(upgraded, addr).await {
                        log::warn!("Proxy I/O error: {}", e);
                    } else {
                        log::info!("Proxied request");
                    };
                }
                Err(e) => {
                    log::warn!("Proxy upgrade error: {}", e)
                }
            }
        });

        Ok(Response::new(empty()))
    } else {
        log::warn!("CONNECT host is not socket addr: {:?}", req.uri());
        let mut resp = Response::new(full("CONNECT must be sent to a socket address"));
        *resp.status_mut() = StatusCode::BAD_REQUEST;

        Ok(resp)
    }
}

async fn buffer_request(req: Request<Incoming>) -> Result<Request<Bytes>, hyper::Error> {
    let (parts, body) = req.into_parts();
    let body = body.collect().await?.to_bytes();
    Ok(Request::from_parts(parts, body))
}

fn host_addr(uri: &http::Uri) -> Option<String> {
    uri.authority().and_then(|auth| Some(auth.to_string()))
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

async fn tunnel(upgraded: hyper::upgrade::Upgraded, addr: String) -> std::io::Result<()> {
    let mut server = tokio::net::TcpStream::connect(addr).await?;
    let mut upgraded = RecordingStream::new(TokioIo::new(upgraded));

    let (from_client, from_server) =
        tokio::io::copy_bidirectional(&mut server, &mut upgraded).await?;

    log::info!(
        "client wrote {} bytes and received {} bytes. \n\nread:\n{}\n\n wrote: {}\n\n",
        from_client,
        from_server,
        String::from_utf8_lossy(&upgraded.read_bytes),
        String::from_utf8_lossy(&upgraded.written_bytes)
    );

    Ok(())
}

fn error_response(
    code: StatusCode,
    err: Error,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, Error> {
    log::error!("failed to process request: {}", err.to_string());
    Ok(Response::builder()
        .status(code)
        .body(full(err.to_string()))?)
}

fn to_service_response(
    response: Response<Bytes>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, Error> {
    let (parts, body) = response.into_parts();
    Ok(Response::from_parts(parts, full(body)))
}

use crate::server::Error::{IOError, ServerConnectionError, ServerError, TlsError, Unknown};
use async_trait::async_trait;
use bytes::BytesMut;
use hyper_util::rt::TokioExecutor;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

#[cfg(feature = "https")]
use crate::server::tls::{CertificateResolverFactory, TcpStreamPeekBuffer};

use crate::server::RequestMetadata;
#[cfg(feature = "https")]
use tls_detect::is_encrypted;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

struct RecordingStream<S> {
    stream: S,
    read_bytes: BytesMut,    // Buffer to store bytes read from the stream
    written_bytes: BytesMut, // Buffer to store bytes written to the stream
}

impl<S: AsyncRead + AsyncWrite + Unpin> RecordingStream<S> {
    pub fn new(stream: S) -> Self {
        RecordingStream {
            stream,
            read_bytes: BytesMut::new(),
            written_bytes: BytesMut::new(),
        }
    }

    // Method to access the collected read bytes
    pub fn get_read_bytes(&self) -> &[u8] {
        &self.read_bytes
    }

    // Method to access the collected written bytes
    pub fn get_written_bytes(&self) -> &[u8] {
        &self.written_bytes
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> AsyncRead for RecordingStream<S> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        let stream = Pin::new(&mut this.stream);

        let before = buf.filled().len();
        match stream.poll_read(cx, buf) {
            Poll::Ready(Ok(())) => {
                let after = buf.filled().len();
                let new_bytes = &buf.filled()[before..after];
                this.read_bytes.extend_from_slice(new_bytes);
                Poll::Ready(Ok(()))
            }
            other => other,
        }
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> AsyncWrite for RecordingStream<S> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        let this = self.get_mut();
        let stream = Pin::new(&mut this.stream);

        match stream.poll_write(cx, buf) {
            Poll::Ready(Ok(size)) => {
                this.written_bytes.extend_from_slice(&buf[..size]);
                Poll::Ready(Ok(size))
            }
            other => other,
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.get_mut().stream).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.get_mut().stream).poll_shutdown(cx)
    }
}

fn normalize_absolute_uri(
    req: &mut http::Request<Bytes>,
) -> Result<(), Error> {
    let default_scheme = req
        .extensions()
        .get::<RequestMetadata>()
        .map(|m| m.scheme)
        .unwrap_or("http");

    // If already absolute-form (scheme + authority), leave as-is
    let uri = req.uri().clone();
    if uri.scheme().is_some() && uri.authority().is_some() {
        return Ok(());
    }

    // Use Host header to reconstruct the absolute URI
    let host = req
        .headers()
        .get(http::header::HOST)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            Error::ConfigurationError("Missing Host header on origin-form request".into())
        })?;

    let path_and_query = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");

    let abs = format!("{}://{}{}", default_scheme, host, path_and_query);
    let new_uri: http::Uri = abs.parse().map_err(|e| {
        Error::ConfigurationError(format!(
            "Invalid absolute URI constructed from Host+path: {}",
            e
        ))
    })?;

    *req.uri_mut() = new_uri;
    Ok(())
}
