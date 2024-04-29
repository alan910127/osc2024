pub mod build_img;
mod macros;
pub mod push_kernel;
pub mod qemu;

use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
};

use clap::ValueEnum;

const CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
const TARGET_TRIPLE: &str = "aarch64-unknown-none-softfloat";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("push kernel failed: {0}")]
    PushKernelFailed(#[from] push_kernel::Error),

    #[error("could not determine repository root")]
    CouldNotDetermineRepositoryRoot,
    #[error("failed to run command '{0}'")]
    CommandFailed(&'static str),

    #[error("cannot run user program in QEMU")]
    CannotRunUserProgramInQemu,
}

type Result<T> = std::result::Result<T, Error>;

pub struct TaskRunner {
    root: PathBuf,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum BinTarget {
    Kernel,
    Uartload,
    UserProgram,
}

impl BinTarget {
    fn as_str(&self) -> &str {
        match self {
            BinTarget::Kernel => "kernel",
            BinTarget::Uartload => "uartload",
            BinTarget::UserProgram => "user-program",
        }
    }

    fn image_name(&self) -> &str {
        match self {
            BinTarget::Kernel => "rpi3-kernel.img",
            BinTarget::Uartload => "kernel8.img",
            BinTarget::UserProgram => "user-program.img",
        }
    }
}

impl TaskRunner {
    pub fn new() -> Result<Self> {
        let root = Path::new(CARGO_MANIFEST_DIR)
            .ancestors()
            .nth(2)
            .ok_or(Error::CouldNotDetermineRepositoryRoot)?
            .to_path_buf();

        Ok(Self { root })
    }

    pub fn run_check(&self) -> Result<()> {
        self.cargo("fmt --all --check", None)?;
        self.cargo("check --all-features", None)?;
        self.cargo("clippy --all-features --no-deps -- -D warnings", None)?;
        Ok(())
    }

    pub fn run_build(&self, args: build_img::Args) -> Result<()> {
        build_img::run_build(args, self.root.clone(), |args, envs| self.cargo(args, envs))?;

        Ok(())
    }

    pub fn run_qemu(&self, args: qemu::Args) -> Result<()> {
        if args.target == BinTarget::UserProgram {
            return Err(Error::CannotRunUserProgramInQemu);
        }

        self.run_build(build_img::Args::new(args.target, args.debug))?;

        let kernel_path = self.release_dir().join(args.target.image_name());
        qemu::run_qemu(kernel_path, args)?;

        Ok(())
    }

    pub fn run_push_kernel(&self, args: push_kernel::Args) -> Result<()> {
        self.run_build(build_img::Args::new(BinTarget::Kernel, false))?;

        push_kernel::run_push_kernel(args)?;

        Ok(())
    }

    fn cargo(&self, args: &str, envs: Option<HashMap<String, String>>) -> Result<()> {
        let mut command = Command::new("cargo");
        if let Some(envs) = envs {
            command.envs(envs);
        }
        command
            .current_dir(&self.root)
            .args(shlex::split(args).expect("invalid cargo command"));

        let args = command
            .get_args()
            .filter_map(OsStr::to_str)
            .collect::<Vec<_>>();

        tracing::info!(command = format!("cargo {}", args.join(" ")), "Running");
        if command.status()?.success() {
            Ok(())
        } else {
            Err(Error::CommandFailed("cargo"))
        }
    }

    fn release_dir(&self) -> PathBuf {
        self.root.join(format!("target/{}/release", TARGET_TRIPLE))
    }
}
