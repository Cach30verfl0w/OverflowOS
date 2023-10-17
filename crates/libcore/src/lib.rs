#![feature(pointer_is_aligned)]
#![no_std]

use core::{
    alloc::{
        GlobalAlloc,
        Layout,
    },
    cell::RefCell,
    slice,
};
use libcpu::MemoryAddress;
use uefi::table::boot::{
    MemoryDescriptor,
    MemoryMap,
};

pub struct FrameTable<'a> {
    pub frame_table: &'a mut [u8],
}

impl FrameTable<'_> {
    pub fn toggle_frame_alloc_status(&mut self, page_index: usize) {
        let frame_block_index = page_index % 8;
        let frame_table_index = page_index - frame_block_index;
        if let Some(value) = self.frame_table.get_mut(frame_table_index) {
            *value ^= (1 << frame_block_index);
        }
    }

    pub fn page_allocated(&mut self, page_index: usize) -> bool {
        let frame_block_index = page_index % 8;
        let frame_table_index = page_index - frame_block_index;
        if let Some(value) = self.frame_table.get_mut(frame_table_index) {
            return (*value & (1 << frame_block_index)) == 1;
        }
        false
    }
}

pub struct FrameAllocator<'a> {
    pub start_address: MemoryAddress,
    pub stop_address: MemoryAddress,
    pub page_size: u16,
    pub frame_table: RefCell<FrameTable<'a>>,
}

unsafe impl GlobalAlloc for FrameAllocator<'_> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let base_page_count = (layout.size() / self.page_size as usize).max(1);
        let pages = base_page_count
            + if (layout.size() - base_page_count) % self.page_size as usize != 0 {
                1
            } else {
                0
            };

        match self.find_first_frame_index(pages) {
            None => panic!("No pages available!"),
            Some(index) => {
                for i in 0..pages {
                    self.frame_table
                        .borrow_mut()
                        .toggle_frame_alloc_status(index + i);
                }
                (self.start_address + (index * 4096) as MemoryAddress) as *mut u8
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let base_page_count = (layout.size() / self.page_size as usize).max(1);
        let pages = base_page_count
            + if (layout.size() - base_page_count) % self.page_size as usize != 0 {
                1
            } else {
                0
            };

        let address = ptr as MemoryAddress;

        let page_index = ((address - self.start_address) / 4096) as usize;
        let mut frame_table = self.frame_table.borrow_mut();
        for i in 0..pages {
            if !frame_table.page_allocated((page_index + i)) {
                panic!(
                    "Page Fault - Free (PI: {}, CPC: {}, PC: {}, MA: 0x{:X}, SA: 0x{:X}, AO: 0x{:X})",
                    page_index,
                    i,
                    pages,
                    address,
                    self.start_address,
                    address - self.start_address
                );
            }

            frame_table.toggle_frame_alloc_status(page_index + i);
        }
    }
}

impl FrameAllocator<'_> {
    pub fn new(memory_map: &MemoryMap, page_size: u16) -> Self {
        let table_size = (memory_map
            .entries()
            .map(|desc| desc.page_count)
            .sum::<u64>()
            >> 3);
        let frame_table = unsafe { slice::from_raw_parts_mut(0x0001 as *mut _, table_size as usize) };
        frame_table.fill(0);

        let allocator = Self {
            start_address: table_size + 1,
            stop_address: {
                let last_descriptor = memory_map.entries().last().unwrap();
                last_descriptor.phys_start + (last_descriptor.page_count * 4096)
            },
            page_size,
            frame_table: RefCell::new(FrameTable { frame_table }),
        };

        allocator
    }

    pub fn reserve_memory_section(&mut self, descriptor: &MemoryDescriptor) {
        let pages = (descriptor.page_count * 4096) / self.page_size as u64;
        let start_page_index = descriptor.virt_start / 4096;

        for i in 0..pages {
            self.frame_table
                .borrow_mut()
                .toggle_frame_alloc_status((start_page_index + i) as usize);
        }
    }

    pub fn find_first_frame_index(&self, page_count: usize) -> Option<usize> {
        let frame_table = &self.frame_table.borrow().frame_table;

        for i in 0..frame_table.len() {
            let block = frame_table.get(i).unwrap();
            if block.count_ones() == 8 {
                continue;
            }

            let mut counter = 0;
            for bit in 0..8 {
                if *block & (1 << bit) == 0 {
                    counter += 1;
                } else {
                    counter = 0;
                }

                if counter >= page_count {
                    return Some(i * 8 + bit);
                }
            }
        }
        None
    }

    #[inline]
    pub fn available_frames(&self) -> usize {
        ((self.stop_address - self.start_address) / self.page_size as u64) as usize
    }

    #[inline]
    pub fn allocated_frames(&self) -> usize {
        let frame_table = &self.frame_table.borrow().frame_table;

        let mut count = 0;
        for i in 0..frame_table.len() {
            count += frame_table.get(i).unwrap().count_ones() as usize;
        }
        count
    }

    #[inline]
    pub fn remaining_frames(&self) -> usize {
        self.available_frames() - self.allocated_frames()
    }
}
