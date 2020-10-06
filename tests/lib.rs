#[macro_use]
extern crate lazy_static;

mod internal;
mod examples;

use httpmock::standalone::start_standalone_server;
use std::sync::Mutex;
use std::thread::{spawn, JoinHandle};
use tokio::task::LocalSet;

/// ====================================================================================
/// The rest of this file is only required to simulate that a standalone mock server is
/// running somewhere else. The tests above will is.
/// ====================================================================================
pub fn simulate_standalone_server() {
    let _ = STANDALONE_SERVER.lock().unwrap_or_else(|e| e.into_inner());
}

lazy_static! {
    static ref STANDALONE_SERVER: Mutex<JoinHandle<Result<(), String>>> = Mutex::new(spawn(|| {
        let srv = start_standalone_server(5000, false);
        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        LocalSet::new().block_on(&mut runtime, srv)
    }));
}
