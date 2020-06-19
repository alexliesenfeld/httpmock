use httpmock::{start_server, HttpMockConfig};
use std::sync::Arc;
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
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("httpmock=info"));

    let params: CommandLineParameters = CommandLineParameters::from_args();

    log::info!(
        "Starting {} server V{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    let config = HttpMockConfig::new(params.port, params.workers, params.expose);

    let state = Arc::new(MockServerState::new());
    start_server(config, &state, None, None).await;
}
