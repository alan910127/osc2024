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
mod shell;

use cpio::CpioArchive;
use devicetree::DeviceTree;
use panic_wait as _;
use shell::commands;
use small_std::println;

use crate::devicetree::DeviceTreeEntryValue;

const INITRD_DEVICETREE_NODE: &str = "chosen";
const INITRD_DEVICETREE_PROP: &str = "linux,initrd-start";

unsafe fn kernel_init(devicetree_start_addr: usize) -> ! {
    if let Err(e) = driver::register_drivers() {
        panic!("Failed to initialize driver subsystem: {}", e);
    }

    device::driver::driver_manager().init_drivers();

    // Finnaly go from unsafe to safe 🎉
    main(devicetree_start_addr)
}

fn main(devicetree_start_addr: usize) -> ! {
    println!(
        "[0] {} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    println!("[1] Drivers loaded:");
    device::driver::driver_manager().enumerate();

    println!("[2] DTB loaded at: {:#x}", devicetree_start_addr);

    let mut cpio_start_addr = 0;

    let devicetree = unsafe { DeviceTree::new(devicetree_start_addr) };
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
    println!("[3] CPIO loaded at: {:#x}", cpio_start_addr);

    println!("[4] Echoing input now");

    let cpio = unsafe { CpioArchive::new(cpio_start_addr) };
    let mut shell = shell::Shell::new();
    let ls = commands::LsCommand::new(&cpio);
    let cat = commands::CatCommand::new(&cpio);
    shell.register(&commands::HelloCommand);
    shell.register(&commands::RebootCommand);
    shell.register(&commands::InfoCommand);
    shell.register(&ls);
    shell.register(&cat);
    shell.run_loop();
}
