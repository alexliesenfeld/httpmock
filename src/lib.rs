#[macro_use]
extern crate typed_builder;

use crate::server::handler::{HandlerConfig, RequestHandler};
use crate::server::HttpMockServer;

mod server;

#[derive(TypedBuilder, Debug)]
pub struct HttpMockConfig {
    pub port: u16,
}

pub fn start(http_mock_config: HttpMockConfig) {
    let handler_config = HandlerConfig::builder().build();
    let request_handler = RequestHandler::from_config(handler_config);

    let http_server_config = server::ServerConfig::builder()
        .port(http_mock_config.port)
        .request_handler(request_handler)
        .build();
    let server = HttpMockServer::from_config(http_server_config);

    server.start();
}
