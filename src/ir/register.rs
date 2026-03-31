/// All supported architectures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Arch {
    X86_16,
    X86_32,
    X86_64,
    Arm64,
    RiscV64,
    Mips,
}

impl Arch {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "x86_16" | "x86-16" | "i8086" => Some(Arch::X86_16),
            "x86_32" | "x86-32" | "i386" | "x86" => Some(Arch::X86_32),
            "x86_64" | "x86-64" | "amd64" => Some(Arch::X86_64),
            "arm64" | "aarch64" => Some(Arch::Arm64),
            "riscv64" | "riscv" | "rv64" => Some(Arch::RiscV64),
            "mips" | "mips32" => Some(Arch::Mips),
            _ => None,
        }
    }
}

/// Size qualifier for operands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Size {
    Byte,   // 8-bit
    Word,   // 16-bit
    Dword,  // 32-bit
    Qword,  // 64-bit
    Xmmword, // 128-bit SSE
    Ymmword, // 256-bit AVX
    Zmmword, // 512-bit AVX-512
}

impl Size {
    pub fn bits(&self) -> u32 {
        match self {
            Size::Byte => 8,
            Size::Word => 16,
            Size::Dword => 32,
            Size::Qword => 64,
            Size::Xmmword => 128,
            Size::Ymmword => 256,
            Size::Zmmword => 512,
        }
    }
}

/// Unified register representation across all architectures
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Register {
    // x86-64 General Purpose (64-bit)
    Rax, Rbx, Rcx, Rdx, Rsi, Rdi, Rsp, Rbp,
    R8, R9, R10, R11, R12, R13, R14, R15,
    // x86 General Purpose (32-bit)
    Eax, Ebx, Ecx, Edx, Esi, Edi, Esp, Ebp,
    R8d, R9d, R10d, R11d, R12d, R13d, R14d, R15d,
    // x86 (16-bit)
    Ax, Bx, Cx, Dx, Si, Di, Sp, Bp,
    // x86 (8-bit)
    Al, Ah, Bl, Bh, Cl, Ch, Dl, Dh,
    // x86 Segment
    Cs, Ds, Es, Fs, Gs, Ss,
    // x86 SSE
    Xmm(u8),  // xmm0..xmm15
    // x86 AVX
    Ymm(u8),  // ymm0..ymm15
    // x86 AVX-512
    Zmm(u8),  // zmm0..zmm31

    // ARM64 General Purpose
    X(u8),    // x0..x30
    W(u8),    // w0..w30 (32-bit view)
    Xzr,      // zero register (64-bit)
    Wzr,      // zero register (32-bit)
    ArmSp,    // stack pointer ARM
    // ARM64 SIMD/FP
    V(u8),    // v0..v31

    // RISC-V
    Rv(u8),   // x0..x31
    Fv(u8),   // f0..f31 (floating point)

    // MIPS
    Mips(u8), // $0..$31
    MipsF(u8), // $f0..$f31
}

impl Register {
    pub fn from_str(s: &str) -> Option<Self> {
        let s_lower = s.to_lowercase();
        match s_lower.as_str() {
            // x86-64
            "rax" => Some(Register::Rax), "rbx" => Some(Register::Rbx),
            "rcx" => Some(Register::Rcx), "rdx" => Some(Register::Rdx),
            "rsi" => Some(Register::Rsi), "rdi" => Some(Register::Rdi),
            "rsp" => Some(Register::Rsp), "rbp" => Some(Register::Rbp),
            "r8" => Some(Register::R8), "r9" => Some(Register::R9),
            "r10" => Some(Register::R10), "r11" => Some(Register::R11),
            "r12" => Some(Register::R12), "r13" => Some(Register::R13),
            "r14" => Some(Register::R14), "r15" => Some(Register::R15),
            // x86-32
            "eax" => Some(Register::Eax), "ebx" => Some(Register::Ebx),
            "ecx" => Some(Register::Ecx), "edx" => Some(Register::Edx),
            "esi" => Some(Register::Esi), "edi" => Some(Register::Edi),
            "esp" => Some(Register::Esp), "ebp" => Some(Register::Ebp),
            "r8d" => Some(Register::R8d), "r9d" => Some(Register::R9d),
            "r10d" => Some(Register::R10d), "r11d" => Some(Register::R11d),
            "r12d" => Some(Register::R12d), "r13d" => Some(Register::R13d),
            "r14d" => Some(Register::R14d), "r15d" => Some(Register::R15d),
            // x86-16
            "ax" => Some(Register::Ax), "bx" => Some(Register::Bx),
            "cx" => Some(Register::Cx), "dx" => Some(Register::Dx),
            "si" => Some(Register::Si), "di" => Some(Register::Di),
            "sp" => Some(Register::Sp), "bp" => Some(Register::Bp),
            // x86-8
            "al" => Some(Register::Al), "ah" => Some(Register::Ah),
            "bl" => Some(Register::Bl), "bh" => Some(Register::Bh),
            "cl" => Some(Register::Cl), "ch" => Some(Register::Ch),
            "dl" => Some(Register::Dl), "dh" => Some(Register::Dh),
            // Segments
            "cs" => Some(Register::Cs), "ds" => Some(Register::Ds),
            "es" => Some(Register::Es), "fs" => Some(Register::Fs),
            "gs" => Some(Register::Gs), "ss" => Some(Register::Ss),
            // ARM64 special
            "xzr" => Some(Register::Xzr),
            "wzr" => Some(Register::Wzr),
            _ => {
                // xmm0..xmm15
                if let Some(n) = s_lower.strip_prefix("xmm") {
                    return n.parse::<u8>().ok().filter(|&v| v <= 15).map(Register::Xmm);
                }
                // ymm0..ymm15
                if let Some(n) = s_lower.strip_prefix("ymm") {
                    return n.parse::<u8>().ok().filter(|&v| v <= 15).map(Register::Ymm);
                }
                // zmm0..zmm31
                if let Some(n) = s_lower.strip_prefix("zmm") {
                    return n.parse::<u8>().ok().filter(|&v| v <= 31).map(Register::Zmm);
                }
                // ARM64 x0..x30
                if let Some(n) = s_lower.strip_prefix('x') {
                    if let Ok(v) = n.parse::<u8>() {
                        if v <= 30 { return Some(Register::X(v)); }
                    }
                }
                // ARM64 w0..w30
                if let Some(n) = s_lower.strip_prefix('w') {
                    if let Ok(v) = n.parse::<u8>() {
                        if v <= 30 { return Some(Register::W(v)); }
                    }
                }
                // ARM64 v0..v31
                if let Some(n) = s_lower.strip_prefix('v') {
                    if let Ok(v) = n.parse::<u8>() {
                        if v <= 31 { return Some(Register::V(v)); }
                    }
                }
                None
            }
        }
    }

    /// Get the display name of this register in its native syntax
    pub fn name(&self) -> String {
        match self {
            Register::Rax => "rax".into(), Register::Rbx => "rbx".into(),
            Register::Rcx => "rcx".into(), Register::Rdx => "rdx".into(),
            Register::Rsi => "rsi".into(), Register::Rdi => "rdi".into(),
            Register::Rsp => "rsp".into(), Register::Rbp => "rbp".into(),
            Register::R8 => "r8".into(), Register::R9 => "r9".into(),
            Register::R10 => "r10".into(), Register::R11 => "r11".into(),
            Register::R12 => "r12".into(), Register::R13 => "r13".into(),
            Register::R14 => "r14".into(), Register::R15 => "r15".into(),
            Register::Eax => "eax".into(), Register::Ebx => "ebx".into(),
            Register::Ecx => "ecx".into(), Register::Edx => "edx".into(),
            Register::Esi => "esi".into(), Register::Edi => "edi".into(),
            Register::Esp => "esp".into(), Register::Ebp => "ebp".into(),
            Register::R8d => "r8d".into(), Register::R9d => "r9d".into(),
            Register::R10d => "r10d".into(), Register::R11d => "r11d".into(),
            Register::R12d => "r12d".into(), Register::R13d => "r13d".into(),
            Register::R14d => "r14d".into(), Register::R15d => "r15d".into(),
            Register::Ax => "ax".into(), Register::Bx => "bx".into(),
            Register::Cx => "cx".into(), Register::Dx => "dx".into(),
            Register::Si => "si".into(), Register::Di => "di".into(),
            Register::Sp => "sp".into(), Register::Bp => "bp".into(),
            Register::Al => "al".into(), Register::Ah => "ah".into(),
            Register::Bl => "bl".into(), Register::Bh => "bh".into(),
            Register::Cl => "cl".into(), Register::Ch => "ch".into(),
            Register::Dl => "dl".into(), Register::Dh => "dh".into(),
            Register::Cs => "cs".into(), Register::Ds => "ds".into(),
            Register::Es => "es".into(), Register::Fs => "fs".into(),
            Register::Gs => "gs".into(), Register::Ss => "ss".into(),
            Register::Xmm(n) => format!("xmm{}", n),
            Register::Ymm(n) => format!("ymm{}", n),
            Register::Zmm(n) => format!("zmm{}", n),
            Register::X(n) => format!("x{}", n),
            Register::W(n) => format!("w{}", n),
            Register::Xzr => "xzr".into(),
            Register::Wzr => "wzr".into(),
            Register::ArmSp => "sp".into(),
            Register::V(n) => format!("v{}", n),
            Register::Rv(n) => format!("x{}", n),
            Register::Fv(n) => format!("f{}", n),
            Register::Mips(n) => format!("${}", n),
            Register::MipsF(n) => format!("$f{}", n),
        }
    }

    /// Get the size of this register
    pub fn size(&self) -> Size {
        match self {
            Register::Al | Register::Ah | Register::Bl | Register::Bh |
            Register::Cl | Register::Ch | Register::Dl | Register::Dh => Size::Byte,

            Register::Ax | Register::Bx | Register::Cx | Register::Dx |
            Register::Si | Register::Di | Register::Sp | Register::Bp => Size::Word,

            Register::Eax | Register::Ebx | Register::Ecx | Register::Edx |
            Register::Esi | Register::Edi | Register::Esp | Register::Ebp |
            Register::R8d | Register::R9d | Register::R10d | Register::R11d |
            Register::R12d | Register::R13d | Register::R14d | Register::R15d |
            Register::W(_) | Register::Wzr => Size::Dword,

            Register::Rax | Register::Rbx | Register::Rcx | Register::Rdx |
            Register::Rsi | Register::Rdi | Register::Rsp | Register::Rbp |
            Register::R8 | Register::R9 | Register::R10 | Register::R11 |
            Register::R12 | Register::R13 | Register::R14 | Register::R15 |
            Register::X(_) | Register::Xzr | Register::ArmSp |
            Register::Rv(_) | Register::Fv(_) => Size::Qword,

            Register::Cs | Register::Ds | Register::Es | Register::Fs |
            Register::Gs | Register::Ss => Size::Word,

            Register::Xmm(_) => Size::Xmmword,
            Register::Ymm(_) => Size::Ymmword,
            Register::Zmm(_) => Size::Zmmword,
            Register::V(_) => Size::Xmmword,
            Register::Mips(_) | Register::MipsF(_) => Size::Dword,
        }
    }
}
