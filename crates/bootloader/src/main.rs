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
    string::ToString,
    vec,
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
    table::runtime::ResetType,
    Handle,
    Status,
};
use x86_64::{
    registers::segmentation::{
        Segment,
        CS,
        DS,
        ES,
        FS,
        GS,
        SS,
    },
    structures::{
        gdt::{
            Descriptor,
            GlobalDescriptorTable,
        },
        idt::{
            InterruptDescriptorTable,
        },
    },
};

static mut GLOBAL_DESCRIPTOR_TABLE: GlobalDescriptorTable = GlobalDescriptorTable::new();
static mut INTERRUPT_DESCRIPTOR_TABLE: InterruptDescriptorTable = InterruptDescriptorTable::new();

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
        .open_file(0, Cow::Borrowed("EFI\\BOOT\\KERNEL.ELF"), FileMode::Read)
        .unwrap_or_else(|err| panic!("Unable to open Kernel as file: {}", err));

    // Read kernel
    let file_size = file
        .get_boxed_info::<FileInfo>()
        .unwrap_or_else(|err| panic!("Unable to read Kernel as file: {}", err));
    let mut file_buffer = vec![0; file_size.file_size() as usize];
    file.read(file_buffer.as_mut_slice())
        .unwrap_or_else(|err| panic!("Unable to read Kernel as file: {}", err));

    // Parse as ELF file
    let _entrypoint_function = parse_elf_file(file_buffer.as_slice())
        .unwrap_or_else(|err| panic!("Unable to load Kernel: {}", err));
    let (_runtime_services, _memory_map) = system_table.exit_boot_services();

    unsafe {
        // Load GDT
        let code_selector = GLOBAL_DESCRIPTOR_TABLE.add_entry(Descriptor::kernel_code_segment());
        let data_selector = GLOBAL_DESCRIPTOR_TABLE.add_entry(Descriptor::kernel_data_segment());
        GLOBAL_DESCRIPTOR_TABLE.load();
        CS::set_reg(code_selector);
        DS::set_reg(data_selector);
        SS::set_reg(data_selector);
        FS::set_reg(data_selector);
        ES::set_reg(data_selector);
        GS::set_reg(data_selector);

        // Load IDT
        INTERRUPT_DESCRIPTOR_TABLE.load();
    }

    unsafe { _runtime_services.runtime_services() }.reset(ResetType::SHUTDOWN, Status::SUCCESS, None);
}
