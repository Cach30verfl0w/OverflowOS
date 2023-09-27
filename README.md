# OverflowOS
OverflowOS is a UEFI-based Operating System with a monolithic Kernel, fully written in Rust. The system library of OverflowOS is not compatible with POSIX, but later I want to provide a library for POSIX-compatibility.

## Install Dependencies
Here are a few steps to install all dependencies to setup a development environment for OverflowOS

**Debian/Ubuntu**
```bash
$> sudo apt update -y && sudo apt install -y qemu-system ovmf xorriso
$> cd OverflowOS
$> cargo install --path build/cargo-make-image
```
After the installation of the dependencies you can execute in the project directory `make-image` to build the ISO file and `make-image -q` to build the image and run it in QEMU.
