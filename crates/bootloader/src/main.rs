#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use libcpu::{halt_cpu, PrivilegeLevel, Register};
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

use core::panic::PanicInfo;
use log::info;
use libgraphics::log::install_logger;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    libgraphics::text::write_str("Error, Error").unwrap();
    libgraphics::swap_buffers().unwrap();
    loop {}
}

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    // Clear stdout and if failed, abort execution of bootloader. After that, initialize uefi services
    if let Err(status) = system_table.stdout().clear().map_err(|err| err.status()) {
        return status;
    }

    // Initiate Graphics Driver
    libgraphics::create_context(system_table.boot_services()).unwrap();
    libgraphics::text::create_text_writer_context(ascii::FONT_8X13_BOLD).unwrap();
    libgraphics::fill_buffer(Rgb888::BLACK).unwrap();
    libgraphics::swap_buffers().unwrap();

    // Initiate Logger and display welcome message
    install_logger().unwrap();

    let (width, height) = libgraphics::resolution().unwrap();
    info!("Welcome to OverflowOS Bootloader v{}\n", env!("CARGO_PKG_VERSION"));
    info!("Detected resolution of {}x{} pixels\n", width, height);

    // Exit Boot Services
    let (_, _) = system_table.exit_boot_services();
    info!("Exited UEFI Boot Services, system is now in Runtime Services\n");

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        use libcpu::gdt::*;
        let mut global_descriptor_table = GlobalDescriptorTable::new();
        let code_selector = global_descriptor_table.push(GDTDescriptor::code_segment(PrivilegeLevel::KernelSpace)).unwrap();
        let data_selector = global_descriptor_table.push(GDTDescriptor::data_segment(PrivilegeLevel::KernelSpace)).unwrap();
        global_descriptor_table.load();

        libcpu::set_cs(code_selector);
        libcpu::set_ds(data_selector.0 as Register);
        libcpu::set_ss(data_selector.0 as Register);
        info!("Successfully initialized Global Descriptor Table\n");
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        use libcpu::interrupts::*;
        let interrupt_descriptor_table = InterruptDescriptorTable::default();
        interrupt_descriptor_table.load();
        info!("Successfully initialized Interrupt Descriptor Table\n");
    }

    info!("CPU is now halting!");
    halt_cpu();
}
