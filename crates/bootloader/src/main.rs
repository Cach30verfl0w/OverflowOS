#![no_std]
#![no_main]
#![feature(panic_info_message)]

extern crate alloc;

pub(crate) mod logger;
pub(crate) mod halt;
pub(crate) mod file;
pub(crate) mod error;

use alloc::borrow::Cow;
use core::panic::PanicInfo;
use log::{debug, error, info, Level, LevelFilter};
use uefi::{entry, Handle, Status};
use uefi::prelude::{Boot, SystemTable};
use uefi::proto::media::file::FileMode;
use crate::file::SimpleFileSystemProvider;
use crate::halt::halt_cpu;
use crate::logger::Logger;

static mut SYSTEM_TABLE: Option<SystemTable<Boot>> = None;
static mut LOGGER: Option<Logger> = None;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("Error while running Bootloader => {}", info.message().unwrap());
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
    info!("Welcome to OverflowOS Bootloader v{}\n", env!("CARGO_PKG_VERSION"));

    // Initialize Simple FileSystem
    let mut file_system = match SimpleFileSystemProvider::new() {
        Err(error) => {
            error!("Unable to initialize FileSystem => {}\n", error);
            halt_cpu();
        },
        Ok(provider) => provider
    };
    info!("Successfully initialized File System with {} volume(s)\n", file_system.found_volumes());

    file_system.open_volume(0).unwrap();
    let _file = file_system.open_file(0, Cow::Borrowed("KERNEL.ELF"), FileMode::Read).unwrap();
    debug!("Successfully acquired file handle of KERNEL.ELF");

    halt_cpu();
}