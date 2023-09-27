use std::env;
use std::process::exit;
use clap::ValueEnum;
use log::error;

#[derive(ValueEnum, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub(crate) enum Architecture {
    X86_64,
    X86,
    ARM,
    ARM64
}

impl From<Architecture> for String {
    #[inline]
    fn from(value: Architecture) -> Self {
        match value {
            Architecture::X86_64 => "x86_64",
            Architecture::X86 => "x86",
            Architecture::ARM => "arm",
            Architecture::ARM64 => "arm64"
        }.to_string()
    }
}

impl Architecture {

    #[inline]
    pub(crate) fn system() -> Architecture {
        match env::consts::ARCH {
            "x86" => Self::X86,
            "x86_64" => Self::X86_64,
            "arm" => Self::ARM,
            "arm64" => Self::ARM64,
            arch => {
                error!("Unable to get system architecture => Unsupported architecture {}", arch);
                exit(-1);
            }
        }
    }

}