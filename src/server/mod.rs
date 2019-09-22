use crate::server::data::ApplicationState;
use actix_web::{middleware, web, App, HttpServer};

pub(crate) mod data;
mod handlers;
mod routes;
mod util;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const MOCKS_PATH: &str = "/__mocks";
const MOCK_PATH: &str = "/__mocks/{id}";

#[derive(TypedBuilder, Debug)]
pub struct HttpMockConfig {
    pub port: u16,
    pub workers: usize,
}

pub fn start_server(http_mock_config: HttpMockConfig) {
    let server_state = web::Data::new(ApplicationState::new());
    HttpServer::new(move || {
        let server_state = server_state.clone();
        App::new()
            .register_data(server_state)
            .wrap(middleware::DefaultHeaders::new().header("X-Version", VERSION))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .route(MOCKS_PATH, web::post().to(routes::add))
            .route(MOCK_PATH, web::delete().to(routes::delete_one))
            .route(MOCK_PATH, web::get().to(routes::read_one))
            .default_service(web::route().to_async(routes::serve))
    })
    .bind(format!("127.0.0.1:{}", http_mock_config.port))
    .expect("Cannot bind to port")
    .workers(http_mock_config.workers)
    .run()
    .expect("Cannot start server");
}
