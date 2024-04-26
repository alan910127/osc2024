use aarch64_cpu::{
    asm,
    registers::{ELR_EL2, HCR_EL2, SPSR_EL2, SP_EL1},
};
use core::arch::global_asm;
use tock_registers::interfaces::Writeable;

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
pub unsafe fn _start_rust(
    devicetree_start_addr: u64,
    phys_boot_core_stack_end_exclusive_addr: u64,
) -> ! {
    DEVICETREE_START_ADDR = devicetree_start_addr as usize;

    // Since we are not going back to EL2, here we just use the same stack address directly.
    prepare_el2_to_el1(phys_boot_core_stack_end_exclusive_addr);

    // *return* to EL1. So we will be running kerenl_init() in EL1.
    asm::eret();
}

#[inline(always)]
fn prepare_el2_to_el1(stack_end_addr: u64) {
    // The execution state for EL1 is AArch64.
    // The execution state for EL0 is determined by the current value of PSTATE.nRW when executing at EL0.
    HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);

    SPSR_EL2.write(
        SPSR_EL2::D::Masked // watchpoint, breakpoint, and software step exceptions => for debuggers?
            + SPSR_EL2::A::Masked // SError interrupt
            + SPSR_EL2::F::Masked // FIQ interrupt
            + SPSR_EL2::I::Masked // IRQ
            + SPSR_EL2::M::EL1h, // where the exception is taken from
    );

    // Set exception return address to kernel_init()
    ELR_EL2.set(crate::kernel_init as *const () as u64);

    // Set up stack pointer for EL1.
    SP_EL1.set(stack_end_addr);
}
