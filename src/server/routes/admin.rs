use actix_web::{HttpResponse, Result};

/// This route is responsible for listing all currently stored mocks
pub fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().body("OK"))
}
