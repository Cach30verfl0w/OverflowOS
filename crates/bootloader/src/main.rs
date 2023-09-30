#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(abi_x86_interrupt)]

extern crate alloc;

pub(crate) mod elf_loader;
pub(crate) mod error;
pub(crate) mod file;

use crate::{
    elf_loader::parse_elf_file,
    file::SimpleFileSystemProvider,
};
use alloc::{
    borrow::Cow,
    vec,
};
use core::ffi::c_void;
use libcpu::{
    cpuid::get_cpu_features,
    gdt::{
        GDTDescriptor,
        GlobalDescriptorTable,
    },
    halt_cpu,
    idt::{
        Exception,
        GateType,
        IDTDescriptor,
        InterruptDescriptorTable,
        InterruptStackFrame,
    },
    DescriptorTable,
    PrivilegeLevel,
    SegmentSelector,
};
use log::{
    info,
    LevelFilter,
};
use uefi::{
    entry,
    prelude::{
        Boot,
        SystemTable,
    },
    proto::media::file::{
        File,
        FileInfo,
        FileMode,
    },
    Handle,
    Status,
};

extern "x86-interrupt" fn test_interrupt(stack_frame: &mut InterruptStackFrame) {
    halt_cpu();
}

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    // Clear stdout and if failed, abort execution of bootloader. After that, initialize uefi services
    if let Err(status) = system_table.stdout().clear().map_err(|err| err.status()) {
        return status;
    }

    if let Err(status) = uefi_services::init(&mut system_table).map_err(|err| err.status()) {
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
        .open_file(0, Cow::Borrowed("KERNEL.ELF"), FileMode::Read)
        .unwrap_or_else(|err| panic!("Unable to open Kernel as file: {}", err));

    // Read kernel
    let file_size = file
        .get_boxed_info::<FileInfo>()
        .unwrap_or_else(|err| panic!("Unable to read Kernel as file: {}", err));
    let mut file_buffer = vec![0; file_size.file_size() as usize];
    file.read(file_buffer.as_mut_slice())
        .unwrap_or_else(|err| panic!("Unable to read Kernel as file: {}", err));

    // Parse as ELF file
    let function = parse_elf_file(file_buffer.as_slice())
        .unwrap_or_else(|err| panic!("Unable to load Kernel: {}", err));
    let (runtime_services, memory_map) = system_table.exit_boot_services();

    let mut global_descriptor_table = GlobalDescriptorTable::default();
    global_descriptor_table.insert(1, GDTDescriptor::code_segment(PrivilegeLevel::KernelSpace));
    global_descriptor_table.insert(2, GDTDescriptor::data_segment(PrivilegeLevel::KernelSpace));
    global_descriptor_table.load();

    let mut interrupt_descriptor_table = InterruptDescriptorTable::default();
    /*interrupt_descriptor_table.insert(
        Exception::Division as usize,
        IDTDescriptor::new(
            (test_interrupt as *const ()) as u64,
            SegmentSelector::new(1, DescriptorTable::GDT, PrivilegeLevel::KernelSpace),
            GateType::Trap,
            PrivilegeLevel::KernelSpace,
        ),
    );*/
    interrupt_descriptor_table.load();
    return Status::SUCCESS;
}
