use core::arch::asm;

pub mod gdt;

#[inline]
pub fn wait_for_interrupts() {
    unsafe { asm!("hlt") }
}