//! This module implements the IA-32 and x86_64 specific `CPUID` instruction and wraps the
//! structures and implementations over a user-friendly Rust API.
//!
//! # See also
//! - [CPUID](https://wiki.osdev.org/CPUID) by [OSDev.org](https://wikie.osdev.org/)
//! - [CPUID - CPU Identification](https://www.felixcloutier.com/x86/cpuid) by
//! [Felix Cloutier](https://www.felixcloutier.com/)

// TODO: Document every feature

use alloc::{
    fmt::{
        Display,
        Error,
        Formatter,
    },
    string::{
        String,
        ToString,
    },
    vec::Vec,
};
use core::arch::x86_64::__cpuid;

macro_rules! features {
    ($(#[$attr:meta])* pub enum $name: ident { $($feature: ident ($register: ident, $display: expr) =
    $value: expr),* }) => {
        $(#[$attr])*
        #[repr(u8)]
        pub enum $name {
            $(
            $feature,
            )*
        }

        impl Display for $name {
            fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
                match self {
                    $(
                    Self::$feature => write!(formatter, "{}", $display),
                    )*
                }
            }
        }

        #[must_use]
        pub fn request_cpu_features() -> Vec<$name> {
            let cpuid_result = unsafe { __cpuid(1) };
            let mut data = Vec::new();
            $(
            if (cpuid_result.$register & $value == $value) {
                data.push($name::$feature);
            }
            )*
            data
        }
    }
}

macro_rules! vendor {
    ($(#[$attr:meta])* pub enum $name: ident { $($vendor: ident ($($vendor_string: expr),*) =
    $display: expr),* }) => {
        $(#[$attr])*
        #[repr(u8)]
        pub enum $name {
            $(
            $vendor,
            )*
        }

        impl Display for $name {
            fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
                match self {
                    $(
                    Self::$vendor => write!(formatter, "{}", $display),
                    )*
                }
            }
        }

        impl From<String> for $name {
            #[must_use]
            fn from(value: String) -> Self {
                match value.as_str() {
                    $(
                    $(
                    $vendor_string => Self::$vendor,
                    )*
                    )*
                    _ => panic!("Unknown CPU Vendor: {}", value)
                }
            }
        }

        #[must_use]
        pub fn request_cpu_vendor() -> $name {
            let cpuid_result = unsafe { __cpuid(0) };
            $name::from(String::from_utf8_lossy(
                &[
                    cpuid_result.ebx.to_ne_bytes(),
                    cpuid_result.edx.to_ne_bytes(),
                    cpuid_result.ecx.to_ne_bytes()
                ].concat()
            ).trim().to_string())
        }
    }
}

vendor! {
    #[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
    pub enum CPUVendor {
        AMD        ("AuthenticAMD", "AMDisbetter!") = "Advanced Micro Devices, Inc.",
        Intel      ("GenuineIntel")                 = "Intel Corporation",
        VIA        ("VIA VIA VIA ")                 = "Via Technologies, Inc.",
        Transmeta  ("GenuineTMx86", "TransmetaCPU") = "Transmeta Corporation",
        Cyrix      ("CyrixInstead")                 = "Cyrix Corporation",
        Centaur    ("CentaurHauls")                 = "Centaur Technology",
        NexGen     ("NexGenDriven")                 = "NexGen Incorporated",
        UMC        ("UMC UMC UMC ")                 = "United Microelectronics Corporation",
        SiS        ("SiS SiS SiS ")                 = "Silicon Integrated Systems Corp.",
        Rise       ("RiseRiseRise")                 = "Rise Technology",
        Vortex     ("Vortex86 SoC")                 = "Silicon Integrated Systems Corp.", // I think so
        AO486      ("MiSTer AO486", "GenuineAO486") = "AO486",
        Zhaoxin    ("  Shanghai  ")                 = "Shanghai Zhaoxin Semiconductor Co., Ltd.",
        Hygon      ("HygonGenuine")                 = "Hygon Information Technology Co., Ltd.",
        Elbrus     ("E2K MACHINE ")                 = "Moscow Center of SPARC Technologies",

        // Virtual
        QEMU       ("TCGTCGTCGTCG")                 = "Quick Emulator (QEMU)",
        KVM        (" KVMKVMKVM  ")                 = "Kernel-based Virtual Machine",
        VMWARE     ("VMwareVMware")                 = "VMware",
        VirtualBox ("VBoxVBoxVBox")                 = "VirtualBox",
        Xen        ("XenVMMXenVMM")                 = "Xen Hypervisor",
        HyperV     ("Microsoft Hv")                 = "Microsoft HyperV",
        Parallels  (" prl hyperv ", " lrpepyh vr ") = "Parallels",
        BHYVE      ("bhyve bhyve ")                 = "bhyve",
        QNX        (" QNXQVMBSQG ")                 = "QNX Hypervisor"
    }
}

features! {
    #[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
    pub enum CPUFeature {
        SSE3        (ecx, "SSE3") = 1 << 0,
        PCLMUL      (ecx, "Carry-Less Multiplication") = 1 << 1,
        DTES64      (ecx, "64-Bit Debug Store") = 1 << 2,
        MONITOR     (ecx, "MONITOR and MWAIT instructions") = 1 << 3,
        DS_CPL      (ecx, "CPL-qualified Debug Store") = 1 << 4,
        VMX         (ecx, "Virtual Machine Extensions") = 1 << 5,
        SMX         (ecx, "Safer Mode Extensions") = 1 << 6,
        EST         (ecx, "Enhanced SpeedStep") = 1 << 7,
        TM2         (ecx, "Thermal Monitor 2") = 1 << 8,
        SSSE3       (ecx, "Supplemental SSE3") = 1 << 9,
        CID         (ecx, "L1 Context ID") = 1 << 10,
        SDBG        (ecx, "Silicon Debug Interface") = 1 << 11,
        FMA         (ecx, "Fused Multiply-Add (FMA3)") = 1 << 12,
        CX16        (ecx, "CMPXCHG1B instruction") = 1 << 13,
        XTPR        (ecx, "Can disable sending task priority messages") = 1 << 14,
        PDCM        (ecx, "Prefmon & Debug Capability") = 1 << 15,
        PCID        (ecx, "Process Context Identifiers") = 1 << 17,
        DCA         (ecx, "Direct Cache Access for DMA writes") = 1 << 18,
        SSE4_1      (ecx, "SSE4.1 instructions") = 1 << 19,
        SSE4_2      (ecx, "SSE4.2 instructions") = 1 << 20,
        X2APIC      (ecx, "x2APIC (enhanced APIC)") = 1 << 21,
        MOVBE       (ecx, "MOVBE instruction") = 1 << 22,
        POPCNT      (ecx, "POPCNT instruction") = 1 << 23,
        TSCDeadline (ecx, "APIC implements one-shot operation using a TSC deadline value") = 1 << 24,
        AES         (ecx, "Hardware-accelerated AES Instruction Set") = 1 << 25,
        XSAVE       (ecx, "Extensible processor state restore instructions") = 1 << 26,
        OSXSAVE     (ecx, "XSAVE enabled by OS") = 1 << 27,
        AVX         (ecx, "Advanced Vector Extensions (256-bit SIMD)") = 1 << 28,
        F16C        (ecx, "Floating-point conversion instructions to/from FP16 format") = 1 << 29,
        RDRAND      (ecx, "RDRAND (HRNG) feature") = 1 << 30,
        HYPERVISOR  (ecx, "Hypervisor is present") = 1 << 31,
        FPU         (edx, "Onboard x87 FPU") = 1 << 0,
        VME         (edx, "Virtual 8086 Mode Extensions") = 1 << 1,
        DE          (edx, "Debugging Extensions") = 1 << 2,
        PSE         (edx, "Page Size Extension (4MB pages)") = 1 << 3,
        TSC         (edx, "Time Stamp Counter and RDTSC instruction") = 1 << 4,
        MSR         (edx, "Model-Specific Registers and RDMSR/WRMSR instructions") = 1 << 5,
        PAE         (edx, "Physical Address Extension") = 1 << 6,
        MCE         (edx, "Machine Check Exception") = 1 << 7,
        CX8         (edx, "CMPXCHG8B instruction") = 1 << 8,
        APIC        (edx, "Onboard APIC") = 1 << 9,
        SEP         (edx, "SYSENTER and SYSEXIT fast System Call instuctions") = 1 << 11,
        MTRR        (edx, "Memory Type Range Registers") = 1 << 12,
        PGE         (edx, "Page Global Enable bit") = 1 << 13,
        MCA         (edx, "Machine Check Architecture") = 1 << 14,
        CMOV        (edx, "Conditional move instructions") = 1 << 15,
        PAT         (edx, "Page Attribute Table") = 1 << 16,
        PSE36       (edx, "36-bit Page Size Extension") = 1 << 17,
        PSN         (edx, "Processor Serial Number enabled") = 1 << 18,
        CLFLUSH     (edx, "CLFLUSH cache line flush instruction") = 1 << 19,
        NX          (edx, "Non-Executable Bit (Itanium only)") = 1 << 20,
        DS          (edx, "Debug Store (Save trace of jumps)") = 1 << 21,
        ACPI        (edx, "Onboard Thermal Control MSRs for ACPI") = 1 << 22,
        MMX         (edx, "MMX instructions (64-bit SIMD)") = 1 << 23,
        FXSR        (edx, "FXSAVE and FXRSTOR instructions") = 1 << 24,
        SSE         (edx, "Streaming SIMD Extensions (128-bit SIMD)") = 1 << 25,
        SSE2        (edx, "SSE2 instructions") = 1 << 26,
        SS          (edx, "CPU Cache implements self-snoop") = 1 << 27,
        HTT         (edx, "Mac APIC IDs reserved field is valid") = 1 << 28,
        TM          (edx, "Thermal Monitor automatically limits temperature") = 1 << 29,
        IA64        (edx, "IA64 Processor emulating x86") = 1 << 30,
        PBE         (edx, "Pending Break Enable wakeup capacity") = 1 << 31
    }
}
