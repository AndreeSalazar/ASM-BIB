/// Supported x86 architectures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Arch {
    X86_16,
    X86_32,
    X86_64,
}

impl Arch {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "x86_16" | "x86-16" | "i8086" => Some(Arch::X86_16),
            "x86_32" | "x86-32" | "i386" | "x86" => Some(Arch::X86_32),
            "x86_64" | "x86-64" | "amd64" => Some(Arch::X86_64),
            _ => None,
        }
    }
}

/// Size qualifier for operands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Size {
    Byte,    // 8-bit
    Word,    // 16-bit
    Dword,   // 32-bit
    Qword,   // 64-bit
    Tbyte,   // 80-bit (x87 FPU extended precision)
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
            Size::Tbyte => 80,
            Size::Xmmword => 128,
            Size::Ymmword => 256,
            Size::Zmmword => 512,
        }
    }
}

/// x86 register representation (16/32/64-bit modes)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Register {
    // x86-64 General Purpose (64-bit)
    Rax, Rbx, Rcx, Rdx, Rsi, Rdi, Rsp, Rbp,
    R8, R9, R10, R11, R12, R13, R14, R15,
    // x86 General Purpose (32-bit)
    Eax, Ebx, Ecx, Edx, Esi, Edi, Esp, Ebp,
    R8d, R9d, R10d, R11d, R12d, R13d, R14d, R15d,
    // x86 General Purpose (16-bit)
    Ax, Bx, Cx, Dx, Si, Di, Sp, Bp,
    // x86-64 General Purpose (16-bit, extended)
    R8w, R9w, R10w, R11w, R12w, R13w, R14w, R15w,
    // x86 General Purpose (8-bit)
    Al, Ah, Bl, Bh, Cl, Ch, Dl, Dh,
    // x86-64 General Purpose (8-bit, REX-required)
    Spl, Bpl, Sil, Dil,
    R8b, R9b, R10b, R11b, R12b, R13b, R14b, R15b,
    // x86 Segment registers
    Cs, Ds, Es, Fs, Gs, Ss,
    // Control registers (Ring 0)
    Cr0, Cr2, Cr3, Cr4,
    // Debug registers (Ring 0/1)
    Dr0, Dr1, Dr2, Dr3, Dr6, Dr7,
    // SSE registers (128-bit)
    Xmm(u8),  // xmm0..xmm15
    // AVX registers (256-bit)
    Ymm(u8),  // ymm0..ymm15
    // AVX-512 registers (512-bit)
    Zmm(u8),  // zmm0..zmm31
}

impl Register {
    pub fn from_str(s: &str) -> Option<Self> {
        let s_lower = s.to_lowercase();
        match s_lower.as_str() {
            // x86-64 (64-bit)
            "rax" => Some(Register::Rax), "rbx" => Some(Register::Rbx),
            "rcx" => Some(Register::Rcx), "rdx" => Some(Register::Rdx),
            "rsi" => Some(Register::Rsi), "rdi" => Some(Register::Rdi),
            "rsp" => Some(Register::Rsp), "rbp" => Some(Register::Rbp),
            "r8" => Some(Register::R8), "r9" => Some(Register::R9),
            "r10" => Some(Register::R10), "r11" => Some(Register::R11),
            "r12" => Some(Register::R12), "r13" => Some(Register::R13),
            "r14" => Some(Register::R14), "r15" => Some(Register::R15),
            // x86 (32-bit)
            "eax" => Some(Register::Eax), "ebx" => Some(Register::Ebx),
            "ecx" => Some(Register::Ecx), "edx" => Some(Register::Edx),
            "esi" => Some(Register::Esi), "edi" => Some(Register::Edi),
            "esp" => Some(Register::Esp), "ebp" => Some(Register::Ebp),
            "r8d" => Some(Register::R8d), "r9d" => Some(Register::R9d),
            "r10d" => Some(Register::R10d), "r11d" => Some(Register::R11d),
            "r12d" => Some(Register::R12d), "r13d" => Some(Register::R13d),
            "r14d" => Some(Register::R14d), "r15d" => Some(Register::R15d),
            // x86 (16-bit)
            "ax" => Some(Register::Ax), "bx" => Some(Register::Bx),
            "cx" => Some(Register::Cx), "dx" => Some(Register::Dx),
            "si" => Some(Register::Si), "di" => Some(Register::Di),
            "sp" => Some(Register::Sp), "bp" => Some(Register::Bp),
            // x86-64 (16-bit extended)
            "r8w" => Some(Register::R8w), "r9w" => Some(Register::R9w),
            "r10w" => Some(Register::R10w), "r11w" => Some(Register::R11w),
            "r12w" => Some(Register::R12w), "r13w" => Some(Register::R13w),
            "r14w" => Some(Register::R14w), "r15w" => Some(Register::R15w),
            // x86 (8-bit)
            "al" => Some(Register::Al), "ah" => Some(Register::Ah),
            "bl" => Some(Register::Bl), "bh" => Some(Register::Bh),
            "cl" => Some(Register::Cl), "ch" => Some(Register::Ch),
            "dl" => Some(Register::Dl), "dh" => Some(Register::Dh),
            // 8-bit REX registers
            "spl" => Some(Register::Spl), "bpl" => Some(Register::Bpl),
            "sil" => Some(Register::Sil), "dil" => Some(Register::Dil),
            "r8b" => Some(Register::R8b), "r9b" => Some(Register::R9b),
            "r10b" => Some(Register::R10b), "r11b" => Some(Register::R11b),
            "r12b" => Some(Register::R12b), "r13b" => Some(Register::R13b),
            "r14b" => Some(Register::R14b), "r15b" => Some(Register::R15b),
            // Segment registers
            "cs" => Some(Register::Cs), "ds" => Some(Register::Ds),
            "es" => Some(Register::Es), "fs" => Some(Register::Fs),
            "gs" => Some(Register::Gs), "ss" => Some(Register::Ss),
            // Control registers
            "cr0" => Some(Register::Cr0), "cr2" => Some(Register::Cr2),
            "cr3" => Some(Register::Cr3), "cr4" => Some(Register::Cr4),
            // Debug registers
            "dr0" => Some(Register::Dr0), "dr1" => Some(Register::Dr1),
            "dr2" => Some(Register::Dr2), "dr3" => Some(Register::Dr3),
            "dr6" => Some(Register::Dr6), "dr7" => Some(Register::Dr7),
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
            Register::R8w => "r8w".into(), Register::R9w => "r9w".into(),
            Register::R10w => "r10w".into(), Register::R11w => "r11w".into(),
            Register::R12w => "r12w".into(), Register::R13w => "r13w".into(),
            Register::R14w => "r14w".into(), Register::R15w => "r15w".into(),
            Register::Al => "al".into(), Register::Ah => "ah".into(),
            Register::Bl => "bl".into(), Register::Bh => "bh".into(),
            Register::Cl => "cl".into(), Register::Ch => "ch".into(),
            Register::Dl => "dl".into(), Register::Dh => "dh".into(),
            Register::Spl => "spl".into(), Register::Bpl => "bpl".into(),
            Register::Sil => "sil".into(), Register::Dil => "dil".into(),
            Register::R8b => "r8b".into(), Register::R9b => "r9b".into(),
            Register::R10b => "r10b".into(), Register::R11b => "r11b".into(),
            Register::R12b => "r12b".into(), Register::R13b => "r13b".into(),
            Register::R14b => "r14b".into(), Register::R15b => "r15b".into(),
            Register::Cs => "cs".into(), Register::Ds => "ds".into(),
            Register::Es => "es".into(), Register::Fs => "fs".into(),
            Register::Gs => "gs".into(), Register::Ss => "ss".into(),
            Register::Cr0 => "cr0".into(), Register::Cr2 => "cr2".into(),
            Register::Cr3 => "cr3".into(), Register::Cr4 => "cr4".into(),
            Register::Dr0 => "dr0".into(), Register::Dr1 => "dr1".into(),
            Register::Dr2 => "dr2".into(), Register::Dr3 => "dr3".into(),
            Register::Dr6 => "dr6".into(), Register::Dr7 => "dr7".into(),
            Register::Xmm(n) => format!("xmm{}", n),
            Register::Ymm(n) => format!("ymm{}", n),
            Register::Zmm(n) => format!("zmm{}", n),
        }
    }

    /// Get the size of this register
    pub fn size(&self) -> Size {
        match self {
            Register::Al | Register::Ah | Register::Bl | Register::Bh |
            Register::Cl | Register::Ch | Register::Dl | Register::Dh |
            Register::Spl | Register::Bpl | Register::Sil | Register::Dil |
            Register::R8b | Register::R9b | Register::R10b | Register::R11b |
            Register::R12b | Register::R13b | Register::R14b | Register::R15b => Size::Byte,

            Register::Ax | Register::Bx | Register::Cx | Register::Dx |
            Register::Si | Register::Di | Register::Sp | Register::Bp |
            Register::R8w | Register::R9w | Register::R10w | Register::R11w |
            Register::R12w | Register::R13w | Register::R14w | Register::R15w |
            Register::Cs | Register::Ds | Register::Es | Register::Fs |
            Register::Gs | Register::Ss => Size::Word,

            Register::Eax | Register::Ebx | Register::Ecx | Register::Edx |
            Register::Esi | Register::Edi | Register::Esp | Register::Ebp |
            Register::R8d | Register::R9d | Register::R10d | Register::R11d |
            Register::R12d | Register::R13d | Register::R14d | Register::R15d => Size::Dword,

            Register::Rax | Register::Rbx | Register::Rcx | Register::Rdx |
            Register::Rsi | Register::Rdi | Register::Rsp | Register::Rbp |
            Register::R8 | Register::R9 | Register::R10 | Register::R11 |
            Register::R12 | Register::R13 | Register::R14 | Register::R15 => Size::Qword,

            Register::Cr0 | Register::Cr2 | Register::Cr3 | Register::Cr4 => Size::Qword,
            Register::Dr0 | Register::Dr1 | Register::Dr2 | Register::Dr3 |
            Register::Dr6 | Register::Dr7 => Size::Qword,

            Register::Xmm(_) => Size::Xmmword,
            Register::Ymm(_) => Size::Ymmword,
            Register::Zmm(_) => Size::Zmmword,
        }
    }
}
