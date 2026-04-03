use super::register::Register;

/// x86 opcodes (16/32/64-bit modes including SSE/AVX)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opcode {
    // === Movement ===
    Mov, Movzx, Movsx, Lea, Xchg, Push, Pop,

    // === Arithmetic ===
    Add, Sub, Mul, Imul, Div, Idiv, Inc, Dec, Neg,

    // === Logic ===
    And, Or, Xor, Not, Shl, Shr, Sar, Rol, Ror,

    // === Comparison ===
    Cmp, Test,

    // === Jumps ===
    Jmp, Je, Jne, Jl, Jle, Jg, Jge, Jb, Jbe, Ja, Jae,

    // === Call/Return ===
    Call, Ret, Leave,

    // === String ops ===
    RepMovsb, RepStosb, Scasb,
    RepeCmpsb, RepneScasb, Movsb, Stosb, Cmpsb,

    // === System ===
    Syscall, Int, Hlt, Cli, Sti, Nop, Cpuid, Iretq,

    // === SSE ===
    Movaps, Movups, Addps, Mulps, Xorps,

    // === AVX ===
    Vmovaps, Vaddps, Vmulps,

    // === AVX extended ===
    Vsubps, Vdivps, Vxorps,

    // === SSE scalar ===
    Movss, Addss, Subss, Mulss, Divss, Sqrtss,
    Movsd, Addsd, Subsd, Mulsd, Divsd, Sqrtsd,
    Comiss, Comisd,

    // === SSE packed extra ===
    Subps, Divps, Minps, Maxps,

    // === Conditional moves ===
    Cmove, Cmovne, Cmovl, Cmovle, Cmovg, Cmovge, Cmovb, Cmova,

    // === Bit scan ===
    Bsf, Bsr,

    // === Misc ===
    Cqo, Cdq, Cbw,
}

impl Opcode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            // Movement
            "mov" => Some(Opcode::Mov), "movzx" => Some(Opcode::Movzx),
            "movsx" => Some(Opcode::Movsx), "lea" => Some(Opcode::Lea),
            "xchg" => Some(Opcode::Xchg), "push" => Some(Opcode::Push),
            "pop" => Some(Opcode::Pop),
            // Arithmetic
            "add" => Some(Opcode::Add), "sub" => Some(Opcode::Sub),
            "mul" => Some(Opcode::Mul), "imul" => Some(Opcode::Imul),
            "div" => Some(Opcode::Div), "idiv" => Some(Opcode::Idiv),
            "inc" => Some(Opcode::Inc), "dec" => Some(Opcode::Dec),
            "neg" => Some(Opcode::Neg),
            // Logic
            "and" => Some(Opcode::And), "or" => Some(Opcode::Or),
            "xor" => Some(Opcode::Xor), "not" => Some(Opcode::Not),
            "shl" => Some(Opcode::Shl), "shr" => Some(Opcode::Shr),
            "sar" => Some(Opcode::Sar), "rol" => Some(Opcode::Rol),
            "ror" => Some(Opcode::Ror),
            // Comparison
            "cmp" => Some(Opcode::Cmp), "test" => Some(Opcode::Test),
            // Jumps
            "jmp" => Some(Opcode::Jmp),
            "je" => Some(Opcode::Je), "jne" => Some(Opcode::Jne),
            "jl" => Some(Opcode::Jl), "jle" => Some(Opcode::Jle),
            "jg" => Some(Opcode::Jg), "jge" => Some(Opcode::Jge),
            "jb" => Some(Opcode::Jb), "jbe" => Some(Opcode::Jbe),
            "ja" => Some(Opcode::Ja), "jae" => Some(Opcode::Jae),
            // Call/Return
            "call" => Some(Opcode::Call), "ret" => Some(Opcode::Ret),
            "leave" => Some(Opcode::Leave),
            // String ops
            "rep movsb" => Some(Opcode::RepMovsb),
            "rep stosb" => Some(Opcode::RepStosb),
            "scasb" => Some(Opcode::Scasb),
            "repe cmpsb" => Some(Opcode::RepeCmpsb),
            "repne scasb" => Some(Opcode::RepneScasb),
            "movsb" => Some(Opcode::Movsb),
            "stosb" => Some(Opcode::Stosb),
            "cmpsb" => Some(Opcode::Cmpsb),
            // System
            "syscall" => Some(Opcode::Syscall), "int" => Some(Opcode::Int),
            "hlt" => Some(Opcode::Hlt), "cli" => Some(Opcode::Cli),
            "sti" => Some(Opcode::Sti), "nop" => Some(Opcode::Nop),
            "cpuid" => Some(Opcode::Cpuid), "iretq" => Some(Opcode::Iretq),
            // SSE
            "movaps" => Some(Opcode::Movaps), "movups" => Some(Opcode::Movups),
            "addps" => Some(Opcode::Addps), "mulps" => Some(Opcode::Mulps),
            "xorps" => Some(Opcode::Xorps),
            // AVX
            "vmovaps" => Some(Opcode::Vmovaps), "vaddps" => Some(Opcode::Vaddps),
            "vmulps" => Some(Opcode::Vmulps),
            "vsubps" => Some(Opcode::Vsubps),
            "vdivps" => Some(Opcode::Vdivps),
            "vxorps" => Some(Opcode::Vxorps),
            "movss" => Some(Opcode::Movss),
            "addss" => Some(Opcode::Addss),
            "subss" => Some(Opcode::Subss),
            "mulss" => Some(Opcode::Mulss),
            "divss" => Some(Opcode::Divss),
            "sqrtss" => Some(Opcode::Sqrtss),
            "movsd" => Some(Opcode::Movsd),
            "addsd" => Some(Opcode::Addsd),
            "subsd" => Some(Opcode::Subsd),
            "mulsd" => Some(Opcode::Mulsd),
            "divsd" => Some(Opcode::Divsd),
            "sqrtsd" => Some(Opcode::Sqrtsd),
            "comiss" => Some(Opcode::Comiss),
            "comisd" => Some(Opcode::Comisd),
            "subps" => Some(Opcode::Subps),
            "divps" => Some(Opcode::Divps),
            "minps" => Some(Opcode::Minps),
            "maxps" => Some(Opcode::Maxps),
            "cmove" | "cmovz" => Some(Opcode::Cmove),
            "cmovne" | "cmovnz" => Some(Opcode::Cmovne),
            "cmovl" => Some(Opcode::Cmovl),
            "cmovle" => Some(Opcode::Cmovle),
            "cmovg" => Some(Opcode::Cmovg),
            "cmovge" => Some(Opcode::Cmovge),
            "cmovb" => Some(Opcode::Cmovb),
            "cmova" => Some(Opcode::Cmova),
            "bsf" => Some(Opcode::Bsf),
            "bsr" => Some(Opcode::Bsr),
            "cqo" => Some(Opcode::Cqo),
            "cdq" => Some(Opcode::Cdq),
            "cbw" => Some(Opcode::Cbw),
            _ => None,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            // Movement
            Opcode::Mov => "mov", Opcode::Movzx => "movzx", Opcode::Movsx => "movsx",
            Opcode::Lea => "lea", Opcode::Xchg => "xchg",
            Opcode::Push => "push", Opcode::Pop => "pop",
            // Arithmetic
            Opcode::Add => "add", Opcode::Sub => "sub",
            Opcode::Mul => "mul", Opcode::Imul => "imul",
            Opcode::Div => "div", Opcode::Idiv => "idiv",
            Opcode::Inc => "inc", Opcode::Dec => "dec", Opcode::Neg => "neg",
            // Logic
            Opcode::And => "and", Opcode::Or => "or",
            Opcode::Xor => "xor", Opcode::Not => "not",
            Opcode::Shl => "shl", Opcode::Shr => "shr",
            Opcode::Sar => "sar", Opcode::Rol => "rol", Opcode::Ror => "ror",
            // Comparison
            Opcode::Cmp => "cmp", Opcode::Test => "test",
            // Jumps
            Opcode::Jmp => "jmp", Opcode::Je => "je", Opcode::Jne => "jne",
            Opcode::Jl => "jl", Opcode::Jle => "jle",
            Opcode::Jg => "jg", Opcode::Jge => "jge",
            Opcode::Jb => "jb", Opcode::Jbe => "jbe",
            Opcode::Ja => "ja", Opcode::Jae => "jae",
            // Call/Return
            Opcode::Call => "call", Opcode::Ret => "ret", Opcode::Leave => "leave",
            // String ops
            Opcode::RepMovsb => "rep movsb", Opcode::RepStosb => "rep stosb",
            Opcode::Scasb => "scasb",
            // System
            Opcode::Syscall => "syscall", Opcode::Int => "int",
            Opcode::Hlt => "hlt", Opcode::Cli => "cli",
            Opcode::Sti => "sti", Opcode::Nop => "nop",
            Opcode::Cpuid => "cpuid", Opcode::Iretq => "iretq",
            // SSE
            Opcode::Movaps => "movaps", Opcode::Movups => "movups",
            Opcode::Addps => "addps", Opcode::Mulps => "mulps", Opcode::Xorps => "xorps",
            // AVX
            Opcode::Vmovaps => "vmovaps", Opcode::Vaddps => "vaddps", Opcode::Vmulps => "vmulps",
            Opcode::RepeCmpsb => "repe cmpsb",
            Opcode::RepneScasb => "repne scasb",
            Opcode::Movsb => "movsb", Opcode::Stosb => "stosb", Opcode::Cmpsb => "cmpsb",
            Opcode::Vsubps => "vsubps", Opcode::Vdivps => "vdivps", Opcode::Vxorps => "vxorps",
            Opcode::Movss => "movss", Opcode::Addss => "addss", Opcode::Subss => "subss",
            Opcode::Mulss => "mulss", Opcode::Divss => "divss", Opcode::Sqrtss => "sqrtss",
            Opcode::Movsd => "movsd", Opcode::Addsd => "addsd", Opcode::Subsd => "subsd",
            Opcode::Mulsd => "mulsd", Opcode::Divsd => "divsd", Opcode::Sqrtsd => "sqrtsd",
            Opcode::Comiss => "comiss", Opcode::Comisd => "comisd",
            Opcode::Subps => "subps", Opcode::Divps => "divps",
            Opcode::Minps => "minps", Opcode::Maxps => "maxps",
            Opcode::Cmove => "cmove", Opcode::Cmovne => "cmovne",
            Opcode::Cmovl => "cmovl", Opcode::Cmovle => "cmovle",
            Opcode::Cmovg => "cmovg", Opcode::Cmovge => "cmovge",
            Opcode::Cmovb => "cmovb", Opcode::Cmova => "cmova",
            Opcode::Bsf => "bsf", Opcode::Bsr => "bsr",
            Opcode::Cqo => "cqo", Opcode::Cdq => "cdq", Opcode::Cbw => "cbw",
        }
    }
}

/// An operand to an instruction
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Reg(Register),
    Imm(i64),
    Label(String),
    Memory {
        base: Option<Register>,
        index: Option<Register>,
        scale: u8,
        disp: i64,
    },
    StringLit(String),
}

/// A single normalized instruction in the IR
#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: Opcode,
    pub operands: Vec<Operand>,
}

impl Instruction {
    pub fn new(opcode: Opcode, operands: Vec<Operand>) -> Self {
        Self { opcode, operands }
    }

    pub fn zero(opcode: Opcode) -> Self {
        Self { opcode, operands: vec![] }
    }

    pub fn one(opcode: Opcode, op: Operand) -> Self {
        Self { opcode, operands: vec![op] }
    }

    pub fn two(opcode: Opcode, dst: Operand, src: Operand) -> Self {
        Self { opcode, operands: vec![dst, src] }
    }
}
