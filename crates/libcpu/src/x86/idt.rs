//! This module implements the functionality of the x86_64 and IA-32 specific Interrupt Descriptor
//! Table. The IDT is storing the references to the Interrupt Service Routines for the CPU. These
//! interrupts are triggered by the CPU itself or though `INT` instructions.
//!
//! A single IDT descriptor contains address of the handler function, the [SegmentSelector]
//! and the descriptor's flag. The following structure shows how a single descriptor is represented
//! in the memory:
//! ```text
//! 0                   1                   2                   3                   4
//! 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |     Lower Handler Function Address    |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |            Segment Selector           |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |     Reserved      |       Flags       |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |    Higher Handler Function Address    |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |                                    Padding                                    |
//! |                                                                               |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! ```
//!
//! Interrupts are used to handle critical parts of the program execution like Division trough zero
//! or other recoverable and unrecoverable issues. If a interrupt is triggered, the processor saves
//! the state of the execution, searches a descriptor in the IDT and then calls the handler, that
//! was specified. So with this, we can handle CPU-provided exceptions or other user-defined
//! interrupts.
//!
//! Also other events like Keyboard Interaction are interrupts and get handled by the interrupt
//! descriptor's specified handlers. The keyboard uses the Programmable Interrupt Controller (PIC)
//! for that.
//!
//! # See also
//! - [Interrupt Descriptor Table](https://wiki.osdev.org/Interrupt_Descriptor_Table) by
//! [OSDev.org](https://wiki.osdev.org/)
//! - [Exceptions](https://wiki.osdev.org/Exceptions) by [OSDev.org](https://wiki.osdev.org/)
//! - [Interrupt](https://wiki.osdev.org/Interrupt) by [OSDev.org](https://wiki.osdev.org/)
//! - [x86 Handling Exceptions](https://hackernoon.com/x86-handling-exceptions-lds3uxc) by
//! [HackerNoon.com](https://hackernoon.com/)

use crate::SegmentSelector;

/// This structure implements a single descriptor in the IDT (Interrupt Descriptor Table). This
/// structure is compatible with the raw memory representation of a descriptor. The implementation
/// of the IDT is only needed for IA-32 and x86_64 architectures.
///
/// - `lower_isr_address` - This field represents the first 16 bits of the ISR function.
/// - `segment_selector` - This field represents the segment selector which must point to a valid
/// code segment in the GDT.
/// - `reserved` - This field is always zero and should not be set by the user.
/// - `flags` - This field represents the flags of the descriptor.
/// - `higher_isr_address` - This field represents the last 16 bits of the ISR function.
///
/// # See also
/// - [Interrupt Descriptor Table](https://wiki.osdev.org/Interrupt_Descriptor_Table#Structure_on_IA-32)
/// by [OSDev.org](https://wiki.osdev.org/)
/// - [Interrupt Descriptor Table](https://wiki.osdev.org/Interrupt_Descriptor_Table#Structure_on_x86-64)
/// by [OSDev.org](https://wiki.osdev.org/)
#[repr(C, packed)]
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
pub struct IDTDescriptor {
    /// This field represents the first 16 bits of the ISR function
    lower_isr_address: u16,

    /// This field represents the segment selector which must point to a valid
    /// code segment in the GDT
    segment_selector: SegmentSelector,

    /// This field is always zero and should not be set by the user
    reserved: u8,

    /// This field represents the flags of the descriptor
    flags: u8,

    /// This field represents the last 16 bits of the ISR function
    higher_isr_address: u16,
    #[cfg(target_arch = "x86_64")]
    padding: [u8; 8]
}