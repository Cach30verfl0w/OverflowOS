use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Process '{0}' exited with exit code {1}")]
    FailedProcess(String, i32),
    #[error("System IO: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Cargo Reader: {0}")]
    CargoError(#[from] cargo_toml::Error),
    #[error("Invalid cargo manifest at {0}")]
    InvalidManifest(String),
    #[error("FromUTF8Error: {0}")]
    Utf8Error(#[from] FromUtf8Error),
    #[error("Illegal parameter '{0}'")]
    InvalidParameter(String),
}
