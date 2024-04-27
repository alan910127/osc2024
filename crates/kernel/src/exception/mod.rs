pub mod asynchronous;

use aarch64_cpu::registers::{CurrentEL, ELR_EL2, HCR_EL2, SPSR_EL2, SP_EL1};
use tock_registers::interfaces::{Readable, Writeable};

pub enum PrivilegeLevel {
    User,
    Kernel,
    Hypervisor,
    Unknown,
}

pub fn current_privilege_level() -> (PrivilegeLevel, &'static str) {
    use CurrentEL::EL::Value as ElValue;
    use PrivilegeLevel as PL;

    match CurrentEL.read_as_enum(CurrentEL::EL) {
        Some(ElValue::EL2) => (PL::Hypervisor, "EL2"),
        Some(ElValue::EL1) => (PL::Kernel, "EL1"),
        Some(ElValue::EL0) => (PL::User, "EL0"),
        _ => (PL::Unknown, "Unknown"),
    }
}

#[inline(always)]
pub fn prepare_el2_to_el1(stack_end_addr: u64, exception_return_addr: u64) {
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

    ELR_EL2.set(exception_return_addr);

    // Set up stack pointer for EL1.
    SP_EL1.set(stack_end_addr);
}
