use crate::server::data::ApplicationState;
use actix_web::{middleware, web, App, HttpServer};

pub(crate) mod data;
mod handlers;
mod routes;
mod util;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const MOCKS_PATH: &str = "/__mocks";
const MOCK_PATH: &str = "/__mocks/{id}";

/// Holds server configuration properties.
#[derive(TypedBuilder, Debug)]
pub struct HttpMockConfig {
    pub port: u16,
    pub workers: usize,
    pub expose: bool,
}

/// Starts a new instance of an HTTP mock server. You should never need to use this function
/// directly. Use it if you absolutely need to manage the low-level details of how the mock
/// server operates.
pub fn start_server(http_mock_config: HttpMockConfig) {
    let port = http_mock_config.port;
    let host = match http_mock_config.expose {
        true => "0.0.0.0",    // allow traffic from all sources
        false => "127.0.0.1", // allow traffic from localhost only
    };

    let server_state = web::Data::new(ApplicationState::new());
    HttpServer::new(move || {
        let server_state = server_state.clone();
        App::new()
            .register_data(server_state)
            .wrap(middleware::DefaultHeaders::new().header("X-Version", VERSION))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .route(MOCKS_PATH, web::post().to(routes::add))
            .route(MOCKS_PATH, web::delete().to(routes::delete_all))
            .route(MOCK_PATH, web::delete().to(routes::delete_one))
            .route(MOCK_PATH, web::get().to(routes::read_one))
            .default_service(web::route().to_async(routes::serve))
    })
    .bind(format!("{}:{}", host, port))
    .expect("Cannot bind to port")
    .workers(http_mock_config.workers)
    .run()
    .expect("Cannot start server");
}
