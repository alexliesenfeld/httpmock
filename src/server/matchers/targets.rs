use crate::data::{HttpMockRequest, RequestRequirements};
use crate::server::matchers::sources::{ValueSource};
use crate::server::matchers::util::parse_cookies;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::BTreeMap;

pub(crate) trait ValueTarget<T> {
    fn parse_from_request(&self, req: &HttpMockRequest) -> Option<T>;
}

pub(crate) trait ValueRefTarget<T> {
    fn parse_from_request<'a>(&self, req: &'a HttpMockRequest) -> Option<&'a T>;
}

pub(crate) trait MultiValueValueTarget {
    fn parse_from_request(&self, req: &HttpMockRequest) -> BTreeMap<String, String>;
}

// *************************************************************************************
// StringBodyTarget
// *************************************************************************************
pub(crate) struct StringBodyTarget {}

impl StringBodyTarget {
    pub fn new() -> Self {
        Self {}
    }
}

impl ValueTarget<String> for StringBodyTarget {
    fn parse_from_request(&self, req: &HttpMockRequest) -> Option<String> {
        req.body.as_ref().map(|b| b.to_string()) // FIXME: Avoid copying here. Create a "ValueRefTarget".
    }
}

// *************************************************************************************
// JSONBodyTarget
// *************************************************************************************
pub(crate) struct JSONBodyTarget {}

impl JSONBodyTarget {
    pub fn new() -> Self {
        Self {}
    }
}

impl ValueTarget<Value> for JSONBodyTarget {
    fn parse_from_request(&self, req: &HttpMockRequest) -> Option<Value> {
        let body = req.body.as_ref();
        if body.is_none() {
            return None;
        }

        match serde_json::from_str(body.unwrap()) {
            Err(e) => {
                log::warn!("Cannot parse json value: {}", e);
                None
            }
            Ok(v) => Some(v),
        }
    }
}

// *************************************************************************************
// CookieTarget
// *************************************************************************************
pub(crate) struct CookieTarget {}

impl CookieTarget {
    pub fn new() -> Self {
        Self {}
    }
}
impl MultiValueValueTarget for CookieTarget {
    fn parse_from_request(&self, req: &HttpMockRequest) -> BTreeMap<String, String> {
        let req_cookies = match parse_cookies(req) {
            Ok(v) => v,
            Err(err) => {
                log::info!(
                "Cannot parse cookies. Cookie matching will not work for this request. Error: {}",
                err
            );
                return BTreeMap::new();
            }
        };

        req_cookies
            .into_iter()
            .map(|(k, v)| (k.to_lowercase(), v))
            .collect()
    }
}
