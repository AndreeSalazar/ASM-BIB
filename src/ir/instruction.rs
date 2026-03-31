use super::register::Register;

/// All supported opcodes across ALL architectures (unified)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opcode {
    // === Movement ===
    Mov, Movzx, Movsx, Lea, Xchg, Push, Pop,
    // ARM-specific moves
    Ldr, Str, Ldp, Stp, Adr, Adrp,
    // RISC-V loads/stores
    Lb, Lh, Lw, Ld, Sb, Sh, Sw, Sd, Lui, Auipc,

    // === Arithmetic ===
    Add, Sub, Mul, Imul, Div, Idiv, Inc, Dec, Neg,
    // ARM arithmetic
    Madd, Msub, Sdiv, Udiv,
    // RISC-V immediate
    Addi, Xori, Ori, Andi,

    // === Logic ===
    And, Or, Xor, Not, Shl, Shr, Sar, Rol, Ror,
    // ARM logic
    Orr, Eor, Mvn, Lsl, Lsr, Asr,
    // RISC-V shifts
    Sll, Srl, Sra, Slli, Srli, Srai,

    // === Comparison ===
    Cmp, Test,
    // ARM compare
    Cmn, Tst,

    // === Jumps/Branches ===
    Jmp, Je, Jne, Jl, Jle, Jg, Jge, Jb, Jbe, Ja, Jae,
    // ARM branches
    B, Bl, Br, Blr,
    Beq, Bne, Blt, Bgt, Ble, Bge,
    Cbz, Cbnz, Tbz, Tbnz,
    // RISC-V branches
    Jal, Jalr, Bge_rv, Bltu, Bgeu,

    // === Call/Return ===
    Call, Ret, Leave,
    // ARM return
    ArmRet,

    // === String ops ===
    RepMovsb, RepStosb, Scasb,

    // === System ===
    Syscall, Int, Hlt, Cli, Sti, Nop, Cpuid,
    Iretq,
    // ARM system
    Svc, Mrs, Msr,
    // RISC-V system
    Ecall, Ebreak, Fence,

    // === SSE ===
    Movaps, Movups, Addps, Mulps, Xorps,
    // === AVX ===
    Vmovaps, Vaddps, Vmulps,
    // ARM SIMD
    Fmov, Fadd, Fmul, Fdiv, Fcmp,
}

impl Opcode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            // Movement
            "mov" => Some(Opcode::Mov), "movzx" => Some(Opcode::Movzx),
            "movsx" => Some(Opcode::Movsx), "lea" => Some(Opcode::Lea),
            "xchg" => Some(Opcode::Xchg), "push" => Some(Opcode::Push),
            "pop" => Some(Opcode::Pop),
            "ldr" => Some(Opcode::Ldr), "str" => Some(Opcode::Str),
            "ldp" => Some(Opcode::Ldp), "stp" => Some(Opcode::Stp),
            "adr" => Some(Opcode::Adr), "adrp" => Some(Opcode::Adrp),
            "lb" => Some(Opcode::Lb), "lh" => Some(Opcode::Lh),
            "lw" => Some(Opcode::Lw), "ld" => Some(Opcode::Ld),
            "sb" => Some(Opcode::Sb), "sh" => Some(Opcode::Sh),
            "sw" => Some(Opcode::Sw), "sd" => Some(Opcode::Sd),
            "lui" => Some(Opcode::Lui), "auipc" => Some(Opcode::Auipc),
            // Arithmetic
            "add" => Some(Opcode::Add), "sub" => Some(Opcode::Sub),
            "mul" => Some(Opcode::Mul), "imul" => Some(Opcode::Imul),
            "div" => Some(Opcode::Div), "idiv" => Some(Opcode::Idiv),
            "inc" => Some(Opcode::Inc), "dec" => Some(Opcode::Dec),
            "neg" => Some(Opcode::Neg),
            "madd" => Some(Opcode::Madd), "msub" => Some(Opcode::Msub),
            "sdiv" => Some(Opcode::Sdiv), "udiv" => Some(Opcode::Udiv),
            "addi" => Some(Opcode::Addi), "xori" => Some(Opcode::Xori),
            "ori" => Some(Opcode::Ori), "andi" => Some(Opcode::Andi),
            // Logic
            "and" => Some(Opcode::And), "or" => Some(Opcode::Or),
            "xor" => Some(Opcode::Xor), "not" => Some(Opcode::Not),
            "shl" => Some(Opcode::Shl), "shr" => Some(Opcode::Shr),
            "sar" => Some(Opcode::Sar), "rol" => Some(Opcode::Rol),
            "ror" => Some(Opcode::Ror),
            "orr" => Some(Opcode::Orr), "eor" => Some(Opcode::Eor),
            "mvn" => Some(Opcode::Mvn), "lsl" => Some(Opcode::Lsl),
            "lsr" => Some(Opcode::Lsr), "asr" => Some(Opcode::Asr),
            "sll" => Some(Opcode::Sll), "srl" => Some(Opcode::Srl),
            "sra" => Some(Opcode::Sra),
            "slli" => Some(Opcode::Slli), "srli" => Some(Opcode::Srli),
            "srai" => Some(Opcode::Srai),
            // Comparison
            "cmp" => Some(Opcode::Cmp), "test" => Some(Opcode::Test),
            "cmn" => Some(Opcode::Cmn), "tst" => Some(Opcode::Tst),
            // Jumps
            "jmp" => Some(Opcode::Jmp),
            "je" => Some(Opcode::Je), "jne" => Some(Opcode::Jne),
            "jl" => Some(Opcode::Jl), "jle" => Some(Opcode::Jle),
            "jg" => Some(Opcode::Jg), "jge" => Some(Opcode::Jge),
            "jb" => Some(Opcode::Jb), "jbe" => Some(Opcode::Jbe),
            "ja" => Some(Opcode::Ja), "jae" => Some(Opcode::Jae),
            "b" => Some(Opcode::B), "bl" => Some(Opcode::Bl),
            "br" => Some(Opcode::Br), "blr" => Some(Opcode::Blr),
            "beq" | "b.eq" => Some(Opcode::Beq), "bne" | "b.ne" => Some(Opcode::Bne),
            "blt" | "b.lt" => Some(Opcode::Blt), "bgt" | "b.gt" => Some(Opcode::Bgt),
            "ble" | "b.le" => Some(Opcode::Ble), "bge" | "b.ge" => Some(Opcode::Bge),
            "cbz" => Some(Opcode::Cbz), "cbnz" => Some(Opcode::Cbnz),
            "tbz" => Some(Opcode::Tbz), "tbnz" => Some(Opcode::Tbnz),
            "jal" => Some(Opcode::Jal), "jalr" => Some(Opcode::Jalr),
            "bltu" => Some(Opcode::Bltu), "bgeu" => Some(Opcode::Bgeu),
            // Call/Return
            "call" => Some(Opcode::Call), "ret" => Some(Opcode::Ret),
            "leave" => Some(Opcode::Leave),
            // String
            "rep movsb" => Some(Opcode::RepMovsb),
            "rep stosb" => Some(Opcode::RepStosb),
            "scasb" => Some(Opcode::Scasb),
            // System
            "syscall" => Some(Opcode::Syscall), "int" => Some(Opcode::Int),
            "hlt" => Some(Opcode::Hlt), "cli" => Some(Opcode::Cli),
            "sti" => Some(Opcode::Sti), "nop" => Some(Opcode::Nop),
            "cpuid" => Some(Opcode::Cpuid), "iretq" => Some(Opcode::Iretq),
            "svc" => Some(Opcode::Svc), "mrs" => Some(Opcode::Mrs),
            "msr" => Some(Opcode::Msr),
            "ecall" => Some(Opcode::Ecall), "ebreak" => Some(Opcode::Ebreak),
            "fence" => Some(Opcode::Fence),
            // SSE/AVX
            "movaps" => Some(Opcode::Movaps), "movups" => Some(Opcode::Movups),
            "addps" => Some(Opcode::Addps), "mulps" => Some(Opcode::Mulps),
            "xorps" => Some(Opcode::Xorps),
            "vmovaps" => Some(Opcode::Vmovaps), "vaddps" => Some(Opcode::Vaddps),
            "vmulps" => Some(Opcode::Vmulps),
            // ARM FP
            "fmov" => Some(Opcode::Fmov), "fadd" => Some(Opcode::Fadd),
            "fmul" => Some(Opcode::Fmul), "fdiv" => Some(Opcode::Fdiv),
            "fcmp" => Some(Opcode::Fcmp),
            _ => None,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Opcode::Mov => "mov", Opcode::Movzx => "movzx", Opcode::Movsx => "movsx",
            Opcode::Lea => "lea", Opcode::Xchg => "xchg",
            Opcode::Push => "push", Opcode::Pop => "pop",
            Opcode::Ldr => "ldr", Opcode::Str => "str",
            Opcode::Ldp => "ldp", Opcode::Stp => "stp",
            Opcode::Adr => "adr", Opcode::Adrp => "adrp",
            Opcode::Lb => "lb", Opcode::Lh => "lh",
            Opcode::Lw => "lw", Opcode::Ld => "ld",
            Opcode::Sb => "sb", Opcode::Sh => "sh",
            Opcode::Sw => "sw", Opcode::Sd => "sd",
            Opcode::Lui => "lui", Opcode::Auipc => "auipc",
            Opcode::Add => "add", Opcode::Sub => "sub",
            Opcode::Mul => "mul", Opcode::Imul => "imul",
            Opcode::Div => "div", Opcode::Idiv => "idiv",
            Opcode::Inc => "inc", Opcode::Dec => "dec", Opcode::Neg => "neg",
            Opcode::Madd => "madd", Opcode::Msub => "msub",
            Opcode::Sdiv => "sdiv", Opcode::Udiv => "udiv",
            Opcode::Addi => "addi", Opcode::Xori => "xori",
            Opcode::Ori => "ori", Opcode::Andi => "andi",
            Opcode::And => "and", Opcode::Or => "or",
            Opcode::Xor => "xor", Opcode::Not => "not",
            Opcode::Shl => "shl", Opcode::Shr => "shr",
            Opcode::Sar => "sar", Opcode::Rol => "rol", Opcode::Ror => "ror",
            Opcode::Orr => "orr", Opcode::Eor => "eor",
            Opcode::Mvn => "mvn", Opcode::Lsl => "lsl",
            Opcode::Lsr => "lsr", Opcode::Asr => "asr",
            Opcode::Sll => "sll", Opcode::Srl => "srl", Opcode::Sra => "sra",
            Opcode::Slli => "slli", Opcode::Srli => "srli", Opcode::Srai => "srai",
            Opcode::Cmp => "cmp", Opcode::Test => "test",
            Opcode::Cmn => "cmn", Opcode::Tst => "tst",
            Opcode::Jmp => "jmp", Opcode::Je => "je", Opcode::Jne => "jne",
            Opcode::Jl => "jl", Opcode::Jle => "jle",
            Opcode::Jg => "jg", Opcode::Jge => "jge",
            Opcode::Jb => "jb", Opcode::Jbe => "jbe",
            Opcode::Ja => "ja", Opcode::Jae => "jae",
            Opcode::B => "b", Opcode::Bl => "bl",
            Opcode::Br => "br", Opcode::Blr => "blr",
            Opcode::Beq => "b.eq", Opcode::Bne => "b.ne",
            Opcode::Blt => "b.lt", Opcode::Bgt => "b.gt",
            Opcode::Ble => "b.le", Opcode::Bge => "b.ge",
            Opcode::Cbz => "cbz", Opcode::Cbnz => "cbnz",
            Opcode::Tbz => "tbz", Opcode::Tbnz => "tbnz",
            Opcode::Jal => "jal", Opcode::Jalr => "jalr",
            Opcode::Bge_rv => "bge", Opcode::Bltu => "bltu", Opcode::Bgeu => "bgeu",
            Opcode::Call => "call", Opcode::Ret => "ret", Opcode::Leave => "leave",
            Opcode::ArmRet => "ret",
            Opcode::RepMovsb => "rep movsb", Opcode::RepStosb => "rep stosb",
            Opcode::Scasb => "scasb",
            Opcode::Syscall => "syscall", Opcode::Int => "int",
            Opcode::Hlt => "hlt", Opcode::Cli => "cli",
            Opcode::Sti => "sti", Opcode::Nop => "nop",
            Opcode::Cpuid => "cpuid", Opcode::Iretq => "iretq",
            Opcode::Svc => "svc", Opcode::Mrs => "mrs", Opcode::Msr => "msr",
            Opcode::Ecall => "ecall", Opcode::Ebreak => "ebreak", Opcode::Fence => "fence",
            Opcode::Movaps => "movaps", Opcode::Movups => "movups",
            Opcode::Addps => "addps", Opcode::Mulps => "mulps", Opcode::Xorps => "xorps",
            Opcode::Vmovaps => "vmovaps", Opcode::Vaddps => "vaddps", Opcode::Vmulps => "vmulps",
            Opcode::Fmov => "fmov", Opcode::Fadd => "fadd",
            Opcode::Fmul => "fmul", Opcode::Fdiv => "fdiv", Opcode::Fcmp => "fcmp",
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
