mod cli;
mod prelude;
mod tasks;

use crate::cli::Cli;
use crate::prelude::*;

fn main() -> Result<()> {
    install_tracing();

    Cli::parse_and_run()
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
