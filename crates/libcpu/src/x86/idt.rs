//! This module implements the Interrupt Descriptor Table (IDT) in Rust into this library. The IDT
//! is used to wrap interrupt handlers to interrupts. Interrupts can be triggered by the programmer
//! with the `INT` instruction or by the CPU.
//!
//! Interrupts are used to handle critical situations in programs (a.e. exceptions like `Division
//! with zero`), so interrupts are interrupting the execution of the program.
//!
//! # See also
//! - [Interrupt Descriptor Table](https://wiki.osdev.org/IDT) on [OSDev.org](https://osdev.org)

use crate::gdt::PrivilegeLevel;
use bitflags::{
    bitflags,
    Flags,
};
use core::{
    marker::PhantomData,
    mem,
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

// TODO: Implement High-level API for Exception and Interrupt Handling

/// InterruptDescriptor represents an entry in the Interrupt Descriptor Table (IDT). It is used to
/// define the handler for specific interrupts and exceptions while the code execution.
#[repr(C, packed)]
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
pub struct InterruptDescriptor<F> {
    isr_lower_address: u16,
    segment_selector: u16,
    always0: u8,
    flags: u8,
    isr_mid_address: u16,
    isr_higher_address: u32,
    reserved: u32,
    _phantom: PhantomData<F>,
}

impl<F> InterruptDescriptor<F> {
    pub fn new(
        privilege_level: PrivilegeLevel, handler: &F, access: DescriptorAccess,
        segment_selector: u16, gate_type: GateType,
    ) -> Self {
        let address = (unsafe { mem::transmute::<&F, *const F>(handler) }) as u16;
        Self {
            isr_lower_address: address & 0xFFFF,
            segment_selector,
            always0: 0,
            flags: gate_type.bits() & access.bits() & (privilege_level as u8),
            isr_mid_address: ((address >> 16) as u64 & 0xFFFF) as u16,
            isr_higher_address: ((address >> 32) as u64 & 0xFFFF_FFFF) as u32,
            reserved: 0,
            _phantom: Default::default(),
        }
    }
}
