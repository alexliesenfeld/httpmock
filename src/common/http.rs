use async_trait::async_trait;
use bytes::Bytes;
use http::{Request, Response};
use http_body_util::{BodyExt, Full};
#[cfg(any(feature = "remote-https", feature = "https"))]
use hyper_rustls::HttpsConnector;
use hyper_util::{
    client::legacy::{connect::HttpConnector, Client},
    rt::TokioExecutor,
};
use std::{convert::TryInto, sync::Arc};
use thiserror::Error;
use tokio::runtime::Runtime;

#[derive(Error, Debug)]
pub enum Error {
    #[error("cannot send request: {0}")]
    HyperError(#[from] hyper::Error),
    #[error("cannot send request: {0}")]
    HyperUtilError(#[from] hyper_util::client::legacy::Error),
    #[error("runtime error: {0}")]
    RuntimeError(#[from] tokio::task::JoinError),
    #[error("unknown error")]
    Unknown,
}

#[async_trait]
pub trait HttpClient {
    async fn send(&self, req: Request<Bytes>) -> Result<Response<Bytes>, Error>;
}

pub struct HttpMockHttpClient {
    runtime: Option<Arc<Runtime>>,
    #[cfg(any(feature = "remote-https", feature = "https"))]
    client: Arc<Client<HttpsConnector<HttpConnector>, Full<Bytes>>>,
    #[cfg(not(any(feature = "remote-https", feature = "https")))]
    client: Arc<Client<HttpConnector, Full<Bytes>>>,
}

impl<'a> HttpMockHttpClient {
    #[cfg(any(feature = "remote-https", feature = "https"))]
    pub fn new(runtime: Option<Arc<Runtime>>) -> Self {
        // see https://github.com/rustls/rustls/issues/1938
        if rustls::crypto::CryptoProvider::get_default().is_none() {
            rustls::crypto::ring::default_provider()
                .install_default()
                .expect("cannot install rustls crypto provider");
        }

        let https_connector = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .expect("cannot set up using native root certificates")
            .https_or_http()
            .enable_all_versions()
            .build();

        Self {
            runtime,
            client: Arc::new(Client::builder(TokioExecutor::new()).build(https_connector)),
        }
    }

    #[cfg(not(any(feature = "remote-https", feature = "https")))]
    pub fn new(runtime: Option<Arc<Runtime>>) -> Self {
        Self {
            runtime,
            client: Arc::new(Client::builder(TokioExecutor::new()).build(HttpConnector::new())),
        }
    }
}

#[async_trait]
impl HttpClient for HttpMockHttpClient {
    async fn send(&self, req: Request<Bytes>) -> Result<Response<Bytes>, Error> {
        let (req_parts, req_body) = req.into_parts();
        let hyper_req = Request::from_parts(req_parts, Full::new(req_body));

        let res = if let Some(rt) = self.runtime.clone() {
            let client = self.client.clone();
            rt.spawn(async move { client.request(hyper_req).await })
                .await??
        } else {
            self.client.request(hyper_req).await?
        };

        let (res_parts, res_body) = res.into_parts();
        let body = res_body.collect().await?.to_bytes();

        return Ok(Response::from_parts(res_parts, body));
    }
}
