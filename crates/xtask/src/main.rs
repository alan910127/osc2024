mod cli;
mod prelude;
mod tasks;

use crate::cli::Cli;

fn main() {
    install_tracing();

    if let Err(e) = Cli::parse_and_run() {
        tracing::error!("task failed: {}", e);
    };
}

fn install_tracing() {
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer();
    let filter_layer = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();
}
