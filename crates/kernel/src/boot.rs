use core::arch::global_asm;

#[no_mangle]
#[link_section = ".text._start_arguments"]
pub static BOOT_CORE_ID: u64 = 0;

global_asm!(
    include_str!( "boot.s"),
    CONST_CURRENTEL_EL2 = const 0x8,
    CONST_CORE_ID_MASK = const 0b11,
);

#[no_mangle]
pub unsafe fn _start_rust(devicetree_start_addr: usize) -> ! {
    crate::kernel_init(devicetree_start_addr)
}
