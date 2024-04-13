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

## Dev Notes

This repository contains three entrypoints that might be great to start from:

- [`uartload`](crates/uartload)
- [`uartpush`](crates/uartpush)
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

## Reference

- [rust-embedded/rust-raspberrypi-OS-tutorials](https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials)
