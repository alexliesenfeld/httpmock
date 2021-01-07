#[macro_use]
extern crate lazy_static;

use std::sync::Mutex;
use std::thread::{spawn, JoinHandle};

use tokio::task::LocalSet;

use httpmock::standalone::start_standalone_server;

mod examples;
mod internal;

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
        let mut runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        LocalSet::new().block_on(&mut runtime, srv)
    }));
}
