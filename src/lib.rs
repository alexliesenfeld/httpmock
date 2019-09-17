#[macro_use]
extern crate typed_builder;
#[macro_use]
extern crate lazy_static;

use actix_web::{middleware, web, App, HttpServer};

pub mod api;
mod handlers;
mod routes;
mod util;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const MOCK_PATH: &str = "/__mocks";
const HEALTH_PATH: &str = "/__admin/health";

#[derive(TypedBuilder, Debug)]
pub struct HttpMockConfig {
    pub port: u16,
    pub workers: usize,
}

pub fn start_server(http_mock_config: HttpMockConfig) {
    let server_state = web::Data::new(handlers::HttpMockState::new());
    HttpServer::new(move || {
        let server_state = server_state.clone();
        App::new()
            .register_data(server_state)
            .wrap(middleware::DefaultHeaders::new().header("X-Version", VERSION))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .route(MOCK_PATH, web::post().to(routes::mocks::add))
            .route(MOCK_PATH, web::get().to(routes::mocks::list))
            .route(MOCK_PATH, web::delete().to(routes::mocks::clear))
            .route(HEALTH_PATH, web::get().to(routes::admin::health))
            .default_service(web::route().to_async(routes::mocks::serve))
    })
    .bind(format!("127.0.0.1:{}", http_mock_config.port))
    .expect("Cannot bind to port")
    .workers(http_mock_config.workers)
    .run()
    .expect("Cannot start server");
}
