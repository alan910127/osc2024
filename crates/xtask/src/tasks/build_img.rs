use std::{collections::HashMap, ffi::OsStr, path::PathBuf, process::Command};

use super::{BinTarget, Error, Result, TARGET_TRIPLE};
use crate::envs;

#[derive(Debug, clap::Args)]
pub struct Args {
    pub target: BinTarget,

    #[arg(long)]
    with_symbol: bool,
}

impl Args {
    pub fn new(target: BinTarget, with_symbol: bool) -> Self {
        Self {
            target,
            with_symbol,
        }
    }
}

pub fn run_build<Cargo>(args: Args, project_root: PathBuf, cargo: Cargo) -> Result<()>
where
    Cargo: Fn(&str, Option<HashMap<String, String>>) -> Result<()>,
{
    let rust_flags = format!(
        "-C target-cpu=cortex-a53 -C link-arg=--library-path={0}/crates/{1} -C link-arg=--script={1}.ld -D warnings",
        project_root.display(),
        args.target.as_str(),
    );

    let profile = if args.with_symbol {
        "release-with-symbols"
    } else {
        "release"
    };

    cargo(
        &format!(
            "rustc --package={} --target={} --profile={}",
            args.target.as_str(),
            TARGET_TRIPLE,
            profile,
        ),
        Some(envs! {
            "RUSTFLAGS" => rust_flags,
        }),
    )?;

    let release_dir = project_root.join(format!("target/{}/{}", TARGET_TRIPLE, profile));
    let elf = release_dir.join(args.target.as_str());
    let output_img = release_dir.join(args.target.image_name());

    let mut command = Command::new("rust-objcopy");
    command
        .arg("--strip-all")
        .args(["-O", "binary"])
        .arg(&elf)
        .arg(&output_img);

    let args = command
        .get_args()
        .filter_map(OsStr::to_str)
        .collect::<Vec<_>>();
    tracing::info!(
        command = format!("rust-objcopy {}", args.join(" ")),
        "Running"
    );
    if !command.status()?.success() {
        return Err(Error::CommandFailed("rust-objcopy"));
    }

    let image_size = output_img.metadata()?.len();
    tracing::info!(
        image = %output_img.display(),
        size = image_size,
        "Image built"
    );

    Ok(())
}
