use core::arch::asm;

#[inline]
pub fn wait_for_interrupts() {
    unsafe { asm!("wfi") }
}