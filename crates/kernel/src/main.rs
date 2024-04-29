#![feature(asm_const)]
#![feature(error_in_core)]
#![no_std]
#![no_main]

extern crate alloc;

mod allocator;
mod boot;
mod cpio;
mod devicetree;
mod driver;
mod exception;
mod shell;

use cpio::CpioArchive;
use devicetree::DeviceTree;
use panic_wait as _;
use shell::commands;
use small_std::println;

use crate::{boot::DEVICETREE_START_ADDR, devicetree::DeviceTreeEntryValue};

const INITRD_DEVICETREE_NODE: &str = "chosen";
const INITRD_DEVICETREE_PROP: &str = "linux,initrd-start";

unsafe fn kernel_init() -> ! {
    exception::init_exception_handling();

    if let Err(e) = driver::register_drivers() {
        panic!("Failed to initialize driver subsystem: {}", e);
    }

    device::driver::driver_manager().init_drivers();

    // Finnaly go from unsafe to safe 🎉
    main()
}

fn main() -> ! {
    println!(
        "{} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    let (_, privilege_level) = exception::current_privilege_level();
    println!("Current privilege level: {}", privilege_level);

    println!("Exception handling state:");
    exception::asynchronous::print_state();

    println!("Drivers loaded:");
    device::driver::driver_manager().enumerate();

    println!("DTB loaded at: {:#x}", unsafe { DEVICETREE_START_ADDR });

    let mut cpio_start_addr = 0;

    let devicetree = unsafe { DeviceTree::new(DEVICETREE_START_ADDR) };
    if let Err(e) = devicetree.traverse(|node, props| {
        if node != INITRD_DEVICETREE_NODE {
            return;
        }
        for prop in props {
            let prop = prop.unwrap();
            if prop.name != INITRD_DEVICETREE_PROP {
                continue;
            }
            match prop.value {
                DeviceTreeEntryValue::U32(v) => cpio_start_addr = v as usize,
                DeviceTreeEntryValue::U64(v) => cpio_start_addr = v as usize,
                DeviceTreeEntryValue::String(v) => println!("invalid initrd start address: {}", v),
                DeviceTreeEntryValue::Bytes(v) => println!("invalid initrd start address: {:?}", v),
            }
        }
    }) {
        println!("Failed to parse devicetree: {}", e);
    };

    if cpio_start_addr == 0 {
        println!("No initrd found. Halting...");
        panic!("no initrd found");
    }
    println!("CPIO loaded at: {:#x}", cpio_start_addr);

    println!("Echoing input now");

    let cpio = unsafe { CpioArchive::new(cpio_start_addr) };
    let mut shell = shell::Shell::new();
    let ls = commands::Ls::new(&cpio);
    let cat = commands::Cat::new(&cpio);
    let exec = commands::Exec::new(&cpio);
    shell.register(&commands::Hello);
    shell.register(&commands::Reboot);
    shell.register(&commands::Info);
    shell.register(&ls);
    shell.register(&cat);
    shell.register(&exec);
    shell.run_loop();
}
