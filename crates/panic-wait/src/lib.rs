//! A panic handler that infinitely waits.

#![no_std]

use core::panic::PanicInfo;
use small_std::println;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic_prevent_reenter();

    println!("{}", info);

    cpu::wait_forever();
}

/// Stop immediately if called a second time.
fn panic_prevent_reenter() {
    // Using atoms can save us from needing `unsafe` here.
    use core::sync::atomic::{AtomicBool, Ordering};

    static PANIC_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

    if !PANIC_IN_PROGRESS.load(Ordering::Relaxed) {
        PANIC_IN_PROGRESS.store(true, Ordering::Relaxed);
        return;
    }

    cpu::wait_forever();
}
