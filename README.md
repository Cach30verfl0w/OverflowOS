# OverflowOS
![GitHub](https://img.shields.io/github/license/Cach30verfl0w/OverflowOS) ![GitHub issues](https://img.shields.io/github/issues/Cach30verfl0w/OverflowOS) ![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/Cach30verfl0w/OverflowOS) ![GitHub commit activity (branch)](https://img.shields.io/github/commit-activity/y/Cach30verfl0w/OverflowOS) ![GitHub last commit (branch)](https://img.shields.io/github/last-commit/Cach30verfl0w/OverflowOS/main)
![GitHub pull requests](https://img.shields.io/github/issues-pr/Cach30verfl0w/OverflowOS)

OverflowOS is a UEFI-based Operating System with a monolithic Kernel, fully written in Rust. We support the architectures x86_64 and ARM64, and I'm not planning to implement 32-bit support in the future.

## Current project packages
- [`make-image`](https://github.com/Cach30verfl0w/OverflowOS/tree/main/build/cargo-make-image) - Tooling to generate the ISO file from projects like this (by [Cach30verfl0w](https://github.com/Cach30verfl0w))
- [`kernel`](https://github.com/Cach30verfl0w/OverflowOS/tree/main/crates/kernel) - The original monolithic Kernel of OverflowOS (by [Cach30verfl0w](https://github.com/Cach30verfl0w))
- [`bootloader`](https://github.com/Cach30verfl0w/OverflowOS/tree/main/crates/bootloader) - The original UEFI-based bootloader of OverflowOS (by [Cach30verfl0w](https://github.com/Cach30verfl0w))
- [`libcpu`](https://github.com/Cach30verfl0w/OverflowOS/tree/main/crates/libcpu) - The library for the implementation of architecture-specific CPU features (by [Cach30verfl0w](https://github.com/Cach30verfl0w))

## Install Dependencies
Here are a few steps to install all dependencies to setup a development environment for OverflowOS

**Debian/Ubuntu**
```bash
$> sudo apt update -y && sudo apt install -y qemu-system ovmf xorriso
$> cd OverflowOS
$> cargo install --path build/cargo-make-image
```
After the installation of the dependencies you can execute in the project directory `make-image` to build the ISO file and `make-image -q` to build the image and run it in QEMU.

# Credits
- `x86_64-unknown-none` target from [phil-opp](https://os.phil-opp.com/minimal-rust-kernel/#target-specification)
- Some information from [OSDev.org](https://wiki.osdev.org)
- Information about GDT and IDT from [HackerNoon.com](https://hackernoon.com)
