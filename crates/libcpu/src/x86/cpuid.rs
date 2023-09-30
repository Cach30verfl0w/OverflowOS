//! This module implements the IA-32 and x86_64 specific `CPUID` instruction and wraps the
//! structures and implementations over a user-friendly Rust API.
//!
//! Here is a list with all implemented variations of the `CPUID` instruction:
//! - `EAX = 0` => Indicator of the Processor Vendor
//! - `EAX = 1` => Bits of the processor's features
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
pub use core::arch::x86_64::has_cpuid;

macro_rules! features {
    ($(#[$attr:meta])* pub enum $name: ident { $($(#[$feature_attr: meta])* $feature: ident ($register: ident, $display: expr) =
    $value: expr),* }) => {
        $(#[$attr])*
        #[repr(u8)]
        pub enum $name {
            $(
            $(#[$feature_attr])*
            $feature,
            )*
            Unknown
        }

        impl Display for $name {
            fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
                match self {
                    $(
                    Self::$feature => write!(formatter, "{}", $display),
                    )*
                    Self::Unknown => write!(formatter, "Unknown")
                }
            }
        }

        #[must_use]
        pub fn request_cpu_features() -> Vec<$name> {
            let mut data = Vec::new();
            if has_cpuid() {
                let cpuid_result = unsafe { __cpuid(1) };
                $(
                if (cpuid_result.$register & $value == $value) {
                    data.push($name::$feature);
                }
                )*
            }
            data
        }
    }
}

macro_rules! vendor {
    ($(#[$attr:meta])* pub enum $name: ident { $($(#[$vendor_addr:meta])* $vendor: ident ($($vendor_string: expr),*) =
    $display: expr),* }) => {
        $(#[$attr])*
        #[repr(u8)]
        pub enum $name {
            $(
            $(#[$vendor_addr])*
            $vendor,
            )*
            Unknown
        }

        impl Display for $name {
            fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
                match self {
                    $(
                    Self::$vendor => write!(formatter, "{}", $display),
                    )*
                    Self::Unknown => write!(formatter, "Unknown")
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
            if has_cpuid() {
                let cpuid_result = unsafe { __cpuid(0) };
                $name::from(String::from_utf8_lossy(
                    &[
                        cpuid_result.ebx.to_ne_bytes(),
                        cpuid_result.edx.to_ne_bytes(),
                        cpuid_result.ecx.to_ne_bytes()
                    ].concat()
                ).trim().to_string())
            } else {
                $name::Unknown
            }
        }
    }
}

vendor! {
    /// This enum lists all variant of known x86 processor vendors. You can call [request_cpu_vendor]
    /// to get the enum value of the processor vendor on the current system.
    ///
    /// Here is a list with all companies/products and the company/group name:
    /// - [CPUVendor::AMD] (Semiconductor Company) - Advanced Micro Devices, Inc.
    /// - [CPUVendor::Intel] (Semiconductor Chip Manufacturer) - Intel Corporation
    /// - [CPUVendor::VIA] (Semiconductor Company) - VIA Technologies, Inc.
    /// - [CPUVendor::Transmeta] (Semiconductor Design Company) - Transmeta Corporation
    /// - [CPUVendor::Cyrix] (Microprocessor Developer) - Cyrix Corporation
    /// - [CPUVendor::Centaur] (x86 CPU Design Company) - Centaur Technology
    /// - [CPUVendor::NexGen] (Semiconductor Company) - NexGen Incorporated
    /// - [CPUVendor::UMX] (Semiconductor Company) - United Microelectronics Corporation
    /// - [CPUVendor::SiS] (Manufacturing Company) - Silicon Integrated Systems Corp.
    /// - [CPUVendor::Rise] (Microprocessor Company) - Rise Technology
    /// - [CPUVendor::Vortex] (Processor Family) - Silicon Integrated Systems Corp.
    /// - [CPUVendor::AO486] (Processor Family) - Advanced Micro Devices, Inc.
    /// - [CPUVendor::Zhaoxin] (Semiconductor Family) - Shanghai Zhaoxin Semiconductor Co., Ltd.
    /// - [CPUVendor::Hygon] (Manufacturing Company) - Hygon Information Technology Co., Ltd.
    /// - [CPUVendor::Elbrus] (Processor Family/Company) - Moscow Center of SPARC Technologies
    /// - [CPUVendor::QEMU] (Emulation Software) - Quick Emulator (QEMU)
    /// - [CPUVendor::KVM] (Virtualization Module) - Kernel-based Virtual Machine (KVM)
    /// - [CPUVendor::VMware] (Product/Company) - VMware, Inc.
    /// - [CPUVendor::VirtualBox] (Product) - Oracle Corporation
    /// - [CPUVendor::Xen] (Hypervisor) - Xen Hypervisor
    /// - [CPUVendor::HyperV] (Hypervisor) - Microsoft Corporation
    /// - [CPUVendor::Parallels] (Hypervisor) - Parallels
    /// - [CPUVendor::BHYVE] (Hypervisor) - Bhyve
    /// - [CPUVendor::QNX] (Hypervisor) - QNX
    #[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
    pub enum CPUVendor {
        /// This variant indicates the American semiconductor company
        /// [Advanced Micro Devices, Inc. (AMD)](https://en.wikipedia.org/wiki/AMD) as the CPU
        /// vendor
        AMD        ("AuthenticAMD", "AMDisbetter!") = "Advanced Micro Devices, Inc.",

        /// This variant indicates the American semiconductor chip manufacturer
        /// [Intel Corporation](https://en.wikipedia.org/wiki/Intel) as the CPU vendor
        Intel      ("GenuineIntel")                 = "Intel Corporation",

        /// This variant indicates the Taiwanese semiconductor company
        /// [Via Technologies Inc.](https://en.wikipedia.org/wiki/VIA_Technologies) as the CPU
        /// vendor
        VIA        ("VIA VIA VIA ")                 = "Via Technologies, Inc.",

        /// This variant indicates the American semiconductor design company
        /// [Transmeta Corporation](https://en.wikipedia.org/wiki/Transmeta) as the CPU vendor
        Transmeta  ("GenuineTMx86", "TransmetaCPU") = "Transmeta Corporation",

        /// This variant indicates the American microprocessor developer
        /// [Cyrix Corporation](https://en.wikipedia.org/wiki/Cyrix) as the CPU vendor
        Cyrix      ("CyrixInstead")                 = "Cyrix Corporation",

        /// This variant indicates the American x86 CPU design company
        /// [Centaur Technology, Inc.](https://en.wikipedia.org/wiki/Centaur_Technology) as the CPU
        /// vendor
        Centaur    ("CentaurHauls")                 = "Centaur Technology",

        /// This variant indicates the American semiconductor company
        /// [NexGen, Inc.](https://en.wikipedia.org/wiki/NexGen) as the CPU vendor
        NexGen     ("NexGenDriven")                 = "NexGen Incorporated",

        /// This variant indicates the Taiwanese semiconductor company
        /// [United Microelectronics Corporation](https://en.wikipedia.org/wiki/United_Microelectronics_Corporation)
        /// as the CPU vendor
        UMC        ("UMC UMC UMC ")                 = "United Microelectronics Corporation",

        /// This variant indicates the Taiwanese manufacturing company
        /// [Silicon Integrated Systems](https://en.wikipedia.org/wiki/Silicon_Integrated_Systems)
        /// as the CPU vendor
        SiS        ("SiS SiS SiS ")                 = "Silicon Integrated Systems Corp.",

        /// This variant indicates the American microprocessor manufacturer
        /// [Rise Technology](https://en.wikipedia.org/wiki/Rise_Technology)
        Rise       ("RiseRiseRise")                 = "Rise Technology",

        /// This variant indicates a processor family of the Taiwanese manufacturing company
        /// [Silicon Integrated Systems](https://en.wikipedia.org/wiki/Silicon_Integrated_Systems)
        /// (I think so)
        Vortex     ("Vortex86 SoC")                 = "Silicon Integrated Systems Corp.",

        /// This variant indicates a processor family of the American semiconductor company
        /// [Advanced Micro Devices, Inc. (AMD)](https://en.wikipedia.org/wiki/AMD)
        AO486      ("MiSTer AO486", "GenuineAO486") = "AO486",

        /// This variant indicates the Chinese semiconductor company
        /// [Shanghai Zhaoxin Semiconductor Co., Ltd.](https://en.wikipedia.org/wiki/Zhaoxin) as the
        /// CPU vendor
        Zhaoxin    ("  Shanghai  ")                 = "Shanghai Zhaoxin Semiconductor Co., Ltd.",

        /// This variant indicates the Chinese manufacturing company Hygon Information Technology
        /// Co., Ltd. as the CPU vendor
        Hygon      ("HygonGenuine")                 = "Hygon Information Technology Co., Ltd.",

        /// This variant indicates the Russian microprocessor company
        /// [Moscow Center of SPARC Technologies](https://en.wikipedia.org/wiki/MCST) as the CPU
        /// vendor
        Elbrus     ("E2K MACHINE ")                 = "Moscow Center of SPARC Technologies",

        /// This variant indicates that the system is emulated by
        /// [Quick Emulator (QEMU)](https://en.wikipedia.org/wiki/QEMU)
        QEMU       ("TCGTCGTCGTCG")                 = "Quick Emulator (QEMU)",

        /// This variant indicates that the system is virtualized by the Linux module
        /// [KVM (Kernel-based Virtual Machine)](https://en.wikipedia.org/wiki/Kernel-based_Virtual_Machine)
        KVM        (" KVMKVMKVM  ")                 = "Kernel-based Virtual Machine",

        /// This variant indicates that the system is virtualized by one of the virtualization
        /// products by [VMware, Inc.](https://en.wikipedia.org/wiki/Kernel-based_Virtual_Machine)
        VMware     ("VMwareVMware")                 = "VMware",

        /// This variant indicates that the system is virtualized by
        /// [VirtualBox](https://en.wikipedia.org/wiki/VirtualBox)
        VirtualBox ("VBoxVBoxVBox")                 = "VirtualBox",

        /// This variant indicates that the system is virtualized by the
        /// [Xen Hypervisor](https://en.wikipedia.org/wiki/Xen)
        Xen        ("XenVMMXenVMM")                 = "Xen",

        /// This variant indicates that the system is virtualized by the
        /// [Microsoft Hyper-V Hypervisor](https://en.wikipedia.org/wiki/Hyper-V)
        HyperV     ("Microsoft Hv")                 = "Microsoft Hyper-V",

        /// This variant indicates that the system is virtualized by
        /// [Parallels](https://en.wikipedia.org/wiki/Parallels_(company))
        Parallels  (" prl hyperv ", " lrpepyh vr ") = "Parallels",

        /// This variant indicates that the system is virtualized by
        /// [bhyve Hypervisor](https://en.wikipedia.org/wiki/Bhyve)
        BHYVE      ("bhyve bhyve ")                 = "bhyve",

        /// This variant indicates that the system is virtualized by the QNX Hypervisor
        QNX        (" QNXQVMBSQG ")                 = "QNX"
    }
}

features! {
    #[allow(non_camel_case_types)]
    #[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
    pub enum CPUFeature {
        /// SSE3 (Streaming SIMD Extensions 3) is a instruction set extension for IA-32 that
        /// provides instructions for 512-bit registers and operation with 512-bit data in one
        /// instruction. SSE3 is used to optimize algorithm for bigger data.
        ///
        /// # See also
        /// - [SSE3](https://en.wikipedia.org/wiki/SSE3) by [Wikipedia](https://en.wikipedia.org)
        SSE3        (ecx, "SSE3") = 1 << 0,

        /// Carray-Less Multiplication (PCLMUL) is a instruction set extension for IA-32 that
        /// is an extension for AES-NI und improves the performance of encryption algorithm
        /// implementations, especially for algorithms which are using the Galois-Field
        /// multiplication.
        ///
        /// # See also
        /// - [CLMUL instruction set](https://en.wikipedia.org/wiki/CLMUL_instruction_set) by
        /// [Wikipedia](https://en.wikipedia.org)
        PCLMUL      (ecx, "Carry-Less Multiplication") = 1 << 1,
        DTES64      (ecx, "64-Bit Debug Store") = 1 << 2,

        /// This feature implements the instructions `MONITOR` and `MWAIT`. These instructions are
        /// used to improve the performance of multi-core systems. `MONITOR` is used to mark a
        /// memory section and `MWAIT` tells to processor to wait for changes in the section, that
        /// was selected before. These instructions also can prevent spin locks.
        MONITOR     (ecx, "MONITOR and MWAIT instructions") = 1 << 3,
        DS_CPL      (ecx, "CPL-qualified Debug Store") = 1 << 4,

        /// The Virtual Machine Extensions (VMX) are a feature for IA-32 processors that make
        /// Hardware virtualization possible. So a single processor can execute multiple virtual
        /// machines. AMD has for this the AMD Virtualization (AMD-V) and Intel has the Intel
        /// Virtualization (VT-x for that)
        ///
        /// - The processor implements the VMX Root Mode (In this mode, the host system is running)
        /// and the VMX Non-Root Mode (In this mode, the guest system is running)
        /// - The processor implements the VMCS (Virtual Machine Control Structure) to configure
        /// a virtual machine.
        /// - The processor implements VM-Exit (Jump into the Root Mode) and VM-Entry (Jump into
        /// the Non-Root) mode.
        /// - The processor implements the MM (Virtual Machine Monitor) to make the development
        /// of hypervisors more simpler.
        ///
        /// # See also
        /// - [x86 Virtualization](https://en.wikipedia.org/wiki/X86_virtualization#Hardware-assisted_virtualization)
        /// by [Wikipedia](https://en.wikipedia.org)
        VMX         (ecx, "Virtual Machine Extensions") = 1 << 5,

        /// The Safer Mode Extensions are part of the Intel Trusted Execution Technology, that is
        /// running on the Intel Management Engine (Ring -2).
        ///
        /// # See also
        /// - [Intel Management Engine](https://en.wikipedia.org/wiki/Intel_Management_Engine#Firmware)
        /// by [Wikipedia](https://en.wikipedia.org)
        /// - [Trusted Execution Technology](https://en.wikipedia.org/wiki/Trusted_Execution_Technology)
        /// by [Wikipedia](https://en.wikipedia.org)
        SMX         (ecx, "Safer Mode Extensions") = 1 << 6,

        /// The Intel EIST (Enhanced Intel SpeedStep Technology) is a technology, that was developed
        /// by the Intel Corporation to improve the energy efficiency of the processors.
        /// - Dynamic Clock Frequency Adjustment
        /// - Voltage Adjustment
        /// - Dynamic Frequency and Voltage Switching
        ///
        /// # See also
        /// - [SpeedStep](https://en.wikipedia.org/wiki/SpeedStep) by
        /// [Wikipedia](https://en.wikipedia.org)
        EST         (ecx, "Enhanced SpeedStep") = 1 << 7,

        /// The Thermal Monitor is a feature that was invented by the Intel Corporation to regulate
        /// and monitor the temperature of the processor. This also provides the auto shutdown
        /// before the processor takes damage by the heat.
        ///
        /// # See also
        /// - [Thermal Monitor 2](https://en.wikipedia.org/wiki/Thermal_Monitor_2) by
        /// [Wikipedia](https://en.wikipedia.org)
        TM2         (ecx, "Thermal Monitor 2") = 1 << 8,

        /// Supplemental Streaming SIMD Extensions 3 (SSSE3) is an extension for SSE3. SSSE3 adds
        /// some instructions to improve SSE3.
        ///
        /// # See also
        /// - [SSSE3](https://en.wikipedia.org/wiki/SSSE3) by [Wikipedia](https://en.wikipedia.org)
        SSSE3       (ecx, "Supplemental SSE3") = 1 << 9,

        /// L1 Context ID is a mechanism that can associate the content of the L1 Cache to a
        /// specific context or a specific task.
        CID         (ecx, "L1 Context ID") = 1 << 10,

        /// Silicon Debug interface (SDBG) is a mechanism to identify and fix flaws in software
        /// or hardware systems.
        SDBG        (ecx, "Silicon Debug Interface") = 1 << 11,

        /// Fused Multiply-Add (FMA3) is a arithmetic instruction to calculate a multiplication and
        /// an addition with one step. That can improve the performance of a calculation. This
        /// feature is often used in Machine Larning or other intensive tasks.
        ///
        /// # See also
        /// - [FMA Instruction Set](https://en.wikipedia.org/wiki/FMA_instruction_set) by
        /// [Wikipedia](https://en.wikipedia.org)
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
