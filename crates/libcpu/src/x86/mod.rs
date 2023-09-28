pub mod gdt;
pub mod idt;

use core::arch::asm;

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

#[repr(C, packed(2))]
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
struct DescTablePointer {
    limit: u16,
    base: u64,
}

#[inline]
pub fn wait_for_interrupts() {
    unsafe { asm!("hlt") }
}
