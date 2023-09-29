//! This module implements the functionality of the x86_64 and IA-32 specific Interrupt Descriptor
//! Table. The IDT is storing the references to the Interrupt Service Routines for the CPU. These
//! interrupts are triggered by the CPU itself or though `INT` instructions.
//!
//! A single IDT descriptor contains address of the handler function, the [x86::SegmentSelector]
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

use crate::x86;
