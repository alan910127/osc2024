use clap::{Parser, Subcommand};

use crate::prelude::*;
use crate::tasks::{push_kernel, qemu, BinTarget, TaskRunner};

/// Easily execute automated tasks.
///
/// `cargo xtask` is a pattern that enables you to integrate custom automation into your Rust project,
/// akin to tools such as `make`, `npm run`, or bash scripts.
///
/// For more information about `cargo xtask`, please visit the GitHub repository at https://github.com/matklad/cargo-xtask.
#[derive(Debug, Parser)]
#[command(disable_help_subcommand = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    pub fn parse_and_run() -> Result<()> {
        let cli = Cli::parse();

        let runner = TaskRunner::new()?;

        match cli.command {
            Commands::Check => runner.run_check()?,
            Commands::Build { target } => runner.run_build(target)?,
            Commands::Qemu(args) => runner.run_qemu(args)?,
            Commands::PushKernel(args) => runner.run_push_kernel(args)?,
        };

        Ok(())
    }
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run formatting and linting checks
    Check,
    /// Build the binary and perform post-processing steps (if any)
    Build {
        target: BinTarget,
    },
    /// Run the target in a QEMU emulation environment
    Qemu(qemu::Args),
    PushKernel(push_kernel::Args),
}
