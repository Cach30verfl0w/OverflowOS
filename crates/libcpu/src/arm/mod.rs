use core::arch::asm;

#[inline(always)]
pub fn wait_for_interrupts() {
    unsafe { asm!("wfi", options(noreturn)) }
}
