use crate::error::Error;

pub struct SimpleFileSystemDriver {}

pub fn init_file_system_driver() -> Result<SimpleFileSystemDriver, Error> {
    Ok(SimpleFileSystemDriver {})
}
