#[macro_use]
extern crate typed_builder;

use crate::server::{start_server, ServerConfig};

mod server;

#[derive(TypedBuilder, Debug)]
pub struct HttpMockConfig {
    pub port: u16,
}

pub fn start(http_mock_config: HttpMockConfig) {
    let http_server_config = ServerConfig::builder().port(http_mock_config.port).build();

    start_server(http_server_config);
}
