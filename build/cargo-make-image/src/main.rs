#![feature(absolute_path)]

pub mod arch;
pub mod build;
pub mod command;
pub mod error;
pub mod image;

use crate::{
    arch::Architecture,
    build::build_projects_with_cargo,
    command::run_command,
};
use clap::Parser;
use colorful::{
    Color,
    Colorful,
};
use log::{
    debug,
    error,
    info,
    Level,
};
use std::{
    env,
    path::{
        Path,
        PathBuf,
    },
    process::exit,
};

// https://stackoverflow.com/questions/37498864/finding-executable-in-path-with-rust/37499032#37499032
fn find_in_path<P>(exe_name: P) -> Option<PathBuf>
where
    P: AsRef<Path>,
{
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .filter_map(|dir| {
                let full_path = dir.join(&exe_name);
                if full_path.is_file() {
                    Some(full_path)
                } else {
                    None
                }
            })
            .next()
    })
}

#[derive(Parser, Debug)]
#[command(author, version)]
pub struct Arguments {
    /// The name of the image (inclusive .img) for the generation of the image and ISO file
    #[arg(long, default_value = "overflow.img")]
    image_file: String,

    /// The name of the ISO file (inclusive .iso) for the generation of the ISO from the image file
    #[arg(long, default_value = "overflow.iso")]
    iso_file: String,

    /// Redirect the output of the subprocesses to the stdout and stderr of this processs
    #[arg(long, short, default_value_t = false)]
    stdout_redirect: bool,

    /// After the image generation, run the generated image in QEMU
    #[arg(long, short, default_value_t = false)]
    qemu_run: bool,

    /// The name of the bootloader crate
    #[arg(long, short, default_value = "bootloader")]
    bootloader: String,

    /// Non-default folder for the QEMU system executables
    #[arg(long)]
    qemu_system_folder: Option<String>,

    /// Architectural target of the operating system and the QEMU runner
    #[arg(long, short, default_value_t = Architecture::system())]
    architecture: Architecture,

    #[arg(long)]
    entrypoints: Vec<String>,

    /// The size of all sectors in the image file
    #[arg(long, default_value_t = 512)]
    sector_size: u16,

    /// The count of sectors in the image file
    #[arg(long, default_value_t = 93750)]
    sector_count: u32,

    /// The path to the OVMF firmware, if you run QEMU
    #[arg(long, default_value = "OVMF.fd")]
    ovmf_path: String,

    /// Set the log level
    #[arg(long, short)]
    level: Option<Level>,
}
fn main() {
    // Initialize header and print header
    simple_logger::init_with_env().unwrap();
    info!("{}", "   ____                  ______              ____  _____   ".gradient(Color::Blue));
    info!("{}", "  / __ \\_   _____  _____/ __/ /___ _      __/ __ \\/ ___/ ".gradient(Color::Blue));
    info!("{}", " / / / / | / / _ \\/ ___/ /_/ / __ \\ | /| / / / / /\\__ \\".gradient(Color::Blue));
    info!("{}", "/ /_/ /| |/ /  __/ /  / __/ / /_/ / |/ |/ / /_/ /___/ /    ".gradient(Color::Blue));
    info!("{}", "\\____/ |___/\\___/_/  /_/ /_/\\____/|__/|__/\\____//____/ ".gradient(Color::Blue));
    info!(
        "     {}, written in {} with {}",
        "Operating System".gradient(Color::Blue),
        "Rust".color(Color::Orange3),
        "UEFI".red()
    );
    info!("            Developed by {}", "Cach30verfl0w".gradient(Color::Red));
    // Get arguments, configure logger and get target architecture
    let args = Arguments::parse();
    log::set_max_level(args.level.unwrap_or(Level::Info).to_level_filter());
    debug!(
        "Current target architecture {} {}",
        "=>".color(Color::DarkGray),
        String::from(args.architecture).color(Color::LightGreen2)
    );

    // Get and validate path of Cargo
    let cargo_path = match find_in_path("cargo") {
        Some(path) => path,
        None => {
            error!(
                "Unable to recognize path of Cargo {} {} Cargo executable found in $PATH",
                "=>".color(Color::DarkGray),
                "No".red()
            );
            exit(-1);
        }
    };

    // Check for crates directory
    let crates_directory = Path::new("crates");
    if !crates_directory.exists() {
        error!(
            "Unable to build project => {} to find crates directory! Aborting execution...",
            "Unable".red()
        );
        exit(-2);
    }

    // Build all projects
    if let Err(error) = build_projects_with_cargo(cargo_path, crates_directory, &args) {
        error!("Unable to build OverflowOS crates => {}", error);
        exit(-3);
    }

    // Run built image in QEMU
    if args.qemu_run {
        info!("Preparing for running the Operating System image in QEMU");

        // Generate QEMU path and validate
        let qemu_executable = format!("qemu-system-{}", String::from(args.architecture));
        debug!("Searching for '{}' in arguments or path", qemu_executable.clone().color(Color::Red));
        let qemu_path = args
            .qemu_system_folder
            .map(|path| Path::new(&path).join(&qemu_executable))
            .unwrap_or(find_in_path(qemu_executable).unwrap_or_else(|| {
                error!(
                    "Unable to recognize path of QEMU {} {} QEMU executable found in arguments or \
                     $PATH",
                    "=>".color(Color::DarkGray),
                    "No".red()
                );
                exit(-4);
            }));

        if !qemu_path.exists() {
            error!(
                "Unable to recognize path of QEMU {} {} QEMU executable found in arguments or $PATH",
                "=>".color(Color::DarkGray),
                "No".red()
            );
            exit(-4);
        }

        // Run QEMU
        info!("Run QEMU, redirect subprocess stdout and stderr to process stdout and stderr");
        match run_command(
            qemu_path.as_path(),
            None,
            &[
                "-bios",
                &args.ovmf_path,
                "-cdrom",
                &args.iso_file,
                "-m",
                "512",
            ],
            true,
        ) {
            Ok(()) => {}
            Err(error) => {
                error!("Unable to execute QEMU => {}", error);
                exit(-5);
            }
        }
    }

    info!("Finished execution. Thanks for using this tool");
}
