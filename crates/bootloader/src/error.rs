use log::SetLoggerError;
use thiserror_no_std::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Graphics Error: {0:?}")]
    Graphics(#[from] libgraphics::error::Error),

    #[error("UEFI Error: {0}")]
    UEFI(#[from] uefi::Error),

    #[error("Logger Error: Unable to set logger")]
    Logger(#[from] SetLoggerError)
}
