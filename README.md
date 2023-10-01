<div align = "center">

# `OverflowOS`
![GitHub](https://img.shields.io/github/license/Cach30verfl0w/OverflowOS) ![GitHub issues](https://img.shields.io/github/issues/Cach30verfl0w/OverflowOS) ![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/Cach30verfl0w/OverflowOS) ![GitHub commit activity (branch)](https://img.shields.io/github/commit-activity/y/Cach30verfl0w/OverflowOS) ![GitHub last commit (branch)](https://img.shields.io/github/last-commit/Cach30verfl0w/OverflowOS/main)
![GitHub pull requests](https://img.shields.io/github/issues-pr/Cach30verfl0w/OverflowOS)

OverflowOS is a UEFI-based Operating System with a monolithic Kernel, fully written in Rust. We support the architectures x86_64 and ARM64, and I'm not planning to implement 32-bit support in the future. You can see my planned features in [this project](https://github.com/users/Cach30verfl0w/projects/5). If you have some ideas, just create [an Issue](https://github.com/Cach30verfl0w/OverflowOS/issues/new).

</div>

## Current project packages
- [`OSImage`](https://github.com/Cach30verfl0w/OSImage) -  Command-Line Tool to generate image files for Rust Operating Systems (by [Cach30verfl0w](https://github.com/Cach30verfl0w))
- [`kernel`](https://github.com/Cach30verfl0w/OverflowOS/tree/main/crates/kernel) - The original monolithic Kernel of OverflowOS (by [Cach30verfl0w](https://github.com/Cach30verfl0w))
- [`bootloader`](https://github.com/Cach30verfl0w/OverflowOS/tree/main/crates/bootloader) - The original UEFI-based bootloader of OverflowOS (by [Cach30verfl0w](https://github.com/Cach30verfl0w))
- [`libcpu`](https://github.com/Cach30verfl0w/OverflowOS/tree/main/crates/libcpu) - The library for the implementation of architecture-specific CPU features (by [Cach30verfl0w](https://github.com/Cach30verfl0w))
    - This library currently only supports the architectures x86 and x86_64, but ARM and RISC-V support is also planned
# Install Dependencies
Here are a few steps to install all dependencies to set up a development environment for OverflowOS.

**Debian/Ubuntu**
```bash
$> sudo apt update -y && sudo apt install -y qemu-system ovmf xorriso
$> git clone https://github.com/Cach30verfl0w/OSImage
$> cd OSImage
$> cargo install --path ./
```

## Run in QEMU
```bash
$> osimage build-image --image-file overflow.img --iso-file overflow.iso
$> osimage run-qemu --iso-file overflow.iso
```

## Credits
- `x86_64-unknown-none` target from [phil-opp](https://os.phil-opp.com/minimal-rust-kernel/#target-specification)
- VGA Text Mode Tutorial from [phil-opp](https://os.phil-opp.com/vga-text-mode/)
- Some information from [OSDev.org](https://wiki.osdev.org)
- Information about GDT and IDT from [HackerNoon.com](https://hackernoon.com)
