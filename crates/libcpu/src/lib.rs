#![feature(tuple_trait)]
#![feature(unboxed_closures)]
#![feature(abi_x86_interrupt)]
#![no_std]

extern crate alloc;

#[cfg(any(target_arch = "x86_64", target_arch = "x86_64"))]
mod x86;

#[cfg(any(target_arch = "x86_64", target_arch = "x86_64"))]
pub use x86::*;

// ARM/ARM64
#[cfg(any(target_arch = "arm", target_arch = "arm64"))]
mod arm;

#[cfg(any(target_arch = "arm", target_arch = "arm64"))]
pub use arm::*;

#[cfg(not(any(
    target_arch = "arm",
    target_arch = "arm64",
    target_arch = "x86",
    target_arch = "x86_64"
)))]
compile_error!("Unsupported architecture");

pub fn halt_cpu() -> ! {
    loop {
        wait_for_interrupts();
    }
}
