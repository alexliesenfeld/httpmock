extern crate mocha;
extern crate env_logger;

use mocha::{start_server, HttpMockConfig};
use structopt::StructOpt;

/// Holds command line parameters provided by the user.
#[derive(StructOpt, Debug)]
pub struct CommandLineParameters {
    #[structopt(short, long, default_value = "5000")]
    pub port: u16,
    #[structopt(short, long, default_value = "3")]
    pub workers: usize,
}

fn main() {
    let params: CommandLineParameters = CommandLineParameters::from_args();
    env_logger::init();

    let config = HttpMockConfig::builder()
        .port(params.port)
        .workers(params.workers)
        .build();

    start_server(config);
}
