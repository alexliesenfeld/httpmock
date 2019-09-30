use httpmock::{start_server, HttpMockConfig};
use structopt::StructOpt;

/// Holds command line parameters provided by the user.
#[derive(StructOpt, Debug)]
pub struct CommandLineParameters {
    #[structopt(short, long, default_value = "5000")]
    pub port: u16,
    #[structopt(short, long, default_value = "3")]
    pub workers: usize,
    #[structopt(short, long)]
    pub expose: bool,
}

fn main() {
    env_logger::init();

    let params: CommandLineParameters = CommandLineParameters::from_args();

    if params.expose {
        log::info!("Starting public mock server on port {}", params.port);
    } else {
        log::info!("Starting private mock server on port {}", params.port);
    }

    let config = HttpMockConfig::builder()
        .port(params.port)
        .workers(params.workers)
        .expose(params.expose)
        .build();

    start_server(config);
}
