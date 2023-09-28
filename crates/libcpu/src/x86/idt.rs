//! This module implements the Interrupt Descriptor Table (IDT) in Rust into this library. The IDT
//! is used to wrap interrupt handlers to interrupts. Interrupts can be triggered by the programmer
//! with the `INT` instruction or by the CPU.
//!
//! Interrupts are used to handle critical situations in programs (a.e. exceptions like `Division
//! with zero`), so interrupts are interrupting the execution of the program.
//!
//! # See also
//! - [Interrupt Descriptor Table](https://wiki.osdev.org/IDT) on [OSDev.org](https://osdev.org)

use bitflags::{
    bitflags,
    Flags,
};

bitflags! {
    /// This structure represents most of the flags for the access byte in the descriptor.
    ///
    /// Here is a list of all flags with description:
    /// - [DescriptorAccess::PRESENT] - This bit must be always set to communicate the CPU, that
    /// this segment is valid.
    ///
    /// # See also
    /// - [Interrupt Descriptor Table](https://wiki.osdev.org/IDT) under Segment Descriptor/Access
    /// Byte
    #[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
    pub struct DescriptorAccess: u8 {
        /// This bit must be always set to communicate the CPU, that this segment is valid.
        const PRESENT = 0b1;
    }
}

/// This structure represents the type of the interrupt descriptor's gate.
///
/// Here is a list of all types with description:
/// - [GateType::Interrupt] - If this type is specified, the gate of the interrupt handler is a
/// interrupt gate
/// - [GateType::Trap] - f this type is specified, the gate of the interrupt handler is a trap gate
#[repr(u8)]
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub enum GateType {
    /// If this type is specified, the gate of the interrupt handler is a interrupt gate. An
    /// Interrupt Gate is a way to specify an Interrupt Service Routine (ISR). When an interrupt
    /// is triggered, the CPU uses the Interrupt Gate's Selector and Offset values to call the
    /// ISR. After executing the ISR, the CPU returns from the interrupt. In 32-bit mode with
    /// a 16-bit gate, the O32 IRET instruction is needed to ensure a proper return.
    Interrupt = 0b0000_1110,

    /// If this type is specified, the gate of the interrupt handler is a trap gate. A Trap Gate is
    /// used for handling Exceptions. When an exception occurs, there may be an error code placed on
    /// the stack, which should be removed before returning from the interrupt.
    Trap = 0b0000_1111,
}

/// InterruptDescriptor represents an entry in the Interrupt Descriptor Table (IDT). It is used to
/// define the handler for specific interrupts and exceptions while the code execution.
#[repr(C, packed)]
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub struct InterruptDescriptor {
    base_lo: u16,
    segment_selector: u16,
    always0: u8,
    flags: u8,
    base_hi: u16,
    padding: [u8; 8],
}

impl InterruptDescriptor {
    pub fn new(offset: u64, segment_selector: u16, gate_type: GateType) -> Self {
        Self {
            // TODO: Implement PrivilegeLevel
            base_lo: (offset & 0xFFFF) as u16,
            segment_selector,
            always0: 0,
            flags: (gate_type as u8) | DescriptorAccess::PRESENT.bits(),
            base_hi: ((offset >> 16) & 0xFFFF) as u16,
            padding: [0; 8],
        }
    }
}
