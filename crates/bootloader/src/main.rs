#![no_std]
#![no_main]

extern crate alloc;

pub(crate) mod logger;
pub(crate) mod halt;

use core::panic::PanicInfo;
use log::{info, Level, LevelFilter};
use uefi::{entry, Handle, Status};
use uefi::prelude::{Boot, SystemTable};
use crate::halt::halt_cpu;
use crate::logger::Logger;

static mut SYSTEM_TABLE: Option<SystemTable<Boot>> = None;
static mut LOGGER: Option<Logger> = None;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    halt_cpu();
}

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    // Clear stdout and if failed, abort execution of bootloader. After that, initialize the memory
    // allocator
    unsafe { uefi::allocator::init(system_table.boot_services()) };
    if let Err(status) = system_table.stdout().clear().map_err(|err| err.status()) {
        return status;
    }

    // Move system table into static scope and initialize logging
    unsafe {
        SYSTEM_TABLE = Some(system_table);
        LOGGER = Some(Logger::new(Level::Trace));
    }
    log::set_max_level(LevelFilter::Trace);
    log::set_logger(unsafe { LOGGER.as_ref() }.unwrap()).unwrap();

    // Run bootloader
    info!("Welcome to OverflowOS Bootloader v{}", env!("CARGO_PKG_VERSION"));
    halt_cpu();
}