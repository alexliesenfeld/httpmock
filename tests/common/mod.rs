use httpmock::{start_server, HttpMockConfig};
use log::Level;
use std::thread;
use std::thread::JoinHandle;

lazy_static! {
    pub static ref SERVER: Result<JoinHandle<()>, String> = {
        let t = thread::spawn(move || {
            let config = HttpMockConfig::builder()
                .port(5000 as u16)
                .workers(3 as usize)
                .build();

            start_server(config);
        });
        Ok(t)
    };
    pub static ref LOGGER: Result<(), log::SetLoggerError> =
        { simple_logger::init_with_level(Level::Info) };
}

pub fn prepare_test_environment() {
    // This will init the server and the logger only on the first call and
    // then just use them on any subsequent call of this function.
    // See lazy_static crate.
    assert_eq!(true, SERVER.is_ok() && LOGGER.is_ok());
}
