#[macro_use]
extern crate typed_builder;

use actix_web::{middleware, App, HttpServer, web};
use log::info;
mod routes;

#[derive(TypedBuilder, Debug)]
pub struct HttpMockConfig {
    pub port: u16,
    pub workers: usize,
}

pub fn start(http_mock_config: HttpMockConfig) {
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2"))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(routes::index)
            .default_service( web::route().to(routes::catch_all))
    })
    .bind(format!("127.0.0.1:{}", http_mock_config.port))
    .unwrap()
    .workers(http_mock_config.workers)
    .run();
}
