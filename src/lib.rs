#[macro_use]
extern crate typed_builder;

use crate::server::router::Method::{GET, POST};
use crate::server::router::{HttpMockRouter, Route};
use crate::server::{start_server, ServerConfig};
use crate::routes::create_router;

mod routes;
mod server;

#[derive(TypedBuilder, Debug)]
pub struct HttpMockConfig {
    pub port: u16,
}

pub fn start(http_mock_config: HttpMockConfig) {
    let http_server_config = ServerConfig::builder().port(http_mock_config.port).build();
    let router = create_router();

    start_server(http_server_config, router);
}
