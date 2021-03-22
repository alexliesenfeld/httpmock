use structopt::StructOpt;

use httpmock::standalone::start_standalone_server;
use std::path::PathBuf;

/// Holds command line parameters provided by the user.
#[derive(StructOpt, Debug)]
pub struct CommandLineParameters {
    #[structopt(short, long, default_value = "5000")]
    pub port: u16,
    #[structopt(short, long)]
    pub expose: bool,
    #[structopt(short, long)]
    pub static_mock_dir: Option<PathBuf>,
    #[structopt(short, long)]
    pub disable_access_log: bool,
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("httpmock=info"));

    let params: CommandLineParameters = CommandLineParameters::from_args();

    log::info!("██╗  ██╗████████╗████████╗██████╗ ███╗   ███╗ ██████╗  ██████╗██╗  ██╗");
    log::info!("██║  ██║╚══██╔══╝╚══██╔══╝██╔══██╗████╗ ████║██╔═══██╗██╔════╝██║ ██╔╝");
    log::info!("███████║   ██║      ██║   ██████╔╝██╔████╔██║██║   ██║██║     █████╔╝");
    log::info!("██╔══██║   ██║      ██║   ██╔═══╝ ██║╚██╔╝██║██║   ██║██║     ██╔═██╗");
    log::info!("██║  ██║   ██║      ██║   ██║     ██║ ╚═╝ ██║╚██████╔╝╚██████╗██║  ██╗");
    log::info!("╚═╝  ╚═╝   ╚═╝      ╚═╝   ╚═╝     ╚═╝     ╚═╝ ╚═════╝  ╚═════╝╚═╝  ╚═╝");

    log::info!(
        "Starting {} server V{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    start_standalone_server(
        params.port,
        params.expose,
        params.static_mock_dir,
        !params.disable_access_log,
    )
    .await
    .expect("an error occurred during mock server execution");
}
