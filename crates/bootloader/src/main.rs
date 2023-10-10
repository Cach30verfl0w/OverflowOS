#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(abi_x86_interrupt)]

extern crate alloc;

pub(crate) mod error;
pub(crate) mod file;
pub(crate) mod graphics;

use crate::{
    file::SimpleFileSystemProvider,
};
use alloc::{
    borrow::Cow,
    string::ToString,
    vec,
};
use embedded_graphics::mono_font::ascii;
use libcpu::{gdt::{
    GDTDescriptor,
    GlobalDescriptorTable,
}, set_cs, set_ds, set_ss, PrivilegeLevel, Register, halt_cpu};
use libcpu::interrupts::{Exception, GateType, IDTDescriptor, InterruptDescriptorTable, InterruptStackFrame};
use log::{
    info,
    LevelFilter,
};
use uefi::{entry, prelude::{
    Boot,
    SystemTable,
}, proto::media::file::{
    File,
    FileInfo,
    FileMode,
}, table::runtime::ResetType, Handle, Status, Identify};
use uefi_services::system_table;
use crate::graphics::{TextWriter, UEFIFramebuffer};

extern "x86-interrupt" fn test(frame: InterruptStackFrame) {
    unsafe {
        frame.ret();
    }
}

#[entry]
fn main(_image_handle: Handle, mut sys_table: SystemTable<Boot>) -> Status {
    // Clear stdout and if failed, abort execution of bootloader. After that, initialize uefi services
    if let Err(status) = sys_table.stdout().clear().map_err(|err| err.status()) {
        return status;
    }

    if let Err(status) = uefi_services::init(&mut sys_table).map_err(|err| err.status()) {
        return status;
    }

    // Move system table into static scope and initialize logging
    log::set_max_level(LevelFilter::Info);

    // Run bootloader
    info!("Welcome to OverflowOS Bootloader v{}", env!("CARGO_PKG_VERSION"));

    // Initialize Simple FileSystem
    let mut file_system = SimpleFileSystemProvider::new()
        .unwrap_or_else(|err| panic!("Unable to initialize FileSystem: {}", err));
    info!("Successfully initialized File System with {} volume(s)", file_system.found_volumes());

    // Open first volume and open kernel
    file_system
        .open_volume(0)
        .unwrap_or_else(|err| panic!("Unable to open own volume: {}", err));

    let mut file = file_system
        .open_file(0, Cow::Borrowed("EFI\\BOOT\\KERNEL.ELF"), FileMode::Read)
        .unwrap_or_else(|err| panic!("Unable to open Kernel as file: {}", err));

    // Read kernel
    let file_size = file
        .get_boxed_info::<FileInfo>()
        .unwrap_or_else(|err| panic!("Unable to read Kernel as file: {}", err));
    let mut file_buffer = vec![0; file_size.file_size() as usize];
    file.read(file_buffer.as_mut_slice())
        .unwrap_or_else(|err| panic!("Unable to read Kernel as file: {}", err));

    // Create Frame Buffer and exit boot services
    let mut framebuffer = UEFIFramebuffer::new(unsafe {
        system_table().as_ref().boot_services()
    }).unwrap();
    let (runtime_table, _memory_map) = sys_table.exit_boot_services();

    // Clear Screen
    framebuffer.clear();
    framebuffer.swap_buffer();

    // Create Text writer
    let mut text_writer = TextWriter::new(&mut framebuffer, ascii::FONT_8X13);

    // Load GDT
    text_writer.write_string("Initializing Global Descriptor Table\n");
    let mut global_descriptor_table = GlobalDescriptorTable::new();
    let code_selector = global_descriptor_table
        .push(GDTDescriptor::code_segment(PrivilegeLevel::KernelSpace))
        .unwrap();
    let data_selector = global_descriptor_table
        .push(GDTDescriptor::data_segment(PrivilegeLevel::KernelSpace))
        .unwrap();
    global_descriptor_table.load();
    set_cs(code_selector);
    set_ds(data_selector.0 as Register);
    set_ss(data_selector.0 as Register);

    // Load IDT
    text_writer.write_string("Initializing Interrupt Descriptor Table\n");
    let mut interrupt_descriptor_table = InterruptDescriptorTable::default();
    interrupt_descriptor_table.load();
    interrupt_descriptor_table.insert(Exception::Division, IDTDescriptor::new(test, GateType::Trap, PrivilegeLevel::KernelSpace));

    halt_cpu();
    unsafe {
        runtime_table
            .runtime_services()
            .reset(ResetType::SHUTDOWN, Status::SUCCESS, None);
    }
}
