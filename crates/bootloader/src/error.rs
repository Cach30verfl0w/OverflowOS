use elf::ParseError;
use thiserror_no_std::Error;
use uefi::{
    data_types::FromStrError,
    Status,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("UEFI firmware operation failed with code {0}")]
    UEFI(#[from] Status),
    // Custom Errors
    #[error("{0} is out of bounds")]
    OutOfBounds(usize),
    #[error("The resource is already open")]
    ResourceOpen,
    #[error("Unable to format string")]
    FromStrError(#[from] FromStrError),
    #[error("The requested resource is not a file")]
    NotFile,
    #[error("{0}")]
    ElfError(#[from] ParseError),
}
