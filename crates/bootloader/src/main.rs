#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(abi_x86_interrupt)]

pub(crate) mod error;
pub(crate) mod files;

extern crate alloc;

use core::fmt::Write;
use libcpu::{
    halt_cpu
};
use libgraphics::embedded_graphics::{
    mono_font::ascii,
    pixelcolor::Rgb888,
    prelude::RgbColor,
    Drawable,
};
use uefi::{
    allocator,
    entry,
    prelude::Boot,
    table::SystemTable,
    Handle,
    Status,
};

use crate::{
    error::Error,
    files::init_file_system_driver,
};
use core::{
    panic::PanicInfo,
    ptr::NonNull,
};
use libgraphics::text::{
    next_row,
    TEXT_WRITER_CONTEXT,
};
use log::{
    error,
    info,
};
use uefi::{
    prelude::{
        BootServices,
        RuntimeServices,
    },
    table::runtime::ResetType,
};

static mut BOOT_SERVICES: Option<NonNull<BootServices>> = None;
static mut RUNTIME_SERVICES: Option<NonNull<RuntimeServices>> = None;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Show error with message
    error!("Unrecoverable Error while booting into OverflowOS: ");
    let context = unsafe { TEXT_WRITER_CONTEXT.as_mut() }.unwrap();
    if let Some(message) = info.message() {
        context.write_fmt(message.clone()).unwrap();
    } else {
        libgraphics::text::write_str("No error message provided").unwrap();
    }
    next_row().unwrap();

    // Show location
    if let Some(location) = info.location() {
        error!(" => Error found in {} on {}:{}", location.file(), location.line(), location.column())
    }

    // Wait 10 seconds and shutdown computer
    unsafe {
        BOOT_SERVICES.unwrap().as_ref().stall(10000000);
        RUNTIME_SERVICES
            .unwrap()
            .as_ref()
            .reset(ResetType::SHUTDOWN, Status::LOAD_ERROR, None)
    }
}

fn init_graphics(boot_services: &BootServices) -> Result<(), Error> {
    libgraphics::create_context(boot_services)?;
    libgraphics::text::create_text_writer_context(ascii::FONT_7X14_BOLD).unwrap();
    libgraphics::fill_buffer(Rgb888::BLACK).unwrap();
    libgraphics::swap_buffers().unwrap();
    libgraphics::log::install_logger().unwrap();
    Ok(())
}

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    unsafe {
        allocator::init(system_table.boot_services());
        BOOT_SERVICES = NonNull::new(system_table.boot_services() as *const _ as *mut _);
        RUNTIME_SERVICES = NonNull::new(system_table.runtime_services() as *const _ as *mut _);
    }

    // Clear stdout and if failed, abort execution of bootloader. After that, initialize uefi services
    if let Err(status) = system_table.stdout().clear().map_err(|err| err.status()) {
        return status;
    }

    // Initiate Graphics Driver with Logger and display welcome message with resolution information
    if let Err(error) = init_graphics(system_table.boot_services()) {
        panic!("Unable to initialize Graphics => {} (Shutdown in 10 seconds)", error);
    }

    let (width, height) = libgraphics::resolution().unwrap();
    info!("Welcome to OverflowOS Bootloader v{}\n", env!("CARGO_PKG_VERSION"));
    info!("Detected resolution of {}x{} pixels\n", width, height);

    // Initialize file system over simple file system driver
    let mut file_system_context = match init_file_system_driver(system_table.boot_services()) {
        Err(error) => {
            panic!("Unable to initialize File System Driver => {} (Shutdown in 10 seconds)", error)
        }
        Ok(context) => context,
    };

    // Load kernel into memory
    let kernel_data = files::read_file(&mut file_system_context, 0, "\\EFI\\BOOT\\KERNEL.ELF").unwrap();
    info!("Loaded {} kB of Kernel data into the memory\n", kernel_data.len() / 1024);
    
    // Exit Boot Services and notify user about that
    let (system_table, _) = system_table.exit_boot_services();
    unsafe { RUNTIME_SERVICES = NonNull::new(system_table.runtime_services() as *const _ as *mut _) };
    info!("Exited UEFI Boot Services, system is now in Runtime Services\n");

    info!("CPU is now halting!");
    halt_cpu();
}
