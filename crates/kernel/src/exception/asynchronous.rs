use aarch64_cpu::registers::DAIF;
use tock_registers::interfaces::Readable;

trait DaifField {
    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register>;
}

struct Debug;
struct SError;
#[allow(clippy::upper_case_acronyms)]
struct IRQ;
#[allow(clippy::upper_case_acronyms)]
struct FIQ;

impl DaifField for Debug {
    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
        DAIF::D
    }
}

impl DaifField for SError {
    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
        DAIF::A
    }
}

impl DaifField for IRQ {
    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
        DAIF::I
    }
}

impl DaifField for FIQ {
    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
        DAIF::F
    }
}

fn is_masked<T>() -> bool
where
    T: DaifField,
{
    DAIF.is_set(T::daif_field())
}

/// Print the AArch64 exceptions status
pub fn print_state() {
    macro_rules! print_field {
        ($ty:ident) => {
            crate::println!(
                "{:>16}: {}",
                ::core::stringify!($ty),
                if is_masked::<$ty>() {
                    "Masked"
                } else {
                    "Unmasked"
                }
            );
        };
    }

    print_field!(Debug);
    print_field!(SError);
    print_field!(IRQ);
    print_field!(FIQ);
}
