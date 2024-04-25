pub mod push_kernel;
pub mod qemu;

use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
    process::{Command, Stdio},
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
}

type Result<T> = std::result::Result<T, Error>;

pub struct TaskRunner {
    root: PathBuf,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BinTarget {
    Kernel,
    Uartload,
}

impl BinTarget {
    fn as_str(&self) -> &str {
        match self {
            BinTarget::Kernel => "kernel",
            BinTarget::Uartload => "uartload",
        }
    }

    fn image_name(&self) -> &str {
        match self {
            BinTarget::Kernel => "rpi3-kernel.img",
            BinTarget::Uartload => "kernel8.img",
        }
    }
}

macro_rules! envs {
    ($($key:expr => $value:expr),* $(,)?) => {{
        #[allow(unused_mut)]
        let mut map = ::std::collections::HashMap::new();
        $(map.insert($key.into(), $value.into());)*
        map
    }};
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

    pub fn run_build(&self, target: BinTarget) -> Result<()> {
        self.cargo(
            &format!(
                "rustc --package={} --target={} --release",
                target.as_str(),
                TARGET_TRIPLE,
            ),
            Some(envs! {
                "RUSTFLAGS" => format!(
                    "-C target-cpu=cortex-a53 -C link-arg=--library-path={0}/crates/{1} -C link-arg=--script={1}.ld -D warnings",
                    self.root.display(),
                    target.as_str(),
                )
            }),
        )?;

        let release_dir = self.release_dir();
        let elf = release_dir.join(target.as_str());
        let output_img = release_dir.join(target.image_name());

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

    pub fn run_qemu(&self, args: qemu::Args) -> Result<()> {
        self.run_build(args.target)?;

        let kernel_path = self.release_dir().join(args.target.image_name());
        qemu::run_qemu(kernel_path, args)?;

        Ok(())
    }

    pub fn run_push_kernel(&self, args: push_kernel::Args) -> Result<()> {
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
