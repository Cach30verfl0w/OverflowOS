use std::borrow::Cow;
use std::fs::{File, read_dir};
use std::io::Read;
use std::path::{absolute, Path, PathBuf};
use cargo_toml::Manifest;
use colorful::{Color, Colorful};
use log::debug;
use serde::Deserialize;
use crate::Arguments;
use crate::command::run_command;
use crate::error::Error;

#[derive(Deserialize)]
pub struct BuildConfig {
    pub target: String
}

#[derive(Deserialize)]
pub struct CargoConfig {
    pub build: BuildConfig
}

pub(crate) fn build_projects_with_cargo(cargo_path: PathBuf, crates_directory: &Path, args: &Arguments) -> Result<(), Error> {
    for entry in read_dir(crates_directory)? {
        let entry = entry?;

        // Load manifest file
        let package = match Manifest::from_path(entry.path().join("Cargo.toml"))?.package {
            Some(value) => value,
            None => return Err(Error::InvalidManifest(String::from(entry.path().to_str().unwrap())))
        };
        debug!("Loaded Cargo Manifest in {}", absolute(entry.path()).unwrap().to_str().unwrap().gradient(Color::Green));

        // Read target
        let mut file = File::open(entry.path().join(".cargo/config.toml"))?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        let target = toml::from_str::<CargoConfig>(String::from_utf8(bytes)?
            .as_str())?.build.target;

        // Build project with Cargo
        debug!("Building cargo project {}", package.name);
        run_command(cargo_path.as_path(), None, &[
            Cow::Borrowed("build"),
            Cow::Borrowed("--package"),
            Cow::Owned(package.name),
            Cow::Borrowed("--target"),
            Cow::Owned(target),
            Cow::Borrowed("-Zbuild-std=core,alloc,compiler_builtins"),
            Cow::Borrowed("-Zbuild-std-features=compiler-builtins-mem")
        ], args.stdout_redirect)?;
    }
    Ok(())
}