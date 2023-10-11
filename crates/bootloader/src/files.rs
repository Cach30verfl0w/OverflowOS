use crate::error::Error;
use alloc::vec::Vec;
use log::info;
use uefi::{
    prelude::BootServices,
    proto::media::{
        file::Directory,
        fs::SimpleFileSystem,
    },
    table::boot::{
        ScopedProtocol,
        SearchType,
    },
    Identify,
};

static mut FILE_SYSTEM_CONTEXT: Option<SimpleFileSystemContext> = None;

pub struct SimpleFileSystemContext {
    volumes: Vec<Directory>,
}

pub fn init_file_system_driver<'a>(boot_services: &BootServices) -> Result<(), Error> {
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
        info!("Successfully opened Protocol #{} and acquired volume handle\n", i + 1);
        volumes.push(directory);
    }

    // Create file system context
    unsafe { FILE_SYSTEM_CONTEXT = Some(SimpleFileSystemContext { volumes }) };
    Ok(())
}
