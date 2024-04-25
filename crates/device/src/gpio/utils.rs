use core::arch::asm;

#[inline(always)]
pub fn spin_for_cycles(n: usize) {
    for _ in 0..n {
        unsafe { asm!("nop") };
    }
}
