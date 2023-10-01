//! This module implements the functionality of the x86_64 and IA-32 specific Interrupt Descriptor
//! Table. The IDT is storing the references to the Interrupt Service Routines for the CPU. These
//! interrupts are triggered by the CPU itself or though `INT` instructions.
//!
//! A single IDT descriptor contains address of the handler function, the [SegmentSelector]
//! and the descriptor's flag.
//!
//! The following structure shows how a single descriptor is represented in the memory (x86):
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
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! ```
//!
//! The following structure shows how a single descriptor is represented in the memory (x86_64):
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
//! |    Middle Handler Function Address    |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |                         Higher Handler Function Address                       |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |                                Reserved bytes                                 |
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

use crate::{
    halt_cpu,
    DescriptorTable,
    DescriptorTablePointer,
    MemoryAddress,
    PrivilegeLevel,
    SegmentSelector,
};
use core::{
    arch::asm,
    mem::size_of,
};

extern "x86-interrupt" fn default_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    halt_cpu();
}

/// This enum describes the types of gates that interrupt descriptors are able to represent. I don't
/// included the Task Gate, because that Gate can lead into a GP exception or is poorly optimized or
/// entirely removed. ([Resource](https://wiki.osdev.org/Interrupt_Descriptor_Table#Task_Gate))
///
/// - [GateType::Interrupt] - Interrupt gates are used to specify an ISR. Interrupt gates are
/// automatically deactivating interrupts and reactivating them upon a `iret`.
/// - [GateType::Trap] - Trap gates are used to handle exception. Gate types are not automatically
/// deactivating and reactivating interrupts.
///
/// # See also
/// - [Interrupt Service Routines](https://wiki.osdev.org/Interrupt_Service_Routines) by
/// [OSDev.org](https://wiki.osdev.org/)
#[repr(u8)]
pub enum GateType {
    /// Interrupt gates are used to specify an ISR. Interrupt gates are automatically deactivating
    /// interrupts and reactivating them upon a `iret`.
    Interrupt = 0xE,

    /// Trap gates are used to handle exception. Gate types are not automatically deactivating and
    /// reactivating interrupts.
    Trap = 0xF,
}

// TODO: Add more documentation and references to InterruptStackFrame

/// This structure represents the interrupt stack frame that the CPU pushed on interrupt or
/// exception. We can use that for state recovery.
#[repr(C)]
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub struct InterruptStackFrame {
    pub instruction_pointer: MemoryAddress,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: MemoryAddress,
    pub stack_segment: u64,
}

impl InterruptStackFrame {
    /// This function calls the iretq instruction to recover the CPU state
    #[inline(always)]
    #[cfg(target_arch = "x86_64")]
    pub unsafe fn recover(&self) -> ! {
        asm!(
            r#"
            push {stack_segment}
            push {stack_pointer}
            push {cpu_flags}
            push {code_segment}
            push {instruction_pointer}
            iretq
            "#,
            cpu_flags = in(reg) self.cpu_flags,
            instruction_pointer = in(reg) self.instruction_pointer,
            stack_pointer = in(reg) self.stack_pointer,
            code_segment = in(reg) self.code_segment,
            stack_segment = in(reg) self.stack_segment,
            options(noreturn)
        )
    }

    /// This function calls the iret instruction to recover the CPU state
    #[inline(always)]
    #[cfg(target_arch = "x86")]
    pub unsafe fn recover(&self) -> ! {
        asm!(
            r#"
            push {stack_segment}
            push {stack_pointer}
            push {cpu_flags}
            push {code_segment}
            push {instruction_pointer}
            iret
            "#,
            cpu_flags = in(reg) self.cpu_flags,
            instruction_pointer = in(reg) self.instruction_pointer,
            stack_pointer = in(reg) self.stack_pointer,
            code_segment = in(reg) self.code_segment,
            stack_segment = in(reg) self.stack_segment,
            options(noreturn)
        )
    }
}

/// This structure represents all available vector indexes of exceptions, provided by the
/// architecture/CPU. We didn't included the exceptions 'Coprocessor Segment Overrun' and
/// 'FPU Error Interrupt' because they seems to be deprecated.
///
/// Here is a list with all supported exceptions and they explanations:
/// - [Exception::Division] (Fault) - This exception occurs when the running code is diving any
/// number by 0 or the division result is to long to be representable.
///
/// - [Exception::Debug] (Fault/Trap) - This exception occurs under different conditions. If the
/// exception is a fault, the instruction pointer points to the exception-causing instruction.
/// Otherwise the save instruction pointer points to instruction after the exception-causing
/// instruction.
///
/// - [Exception::NonMaskableInterrupt] (Interrupt) - This interrupt occurs for unrecoverable
/// hardware problems. Newer computers are handling these thins over machine check exceptions etc.
///
/// - [Exception::Breakpoint] (Trap) - This exception occurs if the CPU hits a INT3 instruction.
///
/// - [Exception::Overflow] (Trap) - This exception occurs when `INTO` instruction is executed and
/// the Overflow Bet is set.
///
/// - [Exception::BoundRangeExceeded] (Fault) - The exception occurs when an comparison on index
/// with the lower and upper bounds of an array and the index is out of bounds.
///
/// - [Exception::InvalidOpcode] (Fault) - This exception occur when the CPU hits an invalid or
/// undefined opcode or the instruction length exceeds 15 bytes, or the instruction tries to access
/// a non-existent control register or the UD instruction is executed.
///
/// - [Exception::DeviceNotAvailable] (Fault) - This exception occurs when an FPU instruction is
/// attempted but there is no FPU. However, there are flags in the CR0 register that disable the
/// FPU/MMX/SSE instructions, causing this exception when they are attempted.
///
/// - [Exception::DoubleFault] (Abort) - This exception occurs when an exception is unhandled.
/// Normally, two exception at the same time are handles one after another, but in some cases that
/// is impossible.
///
/// - [Exception::SegmentNotPresent] (Fault) - This exception occurs when the CPU tries to load a
/// segment or gate that doesn't have set the Present bit. If the exception happens during a
/// hardware task switch, the segment values should not be relied upon by the handler.
///
/// - [Exception::StackSegmentFault] (Fault) - This exception occurs when the CPU tries to load a
/// not-present segment descriptor, or the stack address is not in a canonical form, or the stack
/// limit check fails.
///
/// - [Exception::GeneralProtectionFault] (Fault) - This exception occurs for various reasons. The
/// saved instruction pointer points to the instruction which caused the exception.
///
/// - [Exception::PageFault] (Fault) - This exception occurs when a page directory or table entry is
/// not present in the physical memory, or the CPU attempts to load a instruction TLB with a
/// translation for a non-executable page, or a protection check failed, or a reserved bit in the
/// page directory or table entries is set to 1.
///
/// - [Exception::X87FloatingPoint] (Fault) - This exception occurs when the `FWAIT` or `WAIT`
/// instruction or any floating-point waiting instruction is executed and CR0.EN is zero and an
/// unmasked x87 floating-point exception is pending.
///
/// - [Exception::AlignmentCheck] (Fault) - This exception occurs when alignment checking is enabled
/// and an unaligned memory data is referenced.
///
/// - [Exception::MachineCheck] (Abort) - This exception occurs when the processor detects internal
/// errors, lik bad memory, bus errors etc.
///
/// - [Exception::Virtualization] (Fault) - This exception occurs when a EPT violation in VMX
/// non-root operations occurs. Not al EPT violations cause virtualization exceptions. (This fault
/// is only available on Intel CPUs with Intel VT-x)
///
/// - [Exception::ControlProtection] (Fault) - This exception occurs when a control flow transfer
/// that violates the Intel CET (Control Flow Enforcement Technology) was indicated. (This fault is
/// only available on Intel CPUs with Intel CET)
///
/// - [Exception::HypervisorInjection] (Fault) - This exception occurs when the hypervisor into a
/// secure guest VM to notify the VM of pending events (This fault is only available on AMD CPUs
/// with AMD SEV-SNP)
///
/// - [Exception::VMMCommunication] (Fault) - This exception occurs when an SEV-ES enabled guest is
/// running and a NAE event occurs. (This fault is only available on AMD CPUs with AMD SEV-ES)
///
/// - [Exception::Security] - This exception occurs when security-sensitive events under SVM are
/// occurring (This fault is only available on AMD CPUs with AMD-V)
///
/// # See also
/// - [Exceptions](https://wiki.osdev.org/Exceptions) by [OSDev.org](https://wiki.osdev.org/)
/// - [AMD64 Architecture Programmer's Manual Volume 2](https://www.amd.com/content/dam/amd/en/documents/processor-tech-docs/programmer-references/24593.pdf)
/// Chapter 8.2 by [Advanced Micro Devices, Inc.](https://www.amd.com/en.html)
/// - [Intel 64 and IA-32 Architectures Software Developer's Manual Volume 3a](https://cdrdv2-public.intel.com/782154/253668-sdm-vol-3a.pdf)
/// Chapter 6.3.1 by [Intel Corporation](https://www.intel.de/content/www/us/en/homepage.html)
#[repr(u8)]
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub enum Exception {
    /// This exception occurs when the running code is diving any number by 0 or the division result
    /// is to long to be representable. The saved instruction pointer points to the instruction which
    /// caused the exception.
    ///
    /// # See also
    /// - [Exceptions](https://wiki.osdev.org/Exceptions#Division_Error) by
    /// [OSDev.org](https://wiki.osdev.org/)
    Division = 0x0,

    /// This exception occurs under different conditions. If the exception is a fault, the
    /// instruction pointer points to the exception-causing instruction. Otherwise the save
    /// instruction pointer points to instruction after the exception-causing instruction.
    /// - Instruction Fetch Breakpoint (Fault)
    /// - General Detect Condition (Fault)
    /// - Data Read oder Write Breakpoint (Trap)
    /// - I/O Read or Write Breakpoint (Trap)
    /// - Single Step (Trap)
    /// - Task Switch (Trap)
    ///
    /// **- Error Code: No**
    ///
    /// # See also
    /// - [Exceptions](https://wiki.osdev.org/Exceptions#Debug) by
    /// [OSDev.org](https://wiki.osdev.org/)
    Debug = 0x1,

    /// This interrupt occurs for unrecoverable hardware problems. Newer computers are handling
    /// these thins over machine check exceptions etc.
    ///
    /// **- Error Code: No**
    ///
    /// # See also
    /// - [Non-Maskable Interrupt (NMI)](https://wiki.osdev.org/Non_Maskable_Interrupt) by
    /// [OSDev.org](https://wiki.osdev.org/)
    NonMaskableInterrupt = 0x2,

    /// This exception occurs when the CPU hits a INT3 instruction. The saved instruction pointer
    /// points to the byte after the INT3 instruction.
    ///
    /// **- Error Code: No**
    ///
    /// # See also
    /// - [Exceptions](https://wiki.osdev.org/Exceptions#Breakpoint) by
    /// [OSDev.org](https://wiki.osdev.org/)
    Breakpoint = 0x3,

    /// This exception occurs when `INTO` instruction is executed and the Overflow Bet is set. The
    /// saved instruction pointer points to the instruction after the exception-causing instruction.
    ///
    /// **- Error Code: No**
    ///
    /// # See also
    /// - [Exceptions](https://wiki.osdev.org/Exceptions#Overflow) by
    /// [OSDev.org](https://wiki.osdev.org/)
    Overflow = 0x4,

    /// This exception occurs when an comparison on index with the lower and upper bounds of an array
    /// and the index is out of bounds. The instruction pointer points to the exception-causing
    /// instruction.
    ///
    /// **- Error Code: No**
    ///
    /// # See also
    /// - [Exceptions](https://wiki.osdev.org/Exceptions#Bound_Range_Exceeded) by
    /// [OSDev.org](https://wiki.osdev.org/)
    BoundRangeExceeded = 0x5,

    /// This exception occur when the CPU hits an invalid or undefined opcode or the instruction
    /// length exceeds 15 bytes, or the instruction tries to access a non-existent control register
    /// or the UD instruction is executed. The instruction pointer points to the instruction which
    /// caused the exception.
    ///
    /// **- Error Code: No**
    ///
    /// # See also
    /// - [Exceptions](https://wiki.osdev.org/Exceptions#Invalid_Opcode) by
    /// [OSDev.org](https://wiki.osdev.org/)
    InvalidOpcode = 0x6,

    /// This exception occurs when an FPU instruction is attempted but there is no FPU. However, t
    /// here are flags in the CR0 register that disable the FPU/MMX/SSE instructions, causing this
    /// exception when they are attempted. This feature is useful because the operating system can
    /// detect when a user program uses the FPU or XMM registers and then save/restore them
    /// appropriately when multitasking. The saved instruction pointer points to the instruction
    /// that caused the exception.
    ///
    /// **- Error Code: No**
    ///
    /// # See also
    /// - [Exceptions](https://wiki.osdev.org/Exceptions#Device_Not_Available) by
    /// [OSDev.org](https://wiki.osdev.org/)
    DeviceNotAvailable = 0x7,

    /// This exception occurs when an exception is unhandled. Normally, two exception at the same
    /// time are handles one after another, but in some cases that is impossible. The saved
    /// instruction pointer is undefined. A double fault cannot be recovered. The faulting process
    /// must be terminated.
    ///
    /// **- Error Code: Yes (Zero)**
    ///
    /// # See also
    /// - [Exceptions](https://wiki.osdev.org/Exceptions#Double_Fault) by
    /// [OSDev.org](https://wiki.osdev.org/)
    DoubleFault = 0x8,

    /// This exception occurs when an illegal segment selector is referenced as part of a task
    /// switch. When the exception occurred before loading the segment selectors from the TSS, the
    /// saved instruction pointer points to the instruction which caused the exception. Otherwise,
    /// and this is more common, it points to the first instruction in the new task.
    ///
    /// **- Error Code: Yes (Index of the Segment Selector)**
    ///
    /// # See also
    /// - [Exceptions](https://wiki.osdev.org/Exceptions#Invalid_TTS) by
    /// [OSDev.org](https://wiki.osdev.org/)
    InvalidTSS = 0xA,

    /// This exception occurs when the CPU tries to load a segment or gate that doesn't have set
    /// the Present bit. If the exception happens during a hardware task switch, the segment values
    /// should not be relied upon by the handler. The saved instruction pointer points to the
    /// instruction that caused the exception.
    ///
    /// **- Error Code: Yes (Segment Selector Index of the Segment Selector)**
    ///
    /// # See also
    /// - [Exceptions](https://wiki.osdev.org/Exceptions#Segment_Not_Present) by
    /// [OSDev.org](https://wiki.osdev.org/)
    SegmentNotPresent = 0xB,

    /// This exception occurs when the CPU tries to load a not-present segment descriptor, or the
    /// stack address is not in a canonical form, or the stack limit check fails. The saved
    /// instruction pointer points to the instruction which caused the exception, unless the fault
    /// occurred because of loading a non-present stack segment during a hardware task switch, in
    /// which case it points to the next instruction of the new task.
    ///
    /// **- Error Code: Yes (The Stack-Segment Fault sets an error code, which is the stack segment
    /// selector index when a non-present segment descriptor was referenced or a limit check failed
    /// during a hardware task switch. Otherwise (for present segments and already in use), the
    /// error code is 0)**
    ///
    /// # See also
    /// - [Exceptions](https://wiki.osdev.org/Exceptions#Stack-Segment_Fault) by
    /// [OSDev.org](https://wiki.osdev.org/)
    StackSegmentFault = 0xC,

    /// This exception occurs for various reasons. The saved instruction pointer points to the
    /// instruction which caused the exception. The following text shows a list of the most common
    /// reason.
    /// - Segment Error
    /// - Execution a privileged Instruction while CPL != 0
    /// - Writing a 1 in a reserved register field or  writing invalid value combinations
    /// - Referencing or accessing null-descriptors
    ///
    /// **- Error: The segment selector index when the exception is segment related. In every other
    /// case zero**
    ///
    /// # See also
    /// - [Exceptions](https://wiki.osdev.org/Exceptions#General_Protection_Fault) by
    /// [OSDev.org](https://wiki.osdev.org/)
    GeneralProtectionFault = 0xD,

    /// This exception occurs when a page directory or table entry is not present in the physical
    /// memory, or the CPU attempts to load a instruction TLB with a translation for a
    /// non-executable page, or a protection check failed, or a reserved bit in the page directory
    /// or table entries is set to 1. The saved instruction pointer points to the exception-causing
    /// instruction.
    ///
    /// **- Error Code: No**
    ///
    /// # See also
    /// - [Exceptions](https://wiki.osdev.org/Exceptions#Page_Fault) by
    /// [OSDev.org](https://wiki.osdev.org/)
    PageFault = 0xE,

    /// This exception occurs when the `FWAIT` or `WAIT` instruction or any floating-point waiting
    /// instruction is executed and CR0.EN is zero and an unmasked x87 floating-point exception is
    /// pending. The saved instruction pointer points to the instruction which is about to be
    /// executed when the exception occurred. The x87 instruction pointer register contains the
    /// address of the last instruction which caused the exception.
    ///
    /// **- Error Code: No**
    ///
    /// # See also
    /// - [Exceptions](https://wiki.osdev.org/Exceptions#x87_Floating-Point_Exception) by
    /// [OSDev.org](https://wiki.osdev.org/)
    X87FloatingPoint = 0x10,

    /// This exception occurs when alignment checking is enabled and an unaligned memory data is
    /// referenced. Alignment checking is only performed in CPU privilege level
    /// [crate::PrivilegeLevel::UserSpace]. The saved instruction pointer points to the
    /// exception-causing instruction.
    ///
    /// **- Error Code: No**
    ///
    /// # See also
    /// - [Exceptions](https://wiki.osdev.org/Exceptions#Alignment_Check) by
    /// [OSDev.org](https://wiki.osdev.org/)
    AlignmentCheck = 0x17,

    /// This exception occurs when the processor detects internal errors, lik bad memory, bus errors
    /// etc. The value of the saved instruction pointer depends on the implementation and the
    /// exception.
    ///
    /// **- Error Code: No**
    ///
    /// # See also
    /// - [Exceptions](https://wiki.osdev.org/Exceptions#Machine_Check) by
    /// [OSDev.org](https://wiki.osdev.org/)
    MachineCheck = 0x18,

    /// This exception occurs when an unmasked 128-bit media floating-point exception occurs and the
    /// CR4.OSXMMEXCPT bit is set. If not, then these exceptions will cause a
    /// [Exception::InvalidOpcode]. The saved instruction pointer points to the exception-causing
    /// instruction.
    ///
    /// **- Error Code: No**
    ///
    /// # See also
    /// - [Exceptions](https://wiki.osdev.org/Exceptions#SIMD_Floating-Point_Exception) by
    /// [OSDev.org](https://wiki.osdev.org/)
    SIMDFloatingPoint = 0x19,

    /// This exception occurs when a EPT violation in VMX non-root operations occurs. Not al EPT
    /// violations cause virtualization exceptions. The handler can recover from these exception
    /// and restart the program/task without any loss. In some cases, these exceptions are
    /// unrecoverable. The saved instruction pointer points to the exception-causing
    /// instruction. (This fault is only available on Intel CPUs with Intel VT-x)
    ///
    /// **- Error Code: No**
    ///
    /// # See also
    /// - [Intel 64 and IA-32 Architectures Software Developer's Manual Volume 3a](https://cdrdv2-public.intel.com/782154/253668-sdm-vol-3a.pdf)
    /// Chapter 6.15 (Interrupt 20) by [Intel Corporation](https://www.intel.de/content/www/us/en/homepage.html)
    Virtualization = 0x14,

    /// This exception occurs when a control flow transfer that violates the Intel CET (Control Flow
    /// Enforcement Technology) was indicated. The saved instruction pointer points to the
    /// exception-causing instruction. (This fault is only available on Intel CPUs with Intel CET)
    ///
    /// **- Error Code: Yes (32 bits value, last 16 bits are reserved)**
    ///
    /// This fault is providing a few parameters of the error code:
    /// - Bit 14:0
    ///    - 1 => Indicates the exception was caused near a RET instruction
    ///    - 2 => Indicates the exception was caused by a FAR RET or IRET instruction
    ///    - 3 => Indicates the exception was caused due to missing `ENDBRANCH` at target of an
    ///    indirect call or jump instruction
    ///    - 4 => Indicates the exception was caused by a show-stack-restore token check failure
    ///    in the `RSTORSSP` instruction
    ///    - 5 => Indicates the exception was caused by a supervisor shadow stack token check
    ///    failure in th `SETSSBSY` instruction
    /// - Bit 15 of the error code, if set to 1, indicates the exception occurred during enclave
    /// exception.
    ///
    /// # See also
    /// - [Intel 64 and IA-32 Architectures Software Developer's Manual Volume 3a](https://cdrdv2-public.intel.com/782154/253668-sdm-vol-3a.pdf)
    /// Chapter 6.15 (Interrupt 21) by [Intel Corporation](https://www.intel.de/content/www/us/en/homepage.html)
    /// - [A Technical Look at Intel's Control-flow Enforcement Technology](https://www.intel.com/content/www/us/en/developer/articles/technical/technical-look-control-flow-enforcement-technology.html)
    /// by [Intel Corporation](https://www.intel.de/content/www/us/en/homepage.html)
    ControlProtection = 0x21,

    /// This exception occurs when the hypervisor into a secure guest VM to notify the VM of pending
    /// events. (This fault is only available on AMD CPUs with SEV-SNP)
    ///
    /// **- Error Code: No**
    ///
    /// The Hypervisor Injection Exception refers to specific behaviors and exceptions that occur
    /// when a hypervisor attempts certain operations in a virtualized environment, particularly
    /// within the context of AMD Secure Encrypted Virtualization (SEV) and Secure Nested Paging
    /// (SNP) technology.
    ///
    /// # See also
    /// - [AMD64 Architecture Programmer's Manual Volume 2](https://www.amd.com/content/dam/amd/en/documents/processor-tech-docs/programmer-references/24593.pdf)
    /// Chapter 8.2.21 by [Advanced Micro Devices, Inc.](https://www.amd.com/en.html)
    /// - [AMD64 Architecture Programmer's Manual Volume 2](https://www.amd.com/content/dam/amd/en/documents/processor-tech-docs/programmer-references/24593.pdf)
    /// Chapter 15.36.16 by [Advanced Micro Devices, Inc.](https://www.amd.com/en.html)
    HypervisorInjection = 0x1C,

    /// This exception occurs when an SEV-ES enabled guest is running and a NAE event occurs. (This
    /// fault is only available on AMD CPUs with AMD SEV-ES)
    ///
    /// **- Error Code: Yes (Equal to the VMEXIT code)**
    ///
    /// # See also
    /// - [AMD64 Architecture Programmer's Manual Volume 2](https://www.amd.com/content/dam/amd/en/documents/processor-tech-docs/programmer-references/24593.pdf)
    /// Chapter 8.2.22 by [Advanced Micro Devices, Inc.](https://www.amd.com/en.html)
    /// - [AMD64 Architecture Programmer's Manual Volume 2](https://www.amd.com/content/dam/amd/en/documents/processor-tech-docs/programmer-references/24593.pdf)
    /// Chapter 15.35.5 by [Advanced Micro Devices, Inc.](https://www.amd.com/en.html)
    VMMCommunication = 0x1D,

    /// This exception occurs when security-sensitive events under SVM are occurring. The current
    /// only use for this exception is to send external INITS into an exception so the VMM can
    /// destroy sensitive information. (This fault is only available on AMD CPUs with AMD-V)
    ///
    /// **- Error Code: Yes (Currently only 1 is defined)**
    ///
    /// # See also
    /// - [AMD64 Architecture Programmer's Manual Volume 2](https://www.amd.com/content/dam/amd/en/documents/processor-tech-docs/programmer-references/24593.pdf)
    /// Chapter 8.2.23 by [Advanced Micro Devices, Inc.](https://www.amd.com/en.html)
    /// - [AMD64 Architecture Programmer's Manual Volume 2](https://www.amd.com/content/dam/amd/en/documents/processor-tech-docs/programmer-references/24593.pdf)
    /// Chapter 15.28 by [Advanced Micro Devices, Inc.](https://www.amd.com/en.html)
    Security = 0x30,
}

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
/// TODO: 32bit Support
///
/// # See also
/// - [Interrupt Descriptor Table (IA-32)](https://wiki.osdev.org/Interrupt_Descriptor_Table#Structure_on_IA-32)
/// by [OSDev.org](https://wiki.osdev.org/)
/// - [Interrupt Descriptor Table (x86_64)](https://wiki.osdev.org/Interrupt_Descriptor_Table#Structure_on_x86-64)
/// by [OSDev.org](https://wiki.osdev.org/)
#[repr(C, packed)]
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub struct IDTDescriptor {
    lower_isr_address: u16,
    segment_selector: SegmentSelector,
    ist: u8,
    flags: u8,
    middle_isr_address: u16,
    higher_isr_address: u32,
    reserved: u32,
}

impl Default for IDTDescriptor {
    fn default() -> Self {
        IDTDescriptor::new(
            (default_interrupt_handler as *const ()) as u64,
            SegmentSelector::new(1, DescriptorTable::GDT, PrivilegeLevel::KernelSpace),
            GateType::Trap,
            PrivilegeLevel::KernelSpace,
        )
    }
}

impl IDTDescriptor {
    pub fn new(
        handler_address: MemoryAddress, selector: SegmentSelector, gate_type: GateType,
        privilege_level: PrivilegeLevel,
    ) -> Self {
        Self {
            lower_isr_address: (handler_address & 0xFFFF) as u16,
            segment_selector: selector,
            ist: 0,
            flags: 0b1000_0000 | (privilege_level as u8) | (gate_type as u8),
            middle_isr_address: ((handler_address >> 16) & 0xFFFF) as u16,
            higher_isr_address: ((handler_address >> 32) & 0xFFFFFFFF) as u32,
            reserved: 0,
        }
    }
}

/// This structure represents the Interrupt Descriptor Table with the maximum of 256 entries. In #
/// this structure, we store the descriptors in a slice.
///
/// - `descriptors` - This field is a slice that can store 8192 [IDTDescriptor]s
/// - `count` This field holds the max index that is used to insert a descriptor for the
/// [DescriptorTablePointer]
///
/// # See also
/// - [Interrupt Descriptor Table](https://wiki.osdev.org/IDT) by
/// [OSDev.org](https://wiki.osdev.org)
/// - [Interrupts Tutorial](https://wiki.osdev.org/Interrupts_tutorial) by
/// [OSDev.org](https://wiki.osdev.org)
/// - [IDTDescriptor] (Source Code)
#[repr(C, packed)]
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub struct InterruptDescriptorTable {
    descriptors: [IDTDescriptor; 256],
}

impl Default for InterruptDescriptorTable {
    #[must_use]
    fn default() -> Self {
        Self {
            descriptors: [IDTDescriptor::default(); 256],
        }
    }
}

impl InterruptDescriptorTable {
    /// This function generates a pointer to the IDT with the [InterruptDescriptorTable::as_ptr]
    /// function and loads it with the `lidt` instruction.
    ///
    /// # See also
    /// - [LGDT/LIDT](https://www.felixcloutier.com/x86/lgdt:lidt) by
    /// [Felix Clountier](https://www.felixcloutier.com)
    pub fn load(&self) {
        unsafe {
            asm!("lidt [{}]", in(reg) &self.as_ptr(), options(readonly, nostack, preserves_flags));
        }
    }

    /// This function inserts a [IDTDescriptor] at the specified index in the IDT.
    pub fn insert(&mut self, index: usize, descriptor: IDTDescriptor) {
        self.descriptors[index] = descriptor;
    }

    /// This function generates a pointer to the Interrupt Descriptor Table (IDT) with the base
    /// address and the size of the IDT as limit.
    ///
    /// # See also
    /// - [Global Descriptor Table](https://wiki.osdev.org/Global_Descriptor_Table#GDTR) by
    /// [OSDev.org](https://wiki.osdev.org)
    #[must_use]
    pub fn as_ptr(&self) -> DescriptorTablePointer {
        DescriptorTablePointer {
            base: self.descriptors.as_ptr() as MemoryAddress,
            size: (256 * size_of::<IDTDescriptor>() - 1) as u16,
        }
    }
}
