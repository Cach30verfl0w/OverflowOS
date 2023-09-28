//! This module implements the x86/x86_64 specific functionality as a Rust "Wrapper" of the Global
//! Descriptor Table (GDT). The GDT is used to configure memory areas.
//! # See also
//! - [x86 Handling Exceptions](https://hackernoon.com/x86-handling-exceptions-lds3uxc)
//! - [osdev Global Descriptor Table](https://wiki.osdev.org/Global_Descriptor_Table)

use bitflags::{
    bitflags,
    Flags,
};
use core::{
    arch::asm,
    mem::size_of,
};

/// This structure represents the privilege level for the descriptor. The CPU supports a few
/// privilege levels.
///
/// Here is a short explanation of all privilege levels:
/// - [PrivilegeLevel::Ring0] - This is the ring for the Kernel mode. Least protection and maximal
/// access to hardware resources. A bootloader or kernel uses that mode.
/// - [PrivilegeLevel::Ring1] - This is a ring for device drivers. It offers more protection, but
/// not the level of protection as Ring 3. (This ring is not used by almost all operating systems)
/// - [PrivilegeLevel::Ring2] - This is a ring for device drivers. It offers more protection, but
/// not the level of protection as Ring 3. It's the same like Ring 2. (This ring is not used by
/// almost all operating systems)
/// - [PrivilegeLevel::Ring3] - This is the Userspace/Userland ring. This ring ist used by the most
/// operating systems for running applications. This ring grant the least privileges but the highest
/// protection by the hardware. The communication with the hardware resources is handled over the
/// kernel with System Calls.
///
/// # See also
/// - [CPU Security Rings](https://wiki.osdev.org/Security#Rings) on [OSDev.org](https://wiki.osdev.org/)
/// - [Global Descriptor Table](https://wiki.osdev.org/Global_Descriptor_Table) on
/// [OSDev.org](https://wiki.osdev.org/) under Segment Descriptor/Access Byte
#[repr(u8)]
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub enum PrivilegeLevel {
    /// This is the ring for the Kernel mode. Least protection and maximal access to hardware
    /// resources. A bootloader or kernel uses that mode.
    Ring0 = 0b0000_0000,

    /// This is a ring for device drivers. It offers more protection, but not the level of
    /// protection as Ring 3. (This ring is not used by almost all operating systems)
    Ring1 = 0b0010_0000,

    /// This is a ring for device drivers. It offers more protection, but not the level of
    /// protection as Ring 3. It's the same like Ring 2. (This ring is not used by almost all
    /// operating systems)
    Ring2 = 0b0100_0000,

    /// This is the Userspace/Userland ring. This ring ist used by the most operating systems for
    /// running applications. This ring grant the least privileges but the highest protection by
    /// the hardware. The communication with the hardware resources is handled over the kernel
    /// with System Calls.
    Ring3 = 0b0110_0000,
}

bitflags! {
    /// This structure represents most of the flags for the access byte in the descriptor.
    ///
    /// Here is a list of all flags with description:
    /// - [DescriptorAccess::ACCESSED] - This bit is set by the CPU when the CPU accesses the
    /// descriptor. If the descriptor is stored in read only pages and this bit is set to 0, the
    /// CPU will trigger a page fault. You should set this bit.
    /// - [DescriptorAccess::PRESENT] - This bit must be always set to communicate the CPU, that
    /// this segment is valid.
    /// - [DescriptorAccess::USER_SEGMENT] - If set, the segment is a code or data segment. If not,
    /// this segment is a data segment (a.e. a Task State Segment). This flag
    /// - [DescriptorAccess::EXECUTABLE] - If defined, the segment is a executable code segment. If
    /// not, this segment is a data segment
    /// - [DescriptorAccess::READABLE] - This bit is only for code segments. If set, read access to
    /// the code segment is allowed. Write access is never allowed for these segments.
    /// - [DescriptorAccess::WRITABLE] - This bit is only for data segments. If set, write access to
    /// the data segment is allowed. Read access is always allowed for these segments.
    ///
    /// # See also
    /// - [Global Descriptor Table](https://wiki.osdev.org/Global_Descriptor_Table) under
    /// Segment Descriptor/Access Byte
    #[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
    pub struct DescriptorAccess: u8 {
        /// This bit is set by the CPU when the CPU accesses the descriptor. If the descriptor
        /// is stored in read only pages and this bit is set to 0, the CPU will trigger a page
        /// fault. You should set this bit.
        const ACCESSED     = 0b0000_0001;

        /// This bit must be always set to communicate the CPU, that this segment is valid.
        const PRESENT      = 0b1000_0000;

        /// If set, the segment is a code or data segment. If not, this segment is a data
        /// segment (a.e. a Task State Segment). This flag
        const USER_SEGMENT = 0b0001_0000;

        /// If defined, the segment is a executable code segment. If not, this segment is a data
        /// segment.
        const EXECUTABLE   = 0b0000_1000;

        /// This bit is only for code segments. If set, read access to the code segment is
        /// allowed. Write access is never allowed for these segments.
        const READABLE     = 0b0000_0010;

        /// This bit is only for data segments. If set, write access to the data segment is
        /// allowed. Read access is always allowed for these segments.
        const WRITABLE     = 0b0000_0010;
    }
}

bitflags! {
    /// This structure represents the flags, that can be set on a descriptor.
    ///
    /// Here is a list of all flags with description:
    /// - [DescriptorFlags::GRANULARITY] - This flag indicates the scaling of the Limit value. If
    /// this flag is set, the limit is in 4 KiB blocks. If not, the Limit value is in 1 byte blocks.
    /// - [DescriptorFlags::SIZE] - If this flag is set, this is a 32-bit protected mode segment. If
    /// not set, this is a 16-bit protected mode segment.
    /// - [DescriptorFlags::LONG_MODE] - If this flag iet set, this is a 64-bit code segment. If
    /// this is set, you shouldn't set the size flag.
    ///
    /// # See also
    /// - [Global Descriptor Table](https://wiki.osdev.org/Global_Descriptor_Table) under Segment
    /// Descriptor/DescriptorAccess Byte
    #[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
    pub struct DescriptorFlags: u8 {
        /// This flag indicates the scaling of the Limit value. If this flag is set, the limit
        /// is in 4 KiB blocks. If not, the Limit value is in 1 byte blocks.
        const GRANULARITY = 0b1000;

        /// If this flag is set, this is a 32-bit protected mode segment. If not set, this is a
        /// 16-bit protected mode segment.
        const SIZE        = 0b0100;

        /// If this flag iet set, this is a 64-bit code segment. If this is set, you shouldn't set
        /// the size flag.
        const LONG_MODE   = 0b0010;
    }
}

/// This structure represents a single descriptor in the GDT (Global Descriptor Table). This
/// structure is compatible with the raw memory representation of a descriptor. Use the function
/// [`GDTDescriptor::default`] to generate the Null descriptor. The implementation of the GDT is
/// only needed for IA-32 and x86_64/x86 architectures.
///
/// # See also
/// - [Global Descriptor Table](https://wiki.osdev.org/Global_Descriptor_Table) under Segment
/// Descriptor
#[repr(C, packed)]
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
pub struct GDTDescriptor {
    _ignored1: [u8; 5], /* This section is ignored on x86_64 systems TODO: Implement for 32-bit
                         * compatibility */
    access: u8,
    flags: u8,
    _ignored2: [u8; 2], /* This section is ignored on x86_64 systems TODO: Implement for 32-bit
                         * compatibility */
}

impl GDTDescriptor {
    /// This function creates a new GDT descriptor with the specified values. The function parameters
    /// `privilege`, `kind` and `access` are merged to the access byte for the descriptor.
    ///
    /// Here is a list with the parameters:
    /// - `privilege` - This parameter defines the privilege level of the descriptor
    /// - `access` - This parameter defines the access flags of the descriptor
    /// - `flag` - This parameter defines the flags of the descriptor
    ///
    /// # See also
    /// - [GDT Tutorial](https://wiki.osdev.org/GDT_Tutorial) under `What to put in a GDT`
    pub fn new(privilege: PrivilegeLevel, access: DescriptorAccess, flags: DescriptorFlags) -> Self {
        let mut descriptor = GDTDescriptor::default();
        descriptor.access = access.bits() | (privilege as u8);
        descriptor.flags = flags.bits();
        descriptor
    }

    /// This function creates a new GDT descriptor with the default settings for a executable
    /// Kernel-Mode Code segment
    ///
    /// # See also
    /// - [GDT Tutorial](https://wiki.osdev.org/GDT_Tutorial) under `What to put in a GDT`
    #[inline]
    pub fn kernel_mode_code_segment() -> Self {
        Self::new(
            PrivilegeLevel::Ring0,
            DescriptorAccess::PRESENT | DescriptorAccess::READABLE | DescriptorAccess::EXECUTABLE,
            DescriptorFlags::GRANULARITY | DescriptorFlags::LONG_MODE,
        )
    }

    /// This function creates a new GDT descriptor with the default settings for a
    /// Kernel-Mode Data segment
    ///
    /// # See also
    /// - [GDT Tutorial](https://wiki.osdev.org/GDT_Tutorial) under `What to put in a GDT`
    #[inline]
    pub fn kernel_mode_data_segment() -> Self {
        Self::new(
            PrivilegeLevel::Ring0,
            DescriptorAccess::PRESENT | DescriptorAccess::WRITABLE,
            DescriptorFlags::GRANULARITY | DescriptorFlags::LONG_MODE,
        )
    }

    /// This function creates a new GDT descriptor with the default settings for a
    /// User-Mode Code segment
    ///
    /// # See also
    /// - [GDT Tutorial](https://wiki.osdev.org/GDT_Tutorial) under `What to put in a GDT`
    #[inline]
    pub fn user_mode_code_segment() -> Self {
        Self::new(
            PrivilegeLevel::Ring3,
            DescriptorAccess::PRESENT | DescriptorAccess::READABLE | DescriptorAccess::EXECUTABLE,
            DescriptorFlags::GRANULARITY | DescriptorFlags::LONG_MODE,
        )
    }

    /// This function creates a new GDT descriptor with the default settings for a
    /// User-Mode Data segment
    ///
    /// # See also
    /// - [GDT Tutorial](https://wiki.osdev.org/GDT_Tutorial) under `What to put in a GDT`
    #[inline]
    pub fn user_mode_data_segment() -> Self {
        Self::new(
            PrivilegeLevel::Ring3,
            DescriptorAccess::PRESENT | DescriptorAccess::WRITABLE,
            DescriptorFlags::GRANULARITY | DescriptorFlags::LONG_MODE,
        )
    }
}

#[repr(C, packed(2))]
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub struct GlobalDescriptorTablePointer {
    limit: u16,
    base: u64,
}

/// This structure represents the Global Descriptor Table with the maximum of 8192 entries. In this
/// structure, we store the descriptors in a slice.
///
/// # See also
/// - [Global Descriptor Table](https://wiki.osdev.org/Global_Descriptor_Table)
/// - [GDT Tutorial](https://wiki.osdev.org/GDT_Tutorial)
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub struct GlobalDescriptorTable {
    descriptors: [GDTDescriptor; 8192],
    count: usize,
}

impl GlobalDescriptorTable {
    pub fn new() -> Self {
        Self {
            descriptors: [GDTDescriptor::default(); 8192],
            count: 0,
        }
    }

    pub fn load(&self) {
        unsafe {
            asm!("lgdt [{}]", in(reg) &self.as_ptr(), options(readonly, nostack, preserves_flags));
        }
    }

    pub fn insert(&mut self, index: usize, descriptor: GDTDescriptor) {
        self.descriptors[index] = descriptor;
        if self.count < index {
            self.count = index;
        }
    }

    pub fn as_ptr(&self) -> GlobalDescriptorTablePointer {
        GlobalDescriptorTablePointer {
            base: self.descriptors.as_ptr() as u64,
            limit: (self.count * size_of::<GDTDescriptor>() - 1) as u16,
        }
    }
}
