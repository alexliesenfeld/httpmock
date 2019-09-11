use crate::server::router::{HttpMockRouter};
use crate::routes::mock::{mock_resource_handler, get_user_mock};

mod mock;

pub fn create_router() -> HttpMockRouter {
    let mut router = HttpMockRouter::new();
    router.add("/__/mock", mock_resource_handler);
    router.add("/**", get_user_mock);
    router
}
