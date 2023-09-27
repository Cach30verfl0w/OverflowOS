use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::convert::TryFrom;
use uefi::{CString16, Handle, Identify};
use uefi::prelude::BootServices;
use uefi::proto::media::file::{Directory, File, FileAttribute, FileMode, RegularFile};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::{ScopedProtocol, SearchType};
use crate::error::Error;
use crate::SYSTEM_TABLE;

pub(crate) struct SimpleFileSystemProvider<'a> {
    volume_handles: Vec<Handle>,
    open_volumes: Vec<(usize, Directory)>,
    boot_services: &'a BootServices
}

impl<'a> SimpleFileSystemProvider<'a> {

    pub(crate) fn new() -> Result<Self, Error> {
        let mut value = Self {
            volume_handles: Vec::new(),
            open_volumes: Vec::new(),
            boot_services: unsafe { SYSTEM_TABLE.as_ref() }.unwrap().boot_services()
        };
        value.reload()?;
        Ok(value)
    }

    pub(crate) fn reload(&mut self) -> Result<(), Error> {
        let boot_services = unsafe { SYSTEM_TABLE.as_ref() }.unwrap().boot_services();
        let handle_buffer = boot_services.locate_handle_buffer(SearchType::ByProtocol(&SimpleFileSystem::GUID))
            .map_err(|err| err.status())?;
        self.volume_handles = handle_buffer.into_iter().map(|handle| *handle).collect();
        self.open_volumes.clear();
        boot_services.free_pool(handle_buffer.as_ptr() as _).unwrap();
        Ok(())
    }

    pub(crate) fn open_volume<'b>(&mut self, index: usize) -> Result<(), Error> where 'a: 'b {
        if self.open_volumes.iter().any(|(idx, _)| *idx == index) {
            return Err(Error::ResourceOpen);
        }

        match self.volume_handles.get(index) {
            Some(handle) => {
                let mut protocol: ScopedProtocol<'b, SimpleFileSystem> = self.boot_services.open_protocol_exclusive(*handle)
                    .map_err(|err| err.status())?;
                self.open_volumes.push((index, protocol.open_volume().map_err(|err| err.status())?));
                Ok(())
            },
            None => Err(Error::OutOfBounds(index))
        }
    }

    pub(crate) fn open_file(&mut self, volume_index: usize, file_path: Cow<str>, mode: FileMode) -> Result<RegularFile, Error> {
        match self.open_volumes.iter_mut().find(|(index, _)| *index == volume_index).map(|(_, dir)| dir) {
            Some(volume) => {
                // Get file handle
                let file_handle = volume.open(CString16::try_from(file_path.as_ref())?.as_ref(), mode, FileAttribute::empty())
                    .map_err(|err| err.status())?;

                // Get handle as file
                match file_handle.into_regular_file() {
                    Some(file_handle) => Ok(file_handle),
                    None => Err(Error::NotFile)
                }
            },
            None => Err(Error::ResourceNotOpen)
        }
    }

    pub(crate) fn found_volumes(&self) -> usize {
        self.volume_handles.len()
    }

}