use std::fs::{create_dir, read_dir, remove_dir, remove_file};
use std::path::{absolute, Path, PathBuf};
use std::process::exit;
use cargo_toml::Manifest;
use colorful::{Color, Colorful};
use log::{debug, error, info};
use crate::{Arguments, find_in_path};
use crate::command::run_command;
use crate::error::Error;
use crate::image_generator::ImageGenerator;

pub(crate) fn build_projects_with_cargo(cargo_path: PathBuf, crates_directory: &Path, args: &Arguments) -> Result<(), Error> {
    // Create .image directory
    let image_path = Path::new(".image");
    if !image_path.exists() {
        create_dir(image_path)?;
    }

    // Create Image Generator
    let image_generator = ImageGenerator::new(image_path.join(&args.image_file), args.sector_size, args.sector_count)?;

    // Compile every target
    for entry in read_dir(crates_directory)? {
        let entry = entry?;

        // Load manifest file
        let package = match Manifest::from_path(entry.path().join("Cargo.toml"))?.package {
            Some(value) => value,
            None => return Err(Error::InvalidManifest(String::from(entry.path().to_str().unwrap())))
        };
        debug!("Loaded Cargo Manifest in {}", absolute(entry.path()).unwrap().to_str().unwrap().gradient(Color::Green));

        // Generate target string
        let target = format!("{}-unknown-{}", String::from(args.architecture),
                             if args.bootloader.eq(package.name()) {
                                 "uefi"
                             } else {
                                 "none.json"
                             });

        // Build project with Cargo
        info!("Building cargo project {} with target {}", package.name.clone().gradient(Color::Green),
            target.clone().gradient(Color::Red));
        run_command(cargo_path.as_path(), None, &[
            "build",
            "--package",
            package.name(),
            "--target",
            &target,
            "-Zbuild-std=core,alloc,compiler_builtins",
            "-Zbuild-std-features=compiler-builtins-mem"
        ], args.stdout_redirect)?;

        // Copy file into image
        let library = crates_directory.join(package.name()).join("src/lib.rs").as_path().exists();
        if args.bootloader.eq(package.name()) {
            if args.architecture.is64bit() {
                image_generator.copy_into(format!("target/{}/debug/{}.efi", target, package.name()), "EFI/BOOT/BOOTX64.EFI")?;
            } else {
                image_generator.copy_into(format!("target/{}/debug/{}.efi", target, package.name()), "EFI/BOOT/BOOTIA32.EFI")?;
            }
        } else if !library {
            image_generator.copy_into(format!("target/{}/debug/{}", target.replace(".json", ""),
                                              package.name()), format!("{}.ELF", package.name()).to_uppercase())?;
        }
    }

    // Generate ISO file and cleanup
    if image_path.exists() {
        // TODO: Do ISO file creation cross-platform
        // Generate ISO file
        let xorriso_path = match find_in_path("xorriso") {
            Some(path) => path,
            None => {
                error!("Unable to convert Image to ISO: Xorriso can't be found on this system");
                
                // Cleanup and exit
                remove_file(image_path.join(&args.image_file))?;
                remove_dir(image_path)?;
                exit(-1);
            }
        };

        info!("Create ISO file from Image file");
        run_command(xorriso_path.as_path(), None, &[
            "-as",
            "mkisofs",
            "-V",
            "EFI_ISO_BOOT",
            "-e",
            &args.image_file,
            "-no-emul-boot",
            "-o",
            &args.iso_file,
            ".image/"
        ], args.stdout_redirect)?;

        // Cleanup
        remove_file(image_path.join(&args.image_file))?;
        remove_dir(image_path)?;
    }
    Ok(())
}