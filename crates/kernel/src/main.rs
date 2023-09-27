#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
unsafe extern "cdecl" fn kernel_entry(value: i32) -> i32 {
    value
}