use crate::data::{HttpMockRequest, RequestRequirements};
use crate::server::matchers::parse_cookies;
use crate::server::matchers::sources::ValueSource;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::BTreeMap;

pub(crate) trait ValueTarget<T> {
    fn parse_from_request(&self, req: &HttpMockRequest) -> Option<T>;
}

pub(crate) trait ValueRefTarget<T> {
    fn parse_from_request<'a>(&self, req: &'a HttpMockRequest) -> Option<&'a T>;
}

pub(crate) trait MultiValueTarget<T, U> {
    fn parse_from_request(&self, req: &HttpMockRequest) -> Option<Vec<(T, Option<U>)>>;
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

impl MultiValueTarget<String, String> for CookieTarget {
    fn parse_from_request(&self, req: &HttpMockRequest) -> Option<Vec<(String, Option<String>)>> {
        let req_cookies = match parse_cookies(req) {
            Ok(v) => v,
            Err(err) => {
                log::info!(
                "Cannot parse cookies. Cookie matching will not work for this request. Error: {}",
                err
            );
                return None;
            }
        };

        Some(req_cookies.into_iter().map(|(k, v)| (k, Some(v))).collect())
    }
}

// *************************************************************************************
// HeaderTarget
// *************************************************************************************
pub(crate) struct HeaderTarget {}

impl HeaderTarget {
    pub fn new() -> Self {
        Self {}
    }
}

impl MultiValueTarget<String, String> for HeaderTarget {
    fn parse_from_request(&self, req: &HttpMockRequest) -> Option<Vec<(String, Option<String>)>> {
        req.headers.as_ref().map(|headers| {
            headers
                .into_iter()
                .map(|(k, v)| (k.to_string(), Some(v.to_string())))
                .collect()
        })
    }
}

// *************************************************************************************
// HeaderTarget
// *************************************************************************************
pub(crate) struct QueryParameterTarget {}

impl QueryParameterTarget {
    pub fn new() -> Self {
        Self {}
    }
}

impl MultiValueTarget<String, String> for QueryParameterTarget {
    fn parse_from_request(&self, req: &HttpMockRequest) -> Option<Vec<(String, Option<String>)>> {
        req.query_params.as_ref().map(|headers| {
            headers
                .into_iter()
                .map(|(k, v)| (k.to_string(), Some(v.to_string())))
                .collect()
        })
    }
}

// *************************************************************************************
// PathTarget
// *************************************************************************************
pub(crate) struct PathTarget {}

impl PathTarget {
    pub fn new() -> Self {
        Self {}
    }
}

impl ValueTarget<String> for PathTarget {
    fn parse_from_request(&self, req: &HttpMockRequest) -> Option<String> {
        Some(req.path.to_string()) // FIXME: Avoid copying here. Create a "ValueRefTarget".
    }
}

// *************************************************************************************
// MethodTarget
// *************************************************************************************
pub(crate) struct MethodTarget {}

impl MethodTarget {
    pub fn new() -> Self {
        Self {}
    }
}

impl ValueTarget<String> for MethodTarget {
    fn parse_from_request(&self, req: &HttpMockRequest) -> Option<String> {
        Some(req.method.to_string()) // FIXME: Avoid copying here. Create a "ValueRefTarget".
    }
}
