//! This module implements the function for the interaction with the hardware under x86_64 and x86
//! processors. Here are general features like [wait_for_interrupts].
//!
//! # See also
//! [x86_64](https://wiki.osdev.org/X86-64) by [OSDev](https://wiki.osdev.org)

pub mod cpuid;

use crate::MemoryAddress;
use bit_field::BitField;
use core::{
    arch::asm,
    fmt::{
        Display,
        Formatter,
    },
};

macro_rules! implement_control_features {
    ($(#[$attr:meta])* $access: vis enum $name: ident { $($feature: ident ($register: literal, $literal: literal) = $value: expr),* }) => {
        $access enum $name {
            $(
            $feature,
            )*
        }

        impl $name {
            pub fn all_enabled_features() -> alloc::vec::Vec<Self> {
                let mut feature = alloc::vec::Vec::new();
                $(
                if Self::$feature.enabled() {
                    feature.push(Self::$feature);
                }
                )*
                feature
            }

            pub fn enable(&self) {
                unimplemented!("This functionality is currently unimplemented");
            }

            pub fn disable(&self) {
                unimplemented!("This functionality is currently unimplemented");
            }

            pub fn enabled(&self) -> bool {
                let mut result = 0;
                unsafe {
                    match self {
                        $(
                        Self::$feature => {
                            asm!(concat!("mov {0}, ", $register), out(reg) result, options(nomem, nostack, preserves_flags));
                            if (result & $value) == $value { true } else { false }
                        }
                        )*
                    }
                }
            }
        }
    }
}

implement_control_features! {
    #[rustfmt::skip]
    pub enum CPUFeature {
        ProtectedMode("cr0", "Protected Mode enabled")                                = 0 << 0x00,
        MonitorCoProcessor("cr0", "Monitor Co-Processor")                             = 1 << 0x01,
        X87FPUEmulation ("cr0", "x67 FPU Emulation")                                  = 1 << 0x02,
        TaskSwitched("cr0", "Task switched")                                          = 1 << 0x03,
        ExtensionType("cr0", "Extension Type")                                        = 1 << 0x04,
        NumericError("cr0", "Numeric Error")                                          = 1 << 0x05,
        WriteProtect("cr0", "Write Protect")                                          = 1 << 0x06,
        AlignmentMask("cr0", "Alignment Mask")                                        = 1 << 0x07,
        NotWriteThrough("cr0", "Not-write through")                                   = 1 << 0x08,
        CacheDisable("cr0", "Cache Disable")                                          = 1 << 0x09,
        Paging("cr0", "Paging")                                                       = 1 << 0x0A,
        PageLevelWriteThrough("cr3", "Page-Level Write-Trough")                       = 1 << 0x02,
        PageLevelCacheDisable("cr3", "Page-Level Cache Disable")                      = 1 << 0x03,
        VME ("cr4", "Virtual 8086 Mode Extensions")                                   = 1 << 0x00,
        PVI("cr4", "Protected-mode Virtual Interrupts")                               = 1 << 0x01,
        TSD("cr4", "Time Stamp Disable")                                              = 1 << 0x02,
        DE("cr4", "Debugging Extensions")                                             = 1 << 0x03,
        PSE("cr4", "Page Size Extensions")                                            = 1 << 0x04,
        PAE("cr4", "Physical Address Extension")                                      = 1 << 0x05,
        MCE("cr4", "Machine Check Exception")                                         = 1 << 0x06,
        PGE("cr4", "Page Global Enabled")                                             = 1 << 0x07,
        PCE("cr4", "Performance-Monitoring Counter")                                  = 1 << 0x08,
        OSFXSR("cr4", "OS supports FXSAVE AND FXRSTOR")                               = 1 << 0x09,
        OSXMMEXCEPT("cr4", "OS supports unmasked SIMD Floating-Point Exceptions")     = 1 << 0x0A,
        UMIP("cr4", "User Mode Instruction Prevention")                               = 1 << 0x0B,
        VMXE("cr4", "Virtual Machine Extensions enabled")                             = 1 << 0x0C,
        SMXE("cr4", "Safer Mode Extensions enabled")                                  = 1 << 0x0D,
        FSGSBASE("cr4", "RDFSBASE, RDGSBASE, WRFSBASE, and WRGSBASE enabled")         = 1 << 0x10,
        PCIDE("cr4", "PCID enabled")                                                  = 1 << 0x11,
        OSXSAVE("cr4", "XSAVE and Processor Extended States enabled")                 = 1 << 0x12,
        SMEP("cr4", "Supervisor Mode Execution Prevention")                           = 1 << 0x14,
        SMAP("cr4", "Supervisor Mode Access Prevention")                              = 1 << 0x15,
        PKE("cr4", "Protection Key enabled")                                          = 1 << 0x16,
        CET("cr4", "Intel Control-Flow Enforcement Technology (CET)")                 = 1 << 0x17,
        PKS("cr4", "Enable Protection Keys for Supervisor-Mode Pages")                = 1 << 0x18
        //X87("xcr0", "x87 FPU/MMX Support (must be 1)")                                = 1 << 0x00,
        //SSE("xcr0", "XSAVE Support for MXCSR and XMM registers")                      = 1 << 0x01,
        //AVX("xcr0", "AVX enabled and XSAVE support for upper YMM register data")      = 1 << 0x02,
        //BNDREG("xcr0", "MPX enabld and XSAVE support for BND0-BND3 registers")        = 1 << 0x03,
        //BNDCSR("xcr0", "MPX enabled and XSAVE suport for BNDCFGU and BNDSTATUS")      = 1 << 0x04,
        //OPMASK("xcr0", "AVX-512 enabled and XSAVE support for opmask registers")      = 1 << 0x05,
        //ZMM_HI256("xcr0", "AVX-512 enabled and XSAVE support for lower ZMM reg data") = 1 << 0x06,
        //HI16_ZMM("xcr0", "AVS-512 enabled and XSAVE support for upper ZMM data")      = 1 << 0x07,
        //PKRU("xcr0", "XSAVE Support for PKRU Register")                               = 1 << 0x09
    }
}

/// This structure represents the privilege level for the descriptor. x86 and x86_64 CPUs are
/// providing a few rings, but only 2 are used in Production-ready operating systems.
///
/// Here is a short explanation of all privilege levels:
/// - [PrivilegeLevel::KernelSpace] - This is the ring for the Kernel mode. Least protection and
/// maximal access to hardware resources. A bootloader or kernel uses that mode.
/// - [PrivilegeLevel::Ring1] - This is a ring for device drivers. It offers more protection, but
/// not the level of protection as Ring 3. (This ring is not used by almost all operating systems)
/// - [PrivilegeLevel::Ring2] - This is a ring for device drivers. It offers more protection, but
/// not the level of protection as Ring 3. It's the same like Ring 2. (This ring is not used by
/// almost all operating systems)
/// - [PrivilegeLevel::UserSpace] - This is the Userspace/Userland ring. This ring ist used by the
/// most operating systems for running applications. This ring grant the least privileges but the
/// highest protection by the hardware. The communication with the hardware resources is handled
/// over the kernel with System Calls.
///
/// # See also
/// - [CPU Security Rings](https://wiki.osdev.org/Security#Rings) by [OSDev.org](https://wiki.osdev.org/)
/// - [Global Descriptor Table](https://wiki.osdev.org/Global_Descriptor_Table#Segment_Descriptor)
/// by [OSDev.org](https://wiki.osdev.org/)
/// - [Protection Ring](https://en.wikipedia.org/wiki/Protection_ring) by
/// [Wikipedia](https://wikipedia.org)
#[repr(u8)]
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub enum PrivilegeLevel {
    /// This is the ring for the Kernel mode. Least protection and maximal access to hardware
    /// resources. A bootloader or kernel uses that mode.
    ///
    /// # See also
    /// - [Ring 0](https://wiki.osdev.org/Security#Ring_0) by [OSDev.org](https://wiki.osdev.org/)
    KernelSpace = 0b0000_0000,

    /// This is a ring for device drivers. It offers more protection, but not the level of
    /// protection as Ring 3. (This ring is not used by almost all operating systems)
    ///
    /// # See also
    /// - [Rings 1 and 2](https://wiki.osdev.org/Security#Rings_1_and_2) by [OSDev.org](https://wiki.osdev.org/)
    Ring1 = 0b0010_0000,

    /// This is a ring for device drivers. It offers more protection, but not the level of
    /// protection as Ring 3. It's the same like Ring 2. (This ring is not used by almost all
    /// operating systems)
    ///
    /// # See also
    /// - [Rings 1 and 2](https://wiki.osdev.org/Security#Rings_1_and_2) by [OSDev.org](https://wiki.osdev.org/)
    Ring2 = 0b0100_0000,

    /// This is the Userspace/Userland ring. This ring ist used by the most operating systems for
    /// running applications. This ring grant the least privileges but the highest protection by
    /// the hardware. The communication with the hardware resources is handled over the kernel
    /// with System Calls.
    ///
    /// # See also
    /// - [Ring 3](https://wiki.osdev.org/Security#Ring_3) by [OSDev.org](https://wiki.osdev.org/)
    UserSpace = 0b0110_0000,
}

/// This code just implements the Display trait into the privilege level over the Debug trait.
impl Display for PrivilegeLevel {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{:?}", self)
    }
}

/// This implementation is used to convert a bits into the privilege level enum. This function is
/// used in [SegmentSelector::privilege_level]
impl From<u16> for PrivilegeLevel {
    #[must_use]
    fn from(value: u16) -> Self {
        match value {
            0x0 => Self::KernelSpace,
            0x1 => Self::Ring1,
            0x2 => Self::Ring2,
            0x4 => Self::UserSpace,
            _ => panic!("Invalid privilege level {}", value),
        }
    }
}

/// This enum identifies which descriptor table is used in the [SegmentSelector]. You can choose
/// between the [GDT](https://wiki.osdev.org/Global_Descriptor_Table) and the
/// [LDT](https://wiki.osdev.org/Local_Descriptor_Table)
///
/// - [DescriptorTable::GDT] - This value identifies the Global Descriptor Table (GDT)
/// - [DescriptorTable::LDT] - This value identifies the Local Descriptor Table (LDT)
///
/// # See also
/// - [Global Descriptor Table](https://wiki.osdev.org/Global_Descriptor_Table) by
/// [OSDev.org](https://wiki.osdev.org/)
/// - [Local Descriptor Table](https://wiki.osdev.org/Local_Descriptor_Table) by
/// [OSDev.org](https://wiki.osdev.org/)
/// - [Segment Selector](https://wiki.osdev.org/Segment_Selector) by
/// [OSDev.org](https://wiki.osdev.org/)
#[repr(u8)]
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
pub enum DescriptorTable {
    /// This value identifies the Global Descriptor Table (GDT)
    ///
    /// # See also
    /// - [Global Descriptor Table](https://wiki.osdev.org/Global_Descriptor_Table) by
    /// [OSDev.org](https://wiki.osdev.org/)
    #[default]
    GDT = 0b0000,

    /// This value identifies the Local Descriptor Table (LDT)
    ///
    /// # See also
    /// - [Local Descriptor Table](https://wiki.osdev.org/Local_Descriptor_Table) by
    /// [OSDev.org](https://wiki.osdev.org/)
    LDT = 0b1000,
}

/// This code just implements the Display trait into the descriptor table over the Debug trait.
impl Display for DescriptorTable {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{:?}", self)
    }
}

/// This implementation is used to convert a bit into to the table enum. This function is used in
/// [SegmentSelector::table]
impl From<bool> for DescriptorTable {
    #[must_use]
    fn from(value: bool) -> Self {
        match value {
            false => Self::GDT,
            true => Self::LDT,
        }
    }
}

/// This implementation is used to convert the table enum into a bit. This function is used in
/// [SegmentSelector::set_table]
impl From<DescriptorTable> for bool {
    #[must_use]
    fn from(value: DescriptorTable) -> Self {
        match value {
            DescriptorTable::GDT => false,
            DescriptorTable::LDT => true,
        }
    }
}

/// This structure represents the x86_64 and IA-32 specific Segment Selector data structure with a
/// size of 16 bit. It's used in Protected and Long mode. The value of the descriptor identified a
/// segment in the [LDT](https://wiki.osdev.org/Local_Descriptor_Table) or
/// [GDT](https://wiki.osdev.org/Global_Descriptor_Table).
///
/// # See also
/// - [Segment Selector](https://wiki.osdev.org/Segment_Selector) by
/// [OSDev.org](https://wiki.osdev.org/)
#[repr(transparent)]
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    /// This function creates the segment selector by the single values of the entry index, type of
    /// the table and the privilege level of the selector.
    ///
    /// - `index` - The index of the entry in the table (Last 3 bits are moved away)
    /// - `table` - The type of the descriptor table in that the entry is located
    /// - `privilege` - The requested privilege level. This value determines if the selector is
    /// valid during permission checks
    ///
    /// - [Segment Selector](https://wiki.osdev.org/Segment_Selector) by
    /// [OSDev.org](https://wiki.osdev.org/)
    #[inline]
    #[must_use]
    pub fn new(index: u16, table: DescriptorTable, privilege: PrivilegeLevel) -> Self {
        Self((index << 3) | (table as u16) | (privilege as u16 >> 5))
    }

    /// This function replaces the privilege level with the function-specific privilege level.
    ///
    /// # See also
    /// - [CPU Security Rings](https://wiki.osdev.org/Security#Rings) by
    /// [OSDev.org](https://wiki.osdev.org/)
    /// - [Global Descriptor Table](https://wiki.osdev.org/Global_Descriptor_Table#Segment_Descriptor)
    /// by [OSDev.org](https://wiki.osdev.org/)
    /// - [Protection Ring](https://en.wikipedia.org/wiki/Protection_ring) by
    /// [Wikipedia](https://wikipedia.org)
    /// - [PrivilegeLevel] (Source Code)
    #[inline]
    pub fn set_privilege_level(&mut self, level: PrivilegeLevel) {
        self.0.set_bits(0..2, level as u16);
    }

    /// This function returns the privilege level, set by the creator of this selector.
    ///
    /// # See also
    /// - [CPU Security Rings](https://wiki.osdev.org/Security#Rings) by
    /// [OSDev.org](https://wiki.osdev.org/)
    /// - [Global Descriptor Table](https://wiki.osdev.org/Global_Descriptor_Table#Segment_Descriptor)
    /// by [OSDev.org](https://wiki.osdev.org/)
    /// - [Protection Ring](https://en.wikipedia.org/wiki/Protection_ring) by
    /// [Wikipedia](https://wikipedia.org)
    /// - [PrivilegeLevel] (Source Code)
    #[inline]
    #[must_use]
    pub fn privilege_level(&self) -> PrivilegeLevel {
        PrivilegeLevel::from(self.0.get_bits(0..2))
    }

    /// This function replaces the descriptor table with the function-specific descriptor table.
    ///
    /// # See also
    /// - [Segment Selector](https://wiki.osdev.org/Segment_Selector) by
    /// [OSDev.org](https://wiki.osdev.org/)
    /// - [DescriptorTable] (Source Code)
    #[inline]
    pub fn set_table(&mut self, descriptor_table: DescriptorTable) {
        self.0.set_bit(2, descriptor_table.into());
    }

    /// This function returns the descriptor set, set by the creator of this selector.
    ///
    /// # See also
    /// - [Segment Selector](https://wiki.osdev.org/Segment_Selector) by
    /// [OSDev.org](https://wiki.osdev.org/)
    /// - [DescriptorTable] (Source Code)
    #[inline]
    #[must_use]
    pub fn table(&self) -> DescriptorTable {
        DescriptorTable::from(self.0.get_bit(2))
    }

    /// This function replaces the descriptor index with the function-specific descriptor index.
    #[inline]
    pub fn set_index(&mut self, index: u16) {
        self.0.set_bits(0..16, index);
    }

    /// This function returns the descriptor index, set by the creator of this selector.
    #[inline]
    #[must_use]
    pub fn index(&self) -> u16 {
        self.0 >> 3
    }
}

/// This structure represents the pointer structure to a descriptor table like the GDT or IDT. These
/// pointers are used in the `lidt` and `lgdt` instruction.
///
/// - `base` - This field represents the linear base address (not the physical address) to the table
/// (size is depending on the target processor architecture)
/// - `size` - This field represents the size of the table in bytes. (subtracted by 1)
///
/// # See also
/// - [Global Descriptor Table](https://wiki.osdev.org/Global_Descriptor_Table#GDTR) by
/// [OSDev.org](https://wiki.osdev.org)
/// - [Interrupt Descriptor Table](https://wiki.osdev.org/Interrupt_Descriptor_Table#IDTR) by
/// [OSDev.org](https://wiki.osdev.org)
/// - [gdt::GlobalDescriptorTable::as_ptr()] (Source Code)
#[repr(C, packed)]
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub struct DescriptorTablePointer {
    /// This field represents the linear base address (not the physical address) to the table
    /// (size is depending on the target processor architecture)
    ///
    /// # See also
    /// - [Global Descriptor Table](https://wiki.osdev.org/Global_Descriptor_Table#GDTR) by
    /// [OSDev.org](https://wiki.osdev.org)
    /// - [Interrupt Descriptor Table](https://wiki.osdev.org/Interrupt_Descriptor_Table#IDTR) by
    /// [OSDev.org](https://wiki.osdev.org)
    pub base: MemoryAddress,

    /// This field represents the size of the table in bytes. (subtracted by 1)
    ///
    /// # See also
    /// - [Global Descriptor Table](https://wiki.osdev.org/Global_Descriptor_Table#GDTR) by
    /// [OSDev.org](https://wiki.osdev.org)
    /// - [Interrupt Descriptor Table](https://wiki.osdev.org/Interrupt_Descriptor_Table#IDTR) by
    /// [OSDev.org](https://wiki.osdev.org)
    pub size: u16,
}

/// This function implements the halt (`hlt`) instruction as a platform-independent function. The
/// halt instruction halts the CPU until the next external interrupt is triggered.
///
/// # See also
/// - [HLT (x86 instruction)](https://en.wikipedia.org/wiki/HLT_(x86_instruction)) by
/// [Wikipedia](https://en.wikipedia.org)
/// - [HLT - HALT](https://www.felixcloutier.com/x86/hlt) by
/// [Felix Cloutier](https://www.felixcloutier.com/)
#[inline(always)]
pub fn wait_for_interrupts() {
    unsafe { asm!("hlt", options(noreturn)) }
}
