#![feature(tuple_trait)]
#![feature(unboxed_closures)]
#![feature(abi_x86_interrupt)]
#![no_std]

extern crate alloc;

#[cfg(target_pointer_width = "64")]
/// This type represents an address to a memory block in 64-bit architecture systems
type MemoryAddress = u64;

#[cfg(target_pointer_width = "32")]
/// This type represents an address to a memory block on 32-bit architecture systems
type MemoryAddress = u32;

#[cfg(any(target_arch = "x86_64", target_arch = "x86_64"))]
mod x86;

#[cfg(any(target_arch = "x86_64", target_arch = "x86_64"))]
pub use x86::*;

// ARM/ARM64
#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
mod arm;

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
pub use arm::*;

#[cfg(not(any(
    target_arch = "arm",
    target_arch = "aarch64",
    target_arch = "x86",
    target_arch = "x86_64"
)))]
compile_error!("Unsupported architecture");

pub fn halt_cpu() -> ! {
    loop {
        wait_for_interrupts();
    }
}
