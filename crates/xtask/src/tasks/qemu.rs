use std::{ffi::OsStr, path::PathBuf, process::Command};

use super::{BinTarget, Error, Result};

#[derive(Debug, clap::Args)]
pub struct Args {
    pub target: BinTarget,

    /// Expose port for attaching debugger
    #[arg(long)]
    pub debug: bool,

    /// Use stdio, or PTY if not specified
    #[arg(long)]
    stdio: bool,

    #[arg(long, default_value = "initramfs.cpio")]
    cpio: PathBuf,

    #[arg(long, default_value = "bcm2710-rpi-3-b-plus.dtb")]
    dtb: PathBuf,
}

pub fn run_qemu(kernel_path: PathBuf, args: Args) -> Result<()> {
    let mut command = Command::new("qemu-system-aarch64");
    command
        .args(["-M", "raspi3b"])
        .args(["-serial", "null"])
        .args(["-serial", if args.stdio { "stdio" } else { "pty" }])
        .args(["-display", "none"])
        .args([
            "-initrd",
            args.cpio.to_str().expect("invalid UTF-8 sequenct in CPIO"),
        ])
        .args([
            "-dtb",
            args.dtb.to_str().expect("invalid UTF-8 sequenct in DTB"),
        ])
        .args([
            "-kernel",
            kernel_path
                .to_str()
                .expect("invalid UTF-8 sequenct in kernel path"),
        ]);

    if args.debug {
        command.args(["-S", "-s"]);
    }

    let cmd_args = command
        .get_args()
        .filter_map(OsStr::to_str)
        .collect::<Vec<_>>();
    tracing::info!(
        command = format!("qemu-system-aarch64 {}", cmd_args.join(" ")),
        "Running"
    );
    if !command.status()?.success() {
        return Err(Error::CommandFailed("qemu-system-aarch64"));
    }

    Ok(())
}
