pub mod gdt;
pub mod idt;

use core::arch::asm;

#[cfg(target_arch = "x86")]
type Register = u32;

#[cfg(target_arch = "x86_64")]
type Register = u64;

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

/*#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
pub struct ProcessorState {
    pub ax: Register,
    pub cx: Register,
    pub dx: Register,
    pub bx: Register,
    pub sp: Register,
    pub bp: Register,
    pub si: Register,
    pub di: Register,
    pub ip: Register,
    #[cfg(target_feature = "sse")]
    pub xmm8: u128,
    #[cfg(target_feature = "sse")]
    pub xmm9: u128,
    #[cfg(target_feature = "sse")]
    pub xmm10: u128,
    #[cfg(target_feature = "sse")]
    pub xmm11: u128,
    #[cfg(target_feature = "sse")]
    pub xmm12: u128,
    #[cfg(target_feature = "sse")]
    pub xmm13: u128,
    #[cfg(target_feature = "sse")]
    pub xmm14: u128,
    #[cfg(target_feature = "sse")]
    pub xmm15: u128

}

impl ProcessorState {
    pub fn save() -> ProcessorState {
        let mut state = ProcessorState::default();

        // Save default x86_64 registers
        #[cfg(target_arch = "x86_64")]
        unsafe {
            asm!(
                "mov {0}, rax",
                "mov {1}, rcx",
                "mov {2}, rdx",
                "mov {3}, rbx",
                "mov {4}, rsp",
                "mov {5}, rbp",
                "mov {6}, rsi",
                "mov {7}, rdi",
                "mov {8}, rip",
                out(reg) state.ax,
                out(reg) state.cx,
                out(reg) state.dx,
                out(reg) state.bx,
                out(reg) state.sp,
                out(reg) state.bp,
                out(reg) state.si,
                out(reg) state.di,
                out(reg) state.ip
            );
        }

        // Save unrecoverable SSE registers
        #[cfg(all(target_arch = "x86_64", target_feature = "sse"))]
        unsafe {
            asm!(
            "mov {0}, xmm8",
            "mov {1}, xmm9",
            "mov {2}, xmm10",
            "mov {3}, xmm11",
            "mov {4}, xmm12",
            "mov {5}, xmm13",
            "mov {6}, xmm14",
            "mov {7}, xmm15",
            out(reg) state.xmm8,
            out(reg) state.xmm9,
            out(reg) state.xmm10,
            out(reg) state.xmm11,
            out(reg) state.xmm12,
            out(reg) state.xmm13,
            out(reg) state.xmm14,
            out(reg) state.xmm15
            );
        }



        state
    }

    pub fn load(&self) {
        // Load default x86_64 registers
        #[cfg(target_arch = "x86_64")]
        unsafe {
            asm!(
                "mov rax, {0}",
                "mov rcx, {1}",
                "mov rdx, {2}",
                "mov rbx, {3}",
                "mov rsp, {4}",
                "mov rbp, {5}",
                "mov rsi, {6}",
                "mov rdi, {7}",
                "mov rip, {8}",
                in(reg) self.ax,
                in(reg) self.cx,
                in(reg) self.dx,
                in(reg) self.bx,
                in(reg) self.sp,
                in(reg) self.bp,
                in(reg) self.si,
                in(reg) self.di,
                in(reg) self.ip
            )
        }

        // Load unrecoverable SEE registers
        #[cfg(all(target_arch = "x86_64", target_feature = "sse"))]
        unsafe {
            asm!(
                "mov xmm8, {0}",
                "mov xmm9, {1}",
                "mov xmm10, {2}",
                "mov xmm11, {3}",
                "mov xmm12, {4}",
                "mov xmm13, {5}",
                "mov xmm14, {6}",
                "mov xmm15, {7}",
                in(reg) self.xmm8,
                in(reg) self.xmm9,
                in(reg) self.xmm10,
                in(reg) self.xmm11,
                in(reg) self.xmm12,
                in(reg) self.xmm13,
                in(reg) self.xmm14,
                in(reg) self.xmm15
            );
        }
    }
}*/

#[inline]
pub fn wait_for_interrupts() {
    unsafe { asm!("hlt") }
}
