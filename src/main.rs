extern crate simple_logger;

use structopt::StructOpt;

/// Holds command line parameters provided by the user.
#[derive(StructOpt, Debug)]
pub struct CommandLineParameters {
    #[structopt(short, long, default_value = "5000")]
    pub port: u16,
    #[structopt(short, long, default_value = "3")]
    pub workers: usize,
    #[structopt(short, long = "log-level", default_value = "Info")]
    pub log_level: log::Level,
}

fn main() {
    let params: CommandLineParameters = CommandLineParameters::from_args();

    simple_logger::init_with_level(params.log_level)
        .expect("There was an error configuring the logging backend");

    let config = httpmock::HttpMockConfig::builder()
        .port(params.port)
        .workers(params.workers)
        .build();

    httpmock::start_server(config);
}
