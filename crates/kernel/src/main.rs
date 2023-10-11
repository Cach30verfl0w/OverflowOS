#![no_std]
#![no_main]

extern crate log;

use core::panic::PanicInfo;
use log::info;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}

extern "C" fn main() {
    info!("Welcome to the Kernel lol");
}
