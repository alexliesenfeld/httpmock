use super::{
    HandlerConfig, HttpMockHandlerRequest, HttpMockHandlerResponse, Route, RouterFunction,
};
use std::collections::HashMap;

pub fn routes() -> Vec<Route> {
    let mut routes = Vec::new();
    routes.push(Route::from_path("/admin", handle_admin));
    routes
}

fn handle_admin(
    handler_config: &HandlerConfig,
    req: HttpMockHandlerRequest,
) -> HttpMockHandlerResponse {
    HttpMockHandlerResponse::builder()
        .status_code(200 as u16)
        .headers(HashMap::new())
        .body("Hello!")
        .build()
}
