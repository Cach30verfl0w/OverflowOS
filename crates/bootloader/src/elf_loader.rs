use core::{mem, slice};
use elf::{abi, ElfBytes};
use elf::endian::AnyEndian;
use log::{debug, info};
use uefi::table::boot::{AllocateType, MemoryType};
use crate::error::Error;
use crate::SYSTEM_TABLE;

pub fn parse_elf_file(file_buffer: &[u8]) -> Result<(), Error> {
    let elf = ElfBytes::<AnyEndian>::minimal_parse(file_buffer)?;
    info!("Successfully read {} kB from kernel file and parsed to ELF file\n", file_buffer.len() / 1000);
    debug!("Imported {} program header(s) and {} section header(s)\n",
        elf.segments().map(|table| table.len()).unwrap_or(0),
        elf.section_headers().map(|table| table.len()).unwrap_or(0)
    );

    let boot_services = unsafe { SYSTEM_TABLE.as_mut() }.unwrap().boot_services();

    // Load segments into memory
    if let Some(segments) = elf.segments() {
        for segment in segments {
            if segment.p_type != abi::PT_LOAD {
                continue;
            }

            debug!("Found load segment at offset 0x{:X} (P: 0x{:2X} V: 0x{:2X})\n", segment.p_offset,
                segment.p_paddr, segment.p_vaddr);

            // Calculate count of pages
            let num_pages: usize = {
                let padding = segment.p_vaddr & 0x0FFF;
                let total_bytes = segment.p_memsz + padding;
                (1 + (total_bytes >> 12)) as usize
            };

            // Allocate and zero page
            let vaddr = boot_services.allocate_pages(
                AllocateType::Address(segment.p_vaddr & !0x0FFF),
                MemoryType::LOADER_CODE,
                num_pages).map_err(|err| err.status())?;
            if vaddr != (segment.p_vaddr & !0x0FFF) {
                panic!("Address changed from 0x{:X} to {:X}", vaddr, (segment.p_vaddr & !0x0FFF));
            }

            unsafe { boot_services.set_mem((segment.p_vaddr & !0x0FFF) as _, num_pages * 4096, 0) };
            info!("Allocated {} pages ({} bytes) on 0x{:X} for KERNEL.ELF\n", num_pages,
                segment.p_memsz, segment.p_vaddr);

            // Insert data into memory
            unsafe {
                slice::from_raw_parts_mut(segment.p_vaddr as *mut u8, segment.p_memsz as usize)
            }.copy_from_slice(elf.segment_data(&segment)?);
        }
    }

    // Locate kernel_entry function
    let entry_point: unsafe extern "cdecl" fn(i32) -> i32 = unsafe { mem::transmute(elf.ehdr.e_entry) };
    info!("A {}", unsafe { entry_point(12) });
    Ok(())
}