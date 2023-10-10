use thiserror_no_std::Error;

#[derive(Error, Debug)]
pub enum Error {
    UEFIError(#[from] uefi::Error),
    OutOfBounds,
    NoContext,
    ContextAlreadyCreated,
}
