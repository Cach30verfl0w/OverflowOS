//! This module implements the IA-32 and x86_64 specific `CPUID` instruction and wraps the
//! structures and implementations over a user-friendly Rust API.
//!
//! # See also
//! - [CPUID](https://wiki.osdev.org/CPUID) by [OSDev.org](https://wikie.osdev.org/)
//! - [CPUID - CPU Identification](https://www.felixcloutier.com/x86/cpuid) by
//! [Felix Cloutier](https://www.felixcloutier.com/)

use bitflags::bitflags;
use core::arch::x86_64::__cpuid;

// TODO: Document every feature

bitflags! {
    #[rustfmt::skip]
    #[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
    pub struct CPUFeatures: u32 {
        const SSE3       = 1 << 0;
        const PCLMUL     = 1 << 1;
        const DTES64     = 1 << 2;
        const MONITOR    = 1 << 3;
        const DS_CPL     = 1 << 4;
        const VMX        = 1 << 5;
        const SMX        = 1 << 6;
        const EST        = 1 << 7;
        const TM2        = 1 << 8;
        const SSSE3      = 1 << 9;
        const CID        = 1 << 10;
        const SDBG       = 1 << 11;
        const FMA        = 1 << 12;
        const CX16       = 1 << 13;
        const XTPR       = 1 << 14;
        const PDCM       = 1 << 15;
        const PCID       = 1 << 17;
        const DCA        = 1 << 18;
        const SSE4_1     = 1 << 19;
        const SEE4_2     = 1 << 20;
        const X2APIC     = 1 << 21;
        const MOVBE      = 1 << 22;
        const POPCNT     = 1 << 23;
        const TSC        = 1 << 24;
        const AES        = 1 << 25;
        const XSAVE      = 1 << 26;
        const OSXSAVE    = 1 << 27;
        const AVX        = 1 << 28;
        const F16C       = 1 << 29;
        const RDRAND     = 1 << 30;
        const HYPERVISOR = 1 << 31;
    }
}

#[must_use]
pub fn get_cpu_features() -> CPUFeatures {
    CPUFeatures::from_bits_retain(unsafe { __cpuid(1) }.ecx)
}
