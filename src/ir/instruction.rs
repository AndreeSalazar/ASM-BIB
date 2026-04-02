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

    // === System ===
    Syscall, Int, Hlt, Cli, Sti, Nop, Cpuid, Iretq,

    // === SSE ===
    Movaps, Movups, Addps, Mulps, Xorps,

    // === AVX ===
    Vmovaps, Vaddps, Vmulps,
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
