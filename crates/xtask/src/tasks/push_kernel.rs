use std::{
    fs::File,
    io::{stdout, Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use serial2::SerialPort;

/// Push the kernel through a UART-connected serial device for loading by uartload.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Path to the kernel image to push
    #[clap(short, long)]
    image: PathBuf,

    /// Path to the UART serial device
    #[clap(short, long)]
    device: PathBuf,

    /// Baud rate to use for the serial device
    #[clap(short, long, default_value_t = 115200)]
    baud_rate: u32,

    /// Attach to the serial device after pushing the kernel
    #[clap(short, long, default_value = "false")]
    attach: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("kernel image is too large to be pushed: {0}")]
    KernelImageTooLarge(u64),

    #[error("did not received 'OK' from device")]
    NoOkReceived,
}

type Result<T> = std::result::Result<T, Error>;

pub fn run_push_kernel(args: Args) -> Result<()> {
    let mut serial = open_serial(&args.device, args.baud_rate)?;
    tracing::info!(serial = %args.device.display(), "Serial connected");

    let image = File::open(&args.image)?;
    let image_size = image.metadata()?.len();
    let Ok(image_size) = image_size.try_into() else {
        // Downcast from u64 to u32, the only possible error is if the image is too large
        return Err(Error::KernelImageTooLarge(image_size));
    };

    // Read out all remaining data from serial to prevent erroneous things from happening
    read_all(&mut serial)?;

    tracing::info!(
        kernel = %args.image.display(),
        size = image_size,
        "Pushing kernel",
    );

    send_size(image_size, &mut serial)?;
    push_kernel(image_size, image, &mut serial)?;

    if !args.attach {
        tracing::info!("Kernel pushed successfully, exiting...");
        return Ok(());
    }

    tracing::info!("Kernel pushed successfully, attaching to terminal...");
    forward_terminal(serial);

    Ok(())
}

fn wait_for_serial(serial: &Path) {
    if serial.exists() {
        return;
    }

    tracing::warn!(serial = %serial.display(), "Serial does not exist, waiting for serial device to be connected");
    loop {
        std::thread::sleep(Duration::from_secs(1));
        if serial.exists() {
            break;
        }
    }
}

fn open_serial(serial: &Path, baud_rate: u32) -> Result<SerialPort> {
    wait_for_serial(serial);

    let mut port = SerialPort::open(serial, baud_rate)?;
    port.set_read_timeout(Duration::from_secs(1))?;
    port.set_write_timeout(Duration::from_secs(1))?;

    Ok(port)
}

fn read_all(serial: &mut SerialPort) -> Result<()> {
    let mut buffer = [0u8; 1024];
    let mut timeout_retries = 3;

    loop {
        match serial.read(&mut buffer) {
            Ok(0) => {}
            Ok(n) => {
                stdout().write_all(&buffer[..n])?;
                stdout().flush()?;
            }
            Err(e) => {
                match e.kind() {
                    std::io::ErrorKind::TimedOut => {
                        if timeout_retries == 0 {
                            // Maybe we missed the start of the message
                            // So we'll just start sending the kernel
                            return Ok(());
                        }
                        timeout_retries -= 1;
                        continue;
                    }
                    _ => {
                        tracing::error!(error = %e, "Error reading from serial");
                        return Err(e.into());
                    }
                };
            }
        };
    }
}

fn send_size(image_size: u32, serial: &mut SerialPort) -> Result<()> {
    tracing::info!(size = image_size, "Pushing kernel size to device");
    serial.write_all(&image_size.to_le_bytes())?;

    let mut buffer = [0u8; 1024];
    let mut read = 0;
    while read < 2 {
        read += serial.read(&mut buffer[read..])?;
    }

    if &buffer[..2] != b"OK" {
        tracing::error!("Kernel push failed, did not receive 'OK' from device");
        return Err(Error::NoOkReceived);
    };

    stdout().write_all(&buffer[2..])?;
    stdout().flush()?;

    Ok(())
}

fn push_kernel(image_size: u32, mut image: impl Read, serial: &mut SerialPort) -> Result<()> {
    tracing::info!("Pushing kernel to device");
    let pb = ProgressBar::new(image_size as u64);

    let style = ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn std::fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-");
    pb.set_style(style);

    loop {
        let mut buffer = [0u8; 1024];
        let read = image.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        serial.write_all(&buffer[..read])?;
        pb.inc(read as u64);
    }

    pb.finish_with_message("Kernel pushed successfully");

    Ok(())
}

fn forward_terminal(serial: SerialPort) {
    let serial = Arc::new(serial);

    let s = serial.clone();
    let target_to_host = std::thread::spawn(move || {
        let serial = s;
        let mut ch = [0u8];
        loop {
            match serial.read(&mut ch) {
                Ok(1) => {
                    stdout().write_all(&ch).unwrap();
                    stdout().flush().unwrap();
                }
                Ok(_) => continue,
                Err(e) => {
                    match e.kind() {
                        std::io::ErrorKind::TimedOut => continue,
                        _ => {
                            tracing::error!(error = %e, "Error reading from serial");
                            break;
                        }
                    };
                }
            }
        }
        tracing::warn!("Connection closed");
    });

    // host to target
    loop {
        let mut buffer = [0u8];
        match std::io::stdin().read(&mut buffer) {
            Ok(1) => {
                if buffer[0] == 3 {
                    // Ctrl-C
                    break;
                }
                serial.write_all(&buffer).unwrap();
            }
            _ => break,
        }
    }

    target_to_host.join().unwrap();
}
