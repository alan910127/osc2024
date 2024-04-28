use core::arch::global_asm;

use crate::exception;

#[no_mangle]
#[link_section = ".text._start_arguments"]
pub static BOOT_CORE_ID: u64 = 0;

pub static mut DEVICETREE_START_ADDR: usize = 0;

global_asm!(
    include_str!( "boot.s"),
    CONST_CURRENTEL_EL2 = const 0x8,
    CONST_CORE_ID_MASK = const 0b11,
);

#[no_mangle]
pub unsafe extern "C" fn _start_rust(
    devicetree_start_addr: u64,
    phys_boot_core_stack_end_exclusive_addr: u64,
) -> ! {
    DEVICETREE_START_ADDR = devicetree_start_addr as usize;

    exception::transition_from_el2_to_el1(
        // Since we are not going back to EL2, here we just use the same stack address directly.
        phys_boot_core_stack_end_exclusive_addr,
        // Set exception return address to kernel_init()
        crate::kernel_init as *const () as u64,
    );
}
