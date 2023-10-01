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
    /// - [CPUVendor::UMC] (Semiconductor Company) - United Microelectronics Corporation
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

        /// This feature implements the CMPXCHG16B (Compare and exchange 86 bytes), that is used to
        /// provide atomicity in operations on memory. This instruction is particularly used in
        /// critical code sections, where the integrity must be ensured. So with this instruction
        /// the programmer can avoid bugs like Race Conditions.
        CX16        (ecx, "CMPXCHG16B instruction") = 1 << 13,
        XTPR        (ecx, "Can disable sending task priority messages") = 1 << 14,
        PDCM        (ecx, "Prefmon & Debug Capability") = 1 << 15,
        PCID        (ecx, "Process Context Identifiers") = 1 << 17,

        /// This feature provides the functionality, that DMA (Direct Memory Access) operations can
        /// write the cache without involving the CPU. That improves the efficiency of DMA
        /// operations.
        DCA         (ecx, "Direct Cache Access for DMA writes") = 1 << 18,

        /// SSE 4.1 is an extension for the x86 instruction set (developed by Intel) that adds
        /// string and text processing instructions, the CRC32 instruction and the Dot-Product
        /// instruction to the available instruction set.
        ///
        /// # See also
        /// - [SSE4](https://en.wikipedia.org/wiki/SSE4#SSE4.1) by
        /// [Wikipedia](https://en.wikipedia.org)
        SSE4_1      (ecx, "SSE4.1 instructions") = 1 << 19,

        /// SSE 4.2 is an extension for the s86 instruction set (developed by Intel) that adds
        /// string compare instructions, pattern searching instructions, gather and scatter
        /// instructions.
        ///
        /// # See also
        /// - [SSE4](https://en.wikipedia.org/wiki/SSE4#SSE4.2) by
        /// [Wikipedia](https://en.wikipedia.org)
        SSE4_2      (ecx, "SSE4.2 instructions") = 1 << 20,

        /// Enhanced APIC is a extension for the APIC (Advanced Programmable Interrupt Controller)
        /// in modern processors. This extension provides the functionality to add more interrupt
        /// vectors, a direct register-addressing, more scalability and more security functionality.
        ///
        /// # See also
        /// - [Advanced Programmable Interrupt Controller](https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller#X2APIC)
        /// by [Wikipedia](https://en.wikipedia.org)
        X2APIC      (ecx, "x2APIC (enhanced APIC)") = 1 << 21,

        /// The MOVBE instruction is used to swap the byte order of a 16-bit or 32-bit data
        /// operand. It's useful in situations where data is begin transferred between systems
        /// with different endianness.
        MOVBE       (ecx, "MOVBE instruction") = 1 << 22,

        /// This instruction counts all bits in 32-bit or 64-bit data, which are set to 1. The result
        /// is stored in the destination register.
        POPCNT      (ecx, "POPCNT instruction") = 1 << 23,

        /// With this feature, you can configure the APIC to execute a one-shot operation using a
        /// TSC-deadline to control the timing of the event.
        TSCDeadline (ecx, "APIC implements one-shot operation using a TSC deadline value") = 1 << 24,

        /// This feature provides hardware acceleration for the Advanced Encryption Standard/AES
        /// (Rijndael) encryption algorithm. With these instruction you can execute all rounds of
        /// AES encryption and decryption. You also have assistant instruction to expand or generate
        /// a key.
        ///
        /// # See also
        /// - [AES Instruction Set](https://en.wikipedia.org/wiki/AES_instruction_set) by
        /// [Wikipedia](https://en.wikipedia.org)
        AES         (ecx, "Hardware-accelerated AES Instruction Set") = 1 << 25,

        /// This feature provides four instructions to restore and save extended processor
        /// states. These instruction can be used by Operating Systems during context
        /// switching.
        XSAVE       (ecx, "Extensible processor state restore instructions") = 1 << 26,

        /// This feature shows that the operating system has the capability to support this
        /// instruction.
        OSXSAVE     (ecx, "XSAVE enabled by OS") = 1 << 27,

        /// The Advanced Vector Extensions (AVX) provides SIMD instructions for operations on
        /// 256-bit registers.
        ///
        /// # See also
        /// - [Advanced Vector Extensions](https://en.wikipedia.org/wiki/Advanced_Vector_Extensions)
        /// by [Wikipedia](https://en.wikipedia.org)
        AVX         (ecx, "Advanced Vector Extensions (256-bit SIMD)") = 1 << 28,

        /// This feature provides the functionality to convert 32-bit or 64-bit floating-point
        /// numbers to 16-bit floating point numbers. That can provide a better performance for
        /// Neural Network applications. It can also reduce the memory usage of the application.
        F16C        (ecx, "Floating-point conversion instructions to/from FP16 format") = 1 << 29,

        /// This feature is an extension for the x86 instruction set, that was invented by
        /// Intel. This feature can be used to generate cryptographically-secure numbers. On Intel
        /// processors this instruction is part of the Intel Digital Random Number Generator (DRNG)
        ///
        /// # See also
        /// - [RDRAND](https://en.wikipedia.org/wiki/RDRAND) by
        /// [Wikipedia](https://en.wikipedia.org)
        RDRAND      (ecx, "RDRAND feature") = 1 << 30,

        /// This feature indicates the presence of a Hypervisor in this environment (This bit is
        /// always zero on physical CPUs)
        HYPERVISOR  (ecx, "Hypervisor is present") = 1 << 31,

        /// This feature indicates that the system is supporting a FPU. All modern CPUs are providing
        /// a on-board FPU Support.
        FPU         (edx, "Onboard x87 FPU") = 1 << 0,

        /// This feature provides the functionality to execute old 16-bit x86 applications in a
        /// virtual environment.
        ///
        /// # See also
        /// - [Virtual 8086 Mode Extensions](https://en.wikipedia.org/wiki/Virtual_8086_mode#Virtual_8086_mode_enhancements_.28VME.29)
        /// by [Wikipedia](https://en.wikipedia.org)
        VME         (edx, "Virtual 8086 Mode Extensions") = 1 << 1,

        /// This feature provides the functionality for hardware support for the I/O management under
        /// virtual environments on the system.
        DE          (edx, "Debugging Extensions") = 1 << 2,

        /// This feature provides the functionality to request pages that are larger than the
        /// traditional 4 KiB bytes.
        ///
        /// # See also
        /// - [Page Size Extension](https://en.wikipedia.org/wiki/Page_Size_Extension) by
        /// [Wikipedia](https://en.wikipedia.org)
        PSE         (edx, "Page Size Extension (4MB pages)") = 1 << 3,

        /// This feature provides the `RDTSC` instruction to read the Time Stamp Counter of the
        /// processor.
        ///
        /// # See also
        /// - [Time Stamp Counter](https://en.wikipedia.org/wiki/Time_Stamp_Counter) by
        /// [Wikipedia](https://en.wikipedia.org)
        TSC         (edx, "Time Stamp Counter and RDTSC instruction") = 1 << 4,

        /// This feature allows the programmer to use model-specific register of the CPU. That can
        /// be used for vendor-specific operations, monitoring or debugging.
        ///
        /// # See also
        /// - [Model-specific register](https://en.wikipedia.org/wiki/Model-specific_register) by
        /// [Wikipedia](https://en.wikipedia.org)
        MSR         (edx, "Model-Specific Registers and RDMSR/WRMSR instructions") = 1 << 5,

        /// This feature that expands the physical address space from 32 bit to 36 bit. Before
        /// that feature, the physical address space has a size of 4 gigabytes. With this feature
        /// the system is allowed to address 64 gigabytes of memory.
        ///
        /// # See also
        /// - [Physical Address Extension](https://en.wikipedia.org/wiki/Physical_Address_Extension)
        /// by [Wikipedia](https://en.wikipedia.org)
        PAE         (edx, "Physical Address Extension") = 1 << 6,

        /// This feature allows the system to recognize and handle machine check exceptions over the
        /// IDT. So we can use the exception [crate::idt::Exception::MachineCheck] to handle them.
        ///
        /// # See also
        /// - [Machine-Check Exception](https://en.wikipedia.org/wiki/Machine-check_exception) by
        /// [Wikipedia](https://en.wikipedia.org)
        MCE         (edx, "Machine-Check Exception") = 1 << 7,

        /// This feature implements the CMPXCHG1B (Compare and exchange 8 bytes), that is used to
        /// provide atomicity in operations on memory. This instruction is particularly used in
        /// critical code sections, where the integrity must be ensured. So with this instruction
        /// the programmer can avoid bugs like Race Conditions.
        CX8         (edx, "CMPXCHG8B instruction") = 1 << 8,

        /// This feature indicates that the system is supporting a APIC. All modern CPUs are
        /// providing a on-board APIC Support.
        ///
        /// # See also
        /// - [Advanced Programmable Interrupt Controller](https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller)
        /// by [Wikipedia](https://en.wikipedia.org)
        APIC        (edx, "Onboard APIC") = 1 << 9,

        /// This feature provides two x86 instructions to make system calls more efficient. With
        /// the `SYSENTER` instruction, you can change from the user mode to the kernel mode. With
        /// the `SYSEXIT` instruction, you can change back from the kernel mode to the user mode.
        SEP         (edx, "SYSENTER and SYSEXIT fast System Call instuctions") = 1 << 11,

        /// This feature provides a few registers to configure and control the access of specific
        /// memory segments. So you can control the memory caching or memory types like Write-Back
        /// or Write-Through.
        ///
        /// # See also
        /// - [Memory Type Range Regiseter](https://en.wikipedia.org/wiki/Memory_type_range_register)
        /// by [Wikipedia](https://en.wikipedia.org)
        MTRR        (edx, "Memory Type Range Registers") = 1 << 12,

        /// This feature provides a control bit in the CR4 register to control the paging mechanism
        /// in the processor. So you can use global pages (Global pages are accessible by all
        /// processors)
        PGE         (edx, "Page Global Enable bit") = 1 << 13,

        /// The machine check architecture are a few mechanisms and specifications in the x86
        /// architecture to detect machine check errors. With this feature, the CPU tries to
        /// improve the reliability of computer systems.
        ///
        /// # See also
        /// - [Machine Check Architecture](https://en.wikipedia.org/wiki/Machine_Check_Architecture)
        /// by [Wikipedia](https://en.wikipedia.org)
        MCA         (edx, "Machine Check Architecture") = 1 << 14,

        /// This feature implements a few instructions ike CMOV to conditionally move data from one
        /// register to the other register.
        CMOV        (edx, "Conditional move instructions") = 1 << 15,

        /// This feature is a functionality in the x86 architecture to set attributes for memory
        /// pages. So we can control the characteristics of pages in virtual memory.
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
