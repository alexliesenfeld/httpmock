use crate::server::router::{HttpMockRouter, Route};
use crate::server::router::Method::{POST, GET};
use crate::routes::mock::add_mock;

mod mock;

pub fn create_router() -> HttpMockRouter {
    let mut router = HttpMockRouter::new();
    add_route(&mut router, Route::from(POST, "/__/mock/:id", add_mock));
    add_route(&mut router, Route::from(GET, "/__/block/:id", add_mock));
    router
}

fn add_route(router: &mut HttpMockRouter, route: Route) {
    router.add(&route.path.to_string(), route);
}

