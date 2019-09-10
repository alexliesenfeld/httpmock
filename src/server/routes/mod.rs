extern crate regex;
extern crate typed_builder;
use log::{debug, info};
use regex::Regex;
use std::collections::HashMap;
use std::string::FromUtf8Error;

mod admin;
mod mock;
mod verify;

pub fn create_routes() -> Vec<Route> {
    let mut routes = Vec::new();
    routes.extend(mock::routes());
    routes.extend(verify::routes());
    routes.extend(admin::routes());
    routes
}

#[derive(TypedBuilder, Debug)]
pub struct HttpMockHandlerRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(TypedBuilder, Debug)]
pub struct HttpMockHandlerResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

pub type RouterFunction = fn(&HandlerConfig, HttpMockHandlerRequest) -> HttpMockHandlerResponse;

pub struct Route {
    pub path_regex: Regex,
    pub handler: RouterFunction,
}

impl Route {
    pub fn from_path(path: &str, handler: RouterFunction) -> Route {
        let path_regex_str = path.to_string() + "$";
        Route::from(path_regex_str.as_str(), handler)
    }

    pub fn from(path_regex_str: &str, handler: RouterFunction) -> Route {
        Route {
            path_regex: Regex::new(path_regex_str).expect("Cannot parse path regex"),
            handler,
        }
    }
}

#[derive(TypedBuilder)]
pub struct HandlerConfig {
    pub routes: Vec<Route>,
}
