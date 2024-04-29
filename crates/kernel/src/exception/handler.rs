use core::{arch::global_asm, cell::UnsafeCell};

use aarch64_cpu::{
    asm::barrier,
    registers::{ESR_EL1, FAR_EL1, SPSR_EL1, VBAR_EL1},
};
use tock_registers::{
    interfaces::{Readable, Writeable},
    registers::InMemoryRegister,
};

global_asm!(include_str!("exception.s"));

#[repr(transparent)]
struct SpsrEL1(InMemoryRegister<u64, SPSR_EL1::Register>);
struct EsrEL1(InMemoryRegister<u64, ESR_EL1::Register>);

#[repr(C)]
struct ExceptionContext {
    /// general perpose registers
    gpr: [u64; 30],

    /// link register (x30)
    lr: u64,

    /// exception link register
    elr_el1: u64,

    /// saved program status register
    spsr_el1: SpsrEL1,

    /// exception syndrome register
    esr_el1: EsrEL1,
}

fn default_exception_handler(exc: &ExceptionContext, kind: &str) {
    panic!(
        "CPU Exception! (exception kind: '{}')\n\n\
        {}",
        kind, exc
    );
}

// Current Exception Level with SP_EL0

#[no_mangle]
extern "C" fn current_el0_synchronous(_e: &mut ExceptionContext) {
    panic!("Should not be here. Use of SP_EL0 in EL1 is not supported.")
}

#[no_mangle]
extern "C" fn current_el0_irq(_e: &mut ExceptionContext) {
    panic!("Should not be here. Use of SP_EL0 in EL1 is not supported.")
}

#[no_mangle]
extern "C" fn current_el0_serror(_e: &mut ExceptionContext) {
    panic!("Should not be here. Use of SP_EL0 in EL1 is not supported.")
}

// Current Exception Level with SP_ELx, x > 0

#[no_mangle]
extern "C" fn current_elx_synchronous(e: &mut ExceptionContext) {
    default_exception_handler(e, "current_elx_synchronous");
}

#[no_mangle]
extern "C" fn current_elx_irq(e: &mut ExceptionContext) {
    default_exception_handler(e, "current_elx_irq");
}

#[no_mangle]
extern "C" fn current_elx_serror(e: &mut ExceptionContext) {
    default_exception_handler(e, "current_elx_serror");
}

// Lower Exception Level, Using AArch64

#[no_mangle]
extern "C" fn lower_aarch64_synchronous(e: &mut ExceptionContext) {
    default_exception_handler(e, "lower_aarch64_synchronous");
}

#[no_mangle]
extern "C" fn lower_aarch64_irq(e: &mut ExceptionContext) {
    default_exception_handler(e, "lower_aarch64_irq");
}

#[no_mangle]
extern "C" fn lower_aarch64_serror(e: &mut ExceptionContext) {
    default_exception_handler(e, "lower_aarch64_serror");
}

// Lower Exception Level, Using AArch32

#[no_mangle]
extern "C" fn lower_aarch32_synchronous(e: &mut ExceptionContext) {
    default_exception_handler(e, "lower_aarch32_synchronous");
}

#[no_mangle]
extern "C" fn lower_aarch32_irq(e: &mut ExceptionContext) {
    default_exception_handler(e, "lower_aarch32_irq");
}

#[no_mangle]
extern "C" fn lower_aarch32_serror(e: &mut ExceptionContext) {
    default_exception_handler(e, "lower_aarch32_serror");
}

impl core::fmt::Display for SpsrEL1 {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "SPSR_EL1: {:#016x}", self.0.get())?;

        let to_flag_str = |x| if x { "Set" } else { "Not set" };
        writeln!(f, "      Flags:")?;
        writeln!(f, "            Negative (N): {}", to_flag_str(self.0.is_set(SPSR_EL1::N)))?;
        writeln!(f, "            Zero     (Z): {}", to_flag_str(self.0.is_set(SPSR_EL1::Z)))?;
        writeln!(f, "            Carry    (C): {}", to_flag_str(self.0.is_set(SPSR_EL1::C)))?;
        writeln!(f, "            Overflow (V): {}", to_flag_str(self.0.is_set(SPSR_EL1::V)))?;

        let to_mask_str = |x| if x { "Masked" } else { "Unmasked" };
        writeln!(f, "      Exception handling state:")?; 
        writeln!(f, "            Debug  (D): {}", to_mask_str(self.0.is_set(SPSR_EL1::D)))?;
        writeln!(f, "            SError (A): {}", to_mask_str(self.0.is_set(SPSR_EL1::A)))?;
        writeln!(f, "            IRQ    (I): {}", to_mask_str(self.0.is_set(SPSR_EL1::I)))?;
        writeln!(f, "            FIQ    (F): {}", to_mask_str(self.0.is_set(SPSR_EL1::F)))?;

        write!(f, "      Illegal Execution State (IL): {}", to_flag_str(self.0.is_set(SPSR_EL1::IL)))?;

        Ok(())
    }
}

impl EsrEL1 {
    #[inline(always)]
    fn exception_class(&self) -> Option<ESR_EL1::EC::Value> {
        self.0.read_as_enum(ESR_EL1::EC)
    }
}

impl core::fmt::Display for EsrEL1 {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "ESR_EL1: {:#016x}", self.0.get())?;

        write!(f, "      Exception Class         (EC) : {:#x}", self.0.read(ESR_EL1::EC))?;

        let ec_translation = match self.exception_class() {
            Some(ESR_EL1::EC::Value::DataAbortCurrentEL) => "Data Abort, current EL",
            _ => "N/A",
        };
        writeln!(f, " - {}", ec_translation)?;

        write!(f, "      Instr Specific Syndrome (ISS): {:#x}", self.0.read(ESR_EL1::ISS))?;

        Ok(())
    }
}

impl ExceptionContext {
    #[inline(always)]
    fn exception_class(&self) -> Option<ESR_EL1::EC::Value> {
        self.esr_el1.exception_class()
    }

    #[inline(always)]
    fn fault_address_valid(&self) -> bool {
        use ESR_EL1::EC::Value as EC;

        match self.exception_class() {
            None => false,
            Some(ec) => matches!(
                ec,
                EC::InstrAbortLowerEL
                    | EC::InstrAbortCurrentEL
                    | EC::PCAlignmentFault
                    | EC::DataAbortLowerEL
                    | EC::DataAbortCurrentEL
                    | EC::WatchpointLowerEL
                    | EC::WatchpointCurrentEL
            ),
        }
    }
}

impl core::fmt::Display for ExceptionContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "{}", self.esr_el1)?;

        if self.fault_address_valid() {
            writeln!(f, "FAR_EL1: {:#016x}", FAR_EL1.get())?;
        }

        writeln!(f, "{}", self.spsr_el1)?;
        writeln!(f, "ELR_EL1: {:#016x}", self.elr_el1)?;
        writeln!(f)?;

        let alternating = |x| if x % 2 == 0 { "  " } else { "\n" };
        writeln!(f, "General purpose registers:")?;
        for (i, reg) in self.gpr.iter().enumerate() {
            write!(f, "      x{:<2}: {:#016x}{}", i, reg, alternating(i))?;
        }

        write!(f, "      lr : {:#016x}", self.lr)?;

        Ok(())
    }
}

pub unsafe fn init_exception_handling() {
    extern "Rust" {
        static __exception_vector_start: UnsafeCell<()>;
    }

    VBAR_EL1.set(__exception_vector_start.get() as u64);

    barrier::isb(barrier::SY);
}
