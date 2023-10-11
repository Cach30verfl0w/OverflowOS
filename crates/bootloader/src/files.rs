use crate::error::Error;
use alloc::vec::Vec;
use log::info;
use uefi::{
    prelude::BootServices,
    proto::media::{
        file::{
            Directory,
            File,
            FileAttribute,
            FileInfo,
            FileMode,
        },
        fs::SimpleFileSystem,
    },
    table::boot::{
        MemoryType,
        ScopedProtocol,
        SearchType,
    },
    CString16,
    Identify,
};

pub(crate) struct SimpleFileSystemContext<'a> {
    pub(crate) volumes: Vec<Directory>,
    pub(crate) boot_services: &'a BootServices,
}

pub fn init_file_system_driver<'a>(
    boot_services: &BootServices,
) -> Result<SimpleFileSystemContext, Error> {
    // Get all SimpleFileSystem handles and create volumes vector
    let handle_buffer =
        boot_services.locate_handle_buffer(SearchType::ByProtocol(&SimpleFileSystem::GUID))?;
    let mut volumes = Vec::new();

    // Enumerate handles and acquire directories
    for (i, handle) in handle_buffer.iter().enumerate() {
        // Get protocol and open volumes to directory
        let mut protocol: ScopedProtocol<SimpleFileSystem> =
            boot_services.open_protocol_exclusive(*handle)?;
        let directory = protocol.open_volume()?;

        // Notify user and and push directory into volumes vector
        info!("Successfully opened File System Protocol #{} and acquired volume handle\n", i + 1);
        volumes.push(directory);
    }

    // Create file system context
    Ok(SimpleFileSystemContext {
        volumes,
        boot_services,
    })
}

pub fn read_file<'a>(
    context: &mut SimpleFileSystemContext, index: usize, file_name: &str,
) -> Result<&'a mut [u8], Error> {
    // Open file for read
    let mut handle = context.volumes.get_mut(index).unwrap()
        .open(CString16::try_from(file_name)?.as_ref(), FileMode::Read, FileAttribute::empty())?
        .into_regular_file()
        .unwrap();

    // Create buffer in size of file
    let info = handle.get_boxed_info::<FileInfo>().unwrap();
    let buffer = context
        .boot_services
        .allocate_pool(MemoryType::LOADER_DATA, info.file_size() as usize)
        .unwrap();
    let buffer = unsafe { core::slice::from_raw_parts_mut(buffer, info.file_size() as usize) };

    // Read file
    handle.read(buffer)?;
    Ok(buffer)
}
