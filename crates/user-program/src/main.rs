#![feature(format_args_nl)]
#![no_std]
#![no_main]

use core::{
    fmt::Write,
    ptr::{read_volatile, write_volatile},
};

use panic_wait as _;

const MMIO_BASE: usize = 0x3F00_0000;
const AUX_MU_IO: *mut u32 = (MMIO_BASE + 0x0021_5040) as _;
const AUX_MU_LSR: *const u32 = (MMIO_BASE + 0x0021_5054) as _;

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub extern "C" fn main() -> ! {
    println!("Hello from user program!");

    #[allow(clippy::empty_loop)]
    loop {}
}

struct MiniUart;

impl MiniUart {
    fn write_byte(&self, b: u8) {
        while (unsafe { read_volatile(AUX_MU_LSR) } & (1 << 5)) == 0 {}
        unsafe { write_volatile(AUX_MU_IO, b as u32) };
    }
}

impl core::fmt::Write for MiniUart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            self.write_byte(b);
        }
        Ok(())
    }
}

fn _print(args: core::fmt::Arguments) {
    unsafe {
        MINI_UART.write_fmt(args).unwrap();
    }
}

static mut MINI_UART: MiniUart = MiniUart;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
}
