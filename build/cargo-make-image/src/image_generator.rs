use crate::error::Error;
use colorful::{
    Color,
    Colorful,
};
use fatfs::{
    format_volume,
    FatType,
    FileSystem,
    FormatVolumeOptions,
    FsOptions,
};
use fscommon::BufStream;
use log::debug;
use std::{
    fs,
    fs::File,
    io::{
        Read,
        Write,
    },
    path::Path,
};

pub(crate) struct ImageGenerator {
    file_system: FileSystem<BufStream<File>>,
}

impl ImageGenerator {
    pub(crate) fn new<F: AsRef<Path>>(
        file: F, block_size: u16, block_count: u32,
    ) -> Result<ImageGenerator, Error> {
        // Create zeroed file if exists
        if !file.as_ref().exists() {
            let mut file = File::create(&file)?;
            file.set_len((block_count as u64) * (block_size as u64))?;
            file.write(vec![0; (block_count as usize) * (block_size as usize)].as_slice())?;
        }

        // Format Volume
        let file = fs::OpenOptions::new().read(true).write(true).open(file)?;
        let mut file_buffer = BufStream::new(file);
        format_volume(
            &mut file_buffer,
            FormatVolumeOptions::new()
                .fat_type(FatType::Fat32)
                .bytes_per_sector(block_size)
                .total_sectors(block_count),
        )?;

        let file_system = FileSystem::new(file_buffer, FsOptions::new().update_accessed_date(true))?;
        Ok(Self { file_system })
    }

    pub(crate) fn create_directory<DIR: AsRef<Path>>(&self, directory: DIR) -> Result<(), Error> {
        let directory = directory.as_ref();
        if directory.to_str().unwrap().is_empty() {
            return Ok(());
        }

        if let Some(parent) = directory.parent() {
            self.create_directory(parent)?;
        }

        let root_directory = self.file_system.root_dir();
        root_directory.create_dir(directory.to_str().unwrap())?;
        Ok(())
    }

    pub(crate) fn copy_into<HP: AsRef<Path>, IP: AsRef<Path>>(
        &self, host_file: HP, image_file: IP,
    ) -> Result<(), Error> {
        debug!(
            "Move {} as {} into image",
            host_file.as_ref().to_str().unwrap().gradient(Color::Cyan),
            image_file.as_ref().to_str().unwrap().gradient(Color::Red)
        );

        if image_file.as_ref().is_dir() {
            return Err(Error::InvalidParameter("image_file".to_owned()));
        }

        // Validate directory and create if not exists
        let directory = image_file.as_ref().parent().unwrap();
        if directory.is_dir() {
            return Err(Error::InvalidParameter("image_file".to_owned()));
        }

        // Read host file
        let mut file = fs::OpenOptions::new().read(true).open(&host_file)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        // Ensure directory + create and write file
        if let Some(parent) = image_file.as_ref().parent() {
            self.create_directory(parent)?;
        }

        let mut file = self
            .file_system
            .root_dir()
            .create_file(image_file.as_ref().to_str().unwrap())?;
        file.write_all(bytes.as_slice())?;
        file.flush()?;
        Ok(())
    }
}
