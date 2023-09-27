use core::arch::asm;

#[inline]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) fn yield_cpu() {
    unsafe {
        asm!("hlt");
    }
}

#[inline]
#[cfg(any(target_arch = "arm", target_arch = "arm64"))]
pub(crate) fn yield_cpu() {
    unsafe {
        asm!("wfi");
    }
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "arm", target_arch = "arm64")))]
pub(crate) fn yield_cpu() {
    unsafe {
        asm!("nop");
    }
}

#[inline]
pub(crate) fn halt_cpu() -> ! {
    loop {
        yield_cpu();
    }
}