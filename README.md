# OSC2024

Mini operating system written in Rust.

This is the course project/assignments for "Operating System Capstone," 2024 Spring, at NYCU.

## Table of Contents

- [Student Information](#student-information)
- [Environment Setup](#environment-setup)
- [Dev Notes](#dev-notes)
- [Lab Descriptions](#lab-descriptions)
- [Reference](#reference)

## Student Information

| GitHub Account | Student ID | Name       |
| -------------- | ---------- | ---------- |
| alan910127     | 109652039  | Li-Lun Lin |

## Environment Setup

> [!NOTE]
> Rust uses [LLVM](https://llvm.org/) as its compiler backend, making cross-compilation straightforward with the same set of commands after installing the toolchains.

1. **Install Rust:**

   Make sure you have [Rust](https://rust-lang.org/) installed on your system. For this project, we require the following Rust toolchains:

   - Channel: `nightly-2024-04-04`
   - Target: `aarch64-unknown-none-softfloat`
   - Components: `llvm-tools`

   `rustup` should recognize the settings in [`rust-toolchain.toml`](./rust-toolchain.toml) and automatically install the required toolchains for you.

2. **Install tools for building the kernel image:**

   In addition to Rust, you'll need tools like `rust-objcopy` for building the carno image.
   You can install these tools using Cargo.

   ```sh
   # Alternatively, you can replace the following command with `cargo binstall`.
   cargo install cargo-binutils
   ```

## How to Run

This repository follows the [`cargo-xtask`](https://github.com/matklad/cargo-xtask) pattern, enabling you to execute necessary tasks with a single command: `cargo xtask` in your terminal.

There are four subcommands available under the `cargo xtask` command:

- `check`: Performs formatting and linting checks.
- `build`: Compiles the binary and executes post-processing steps (if any).
- `qemu`: Launches the target in a QEMU emulation environment.
- `push-kernel`: Transfers the kernel through a UART-connected serial device for loading by uartload.

For instance, to experience the full booting process from the bootloader, follow these steps:

1. Create [your CPIO archive file](https://nycu-caslab.github.io/OSC2024/labs/lab2.html#new-ascii-format-cpio-archive) and download [the device tree binary for Raspberry Pi 3b+](https://github.com/raspberrypi/firmware/raw/master/boot/bcm2710-rpi-3-b-plus.dtb).
2. Execute `cargo xtask qemu uartload` in your terminal.
3. Open another terminal session and run `cargo xtask build kernel`.
4. Issue `cargo xtask push-kernel --image ./target/aarch64-unknown-none-softfloat/release/rpi3-kernel.img --device /dev/pts/N`, replacing `/dev/pts/N` with the appropriate device path as shown in your QEMU output.
5. The operating system should be running after pushing the kernel.

## Dev Notes

This repository contains the following entrypoints that might be great to start from:

- [`uartload`](crates/uartload)
- [`kernel`](crates/kernel)

## Lab Descriptions

### Lab 0: Environment Setup ([website](https://nycu-caslab.github.io/OSC2024/labs/lab0.html))

Prepare the environment for developing the mini operating system.

> See [Environment Setup](#environment-setup)

### Lab 1: Hello World ([website](https://nycu-caslab.github.io/OSC2024/labs/lab1.html))

Getting into the world of embedded programming, and try to play with pheripherals.

Tasks:

- [x] **Basic Initialization**: Initialize the memory/registers to be ready to jump into the program.
- [x] **Mini UART**: Setup mini UART to bridge the host and the Raspberry PI.
- [x] **Simple Shell**: Implement a simple shell that display text and read input through mini UART.
- [x] **Mailbox**: Set up the Mailbox service and get the hardware information from it.
- [x] **Reboot**: Add a `reboot` command to the shell to reset the Raspberry PI.

### Lab 2: Booting ([website](https://nycu-caslab.github.io/OSC2024/labs/lab2.html))

Booting the mini operating system, take care of system initialization and preparation

Tasks:

- [x] **UART Bootloader**: Implement a bootloader that loads kernel through mini UART for fast development.
- [x] **Initial Ramdisk**: Parse "New ASCII Format Cpio" archive file and implement `ls` and `cat`.
- [x] **Simple Allocator**: Implement a simple allocator that can be used in the early booting stage.
- [x] **Bootloader Self Relocation**: Add self-relocation feature to the bootloader so it does not need to specify the kernel starting address.
- [x] **DeivceTree**: Integrate DeviceTree support for hardware configuration.

### Lab 3: Exception and Interrupt ([website](https://nycu-caslab.github.io/OSC2024/labs/lab3.html))

Get familiar with exception levels, exceptions, and interrupts.

Tasks:

- [ ] **Exception**: Switch between different exception levels and implement exception handlers.
- [ ] **Interrupt**: Enable and handle the core timer's interrupt.
- [ ] **Rpi3's Peripheral Interrupt**: Implement asynchronous UART read/write by interrupt handlers.
- [ ] **Timer Multiplexing**: Implement the non-blocking shell command `setTimeout` which prints message after specified delay.
- [ ] **Concurrent I/O Devices Handling**: Implement a preemptive task queue for interrupts.

## Reference

- [rust-embedded/rust-raspberrypi-OS-tutorials](https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials)
