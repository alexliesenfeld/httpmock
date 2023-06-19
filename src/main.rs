use clap::Parser;
use httpmock::standalone::start_standalone_server;
use std::env;
use std::path::PathBuf;

/// Holds command line parameters provided by the user.
#[derive(Parser, Debug)]
#[clap(
    version = "0.6",
    author = "Alexander Liesenfeld <alexander.liesenfeld@outlook.com>"
)]
struct ExecutionParameters {
    #[clap(short, long, env = "HTTPMOCK_PORT", default_value = "5000")]
    pub port: u16,
    #[clap(short, long, env = "HTTPMOCK_EXPOSE")]
    pub expose: bool,
    #[clap(short, long, env = "HTTPMOCK_MOCK_FILES_DIR")]
    pub mock_files_dir: Option<PathBuf>,
    #[clap(short, long, env = "HTTPMOCK_DISABLE_ACCESS_LOG")]
    pub disable_access_log: bool,
    #[clap(
        short,
        long,
        env = "HTTPMOCK_REQUEST_HISTORY_LIMIT",
        default_value = "100"
    )]
    pub request_history_limit: usize,
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("httpmock=info"));

    let params: ExecutionParameters = ExecutionParameters::parse();

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

    log::info!("{:?}", params);

    start_standalone_server(
        params.port,
        params.expose,
        params.mock_files_dir,
        !params.disable_access_log,
        params.request_history_limit,
        shutdown_signal(),
    )
    .await
    .expect("an error occurred during mock server execution");
}

#[cfg(not(target_os = "windows"))]
async fn shutdown_signal() {
    let mut hangup_stream = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup())
        .expect("Cannot install SIGINT signal handler");
    let mut sigint_stream =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
            .expect("Cannot install SIGINT signal handler");
    let mut sigterm_stream =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Cannot install SIGINT signal handler");

    tokio::select! {
        _val = hangup_stream.recv() => log::trace!("Received SIGINT"),
        _val = sigint_stream.recv() => log::trace!("Received SIGINT"),
        _val = sigterm_stream.recv() => log::trace!("Received SIGTERM"),
    }
}

#[cfg(target_os = "windows")]
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Cannot install CTRL+C signal handler");
}
