use crate::server::data::ApplicationState;

use regex::Regex;
use std::io::Cursor;
use std::sync::Arc;
use std::thread;
use tiny_http::{Method, Request, Response};

pub(crate) mod data;
mod handlers;
mod routes;
mod util;

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

    let server_state = Arc::new(ApplicationState::new());

    let server = tiny_http::Server::http(format!("{}:{}", host, port)).unwrap();
    let server = Arc::new(server);
    let mut workers = Vec::new();

    for _ in 0..http_mock_config.workers {
        let server = server.clone();
        let server_state = server_state.clone();

        let guard = thread::spawn(move || loop {
            match server.recv() {
                Err(e) => {
                    log::error!("Error receiving HTTP request: {}", e);
                }
                Ok(mut req) => {
                    let response = route(&mut req, &server_state);
                    let response = map_response(response);
                    let result = req.respond(response);
                    if let Err(e) = result {
                        log::error!("Error sending HTTP response: {}", e);
                    }
                }
            }
        });

        workers.push(guard);
    }

    for worker in workers {
        worker.join().expect("Error joining thread");
    }
}

lazy_static! {
    static ref MOCK_PATH: Regex = Regex::new(r"/__mocks/([0-9]+)$").unwrap();
    static ref MOCKS_PATH: Regex = Regex::new(r"/__mocks$").unwrap();
}

fn route(req: &mut Request, state: &ApplicationState) -> Result<Response<Cursor<Vec<u8>>>, String> {
    log::trace!("Matching new incoming request with url '{}'", req.url());

    if MOCKS_PATH.is_match(req.url()) {
        match req.method() {
            Method::Post => return routes::add(state, req),
            Method::Delete => return routes::delete_all(state, req),
            _ => {}
        }
    }

    if MOCK_PATH.is_match(req.url()) {
        let id = get_path_param(&MOCK_PATH, 1, req.url());
        if let Err(e) = id {
            return Err(format!("Cannot parse id from path: {}", e));
        }
        let id = id.unwrap();

        match req.method() {
            Method::Get => return routes::read_one(state, req, id),
            Method::Delete => return routes::delete_one(state, req, id),
            _ => {}
        }
    }

    return routes::serve(state, req);
}

fn map_response(response: Result<Response<Cursor<Vec<u8>>>, String>) -> Response<Cursor<Vec<u8>>> {
    if let Err(e) = response {
        return Response::from_data(e).with_status_code(500);
    }
    response.unwrap()
}

fn get_path_param(regex: &Regex, idx: usize, path: &str) -> Result<usize, String> {
    let cap = regex.captures(path);
    if cap.is_none() {
        return Err(format!(
            "Error capturing parameter from request path: {}",
            path
        ));
    }
    let cap = cap.unwrap();

    let id = cap.get(idx);
    if id.is_none() {
        return Err(format!(
            "Error capturing resource id in request path: {}",
            path
        ));
    }
    let id = id.unwrap().as_str();

    let id = id.parse::<usize>();
    if let Err(e) = id {
        return Err(format!("Error parsing id as a number: {}", e));
    }
    let id = id.unwrap();

    Ok(id)
}
#[cfg(test)]
mod test {
    use crate::server::{MOCKS_PATH, MOCK_PATH};

    #[test]
    fn route_regex_test() {
        assert_eq!(MOCK_PATH.is_match("/__mocks/1"), true);
        assert_eq!(MOCK_PATH.is_match("/__mocks/1295473892374"), true);
        assert_eq!(MOCK_PATH.is_match("/__mocks/abc"), false);
        assert_eq!(MOCK_PATH.is_match("/__mocks"), false);
        assert_eq!(MOCK_PATH.is_match("/__mocks/345345/test"), false);
        assert_eq!(MOCK_PATH.is_match("test/__mocks/345345/test"), false);

        assert_eq!(MOCKS_PATH.is_match("/__mocks"), true);
        assert_eq!(MOCKS_PATH.is_match("/__mocks/5"), false);
        assert_eq!(MOCKS_PATH.is_match("test/__mocks/5"), false);
    }
}

/*
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const MOCKS_PATH: &str = "/__mocks";
const MOCK_PATH: &str = "/__mocks/{id}";

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

*/
