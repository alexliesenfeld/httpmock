use crate::handlers::{HttpMockRequest, HttpMockResponse, HttpMockState, SetMockRequest};
use std::cmp::Ordering;
use std::collections::BTreeMap;

pub fn add_new_mock(state: &HttpMockState, req: SetMockRequest) -> Result<(), &'static str> {
    {
        let mut mocks = state.mocks.write().unwrap();
        mocks.push(req);
    }

    return Result::Ok(());
}

pub fn clear_mocks(state: &HttpMockState, _req: SetMockRequest) -> Result<(), &'static str> {
    {
        let mut mocks = state.mocks.write().unwrap();
        mocks.clear();
    }

    return Result::Ok(());
}

pub fn find_mock(
    state: &HttpMockState,
    req: HttpMockRequest,
) -> Result<Option<HttpMockResponse>, &'static str> {
    {
        let mocks = state.mocks.read().unwrap();
        let result = mocks.iter().find(|&s| request_matches(&req, s));

        if let Some(found) = result {
            return Ok(Some(found.response.clone()));
        }
    }

    return Result::Ok(None);
}

fn request_matches(req: &HttpMockRequest, mock: &SetMockRequest) -> bool {
    let mock = &mock.request;

    if !opt_string_equal(&mock.body, &req.body) {
        println!(
            "body not equal - mock: {:?}, req: {:?}",
            &mock.body, &req.body
        );
        return false;
    }

    if !opt_string_equal(&mock.path, &req.path) {
        println!(
            "path not equal - mock: {:?}, req: {:?}",
            &mock.path, &req.path
        );
        return false;
    }

    if !opt_string_equal(&mock.method, &req.method) {
        println!(
            "method not equal - mock: {:?}, req: {:?}",
            &mock.method, &req.method
        );
        return false;
    }

    /*
    if !opt_first_map_contains_second(&req.headers, &mock.headers) {
        return false;
    }*/

    true
}

fn opt_string_equal(s1: &Option<String>, s2: &Option<String>) -> bool {
    let mut v1 = "";
    if let Some(vv1) = s1 {
        v1 = vv1;
    }

    let mut v2 = "";
    if let Some(vv2) = s2 {
        v2 = vv2;
    }

    return v1 == v2;
}

fn opt_first_map_contains_second(
    s1: &Option<BTreeMap<String, String>>,
    s2: &Option<BTreeMap<String, String>>,
) -> bool {
    return match (s1, s2) {
        (Some(_m1), Some(m2)) => {
            return m2
                .iter()
                .all(|(k, v)| m2.contains_key(k) && m2.get(k).unwrap().cmp(v) == Ordering::Equal)
        }
        (None, None) => true,
        _ => false,
    };
}
