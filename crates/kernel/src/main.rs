#![no_std]
#![no_main]

use core::alloc::{GlobalAlloc, Layout};
use core::panic::PanicInfo;
use libcpu::halt_cpu;

pub struct NonAlloc;

unsafe impl GlobalAlloc for NonAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        todo!()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        todo!()
    }
}

#[global_allocator]
static GLOBAL: NonAlloc = NonAlloc;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
unsafe extern "cdecl" fn kernel_entry() {
    halt_cpu();
}
