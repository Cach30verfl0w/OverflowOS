#![no_std]
#![no_main]

use core::panic::PanicInfo;
use uefi::prelude::{Boot, SystemTable};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
extern "C" fn kernel_entry(system_table: &mut SystemTable<Boot>) {
    system_table.stdout().clear().unwrap();
    panic!("Test\n");
}