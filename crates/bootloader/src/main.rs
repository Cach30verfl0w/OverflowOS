#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(abi_x86_interrupt)]

extern crate alloc;

pub(crate) mod error;
pub(crate) mod file;

use libcpu::halt_cpu;
use libgraphics::embedded_graphics::{
    mono_font::ascii,
    pixelcolor::Rgb888,
    prelude::RgbColor,
};
use uefi::{
    entry,
    prelude::Boot,
    table::SystemTable,
    Handle,
    Status,
};
use uefi_services::system_table;

#[entry]
fn main(_image_handle: Handle, mut sys_table: SystemTable<Boot>) -> Status {
    // Clear stdout and if failed, abort execution of bootloader. After that, initialize uefi services
    if let Err(status) = sys_table.stdout().clear().map_err(|err| err.status()) {
        return status;
    }

    if let Err(status) = uefi_services::init(&mut sys_table).map_err(|err| err.status()) {
        return status;
    }

    // Initiate Graphics Driver
    libgraphics::create_context(unsafe { system_table().as_ref() }.boot_services()).unwrap();
    libgraphics::fill_buffer(Rgb888::BLACK).unwrap();
    libgraphics::swap_buffers().unwrap();

    // Initiate Text Writer of the Graphics Driver
    libgraphics::text::create_text_writer_context(ascii::FONT_8X13).unwrap();
    libgraphics::text::write_str("Test").unwrap();
    libgraphics::text::invalidate_text_write_context().unwrap();
    libgraphics::swap_buffers().unwrap();
    halt_cpu();
}
