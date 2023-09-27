#![no_std]
#![no_main]

use core::panic::PanicInfo;
use uefi::{entry, Handle, Status};
use uefi::prelude::{Boot, SystemTable};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[entry]
fn main(_image_handle: Handle, mut _system_table: SystemTable<Boot>) -> Status {
    Status::SUCCESS
}