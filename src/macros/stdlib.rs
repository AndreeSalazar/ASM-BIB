use crate::ir::*;

/// Generate prologue instructions for a function
pub fn prologue(stack_size: i64, arch: Arch) -> Vec<Instruction> {
    match arch {
        Arch::X86_64 | Arch::X86_32 => vec![
            Instruction::one(Opcode::Push, Operand::Reg(Register::Rbp)),
            Instruction::two(Opcode::Mov, Operand::Reg(Register::Rbp), Operand::Reg(Register::Rsp)),
            Instruction::two(Opcode::Sub, Operand::Reg(Register::Rsp), Operand::Imm(stack_size)),
        ],
        Arch::X86_16 => vec![
            Instruction::one(Opcode::Push, Operand::Reg(Register::Bp)),
            Instruction::two(Opcode::Mov, Operand::Reg(Register::Bp), Operand::Reg(Register::Sp)),
            Instruction::two(Opcode::Sub, Operand::Reg(Register::Sp), Operand::Imm(stack_size)),
        ],
        Arch::Arm64 => vec![
            Instruction::new(Opcode::Stp, vec![
                Operand::Reg(Register::X(29)),
                Operand::Reg(Register::X(30)),
            ]),
            Instruction::two(Opcode::Mov, Operand::Reg(Register::X(29)), Operand::Reg(Register::ArmSp)),
            Instruction::two(Opcode::Sub, Operand::Reg(Register::ArmSp), Operand::Imm(stack_size)),
        ],
        _ => vec![],
    }
}

/// Generate epilogue instructions for a function
pub fn epilogue(arch: Arch) -> Vec<Instruction> {
    match arch {
        Arch::X86_64 | Arch::X86_32 => vec![
            Instruction::zero(Opcode::Leave),
            Instruction::zero(Opcode::Ret),
        ],
        Arch::X86_16 => vec![
            Instruction::two(Opcode::Mov, Operand::Reg(Register::Sp), Operand::Reg(Register::Bp)),
            Instruction::one(Opcode::Pop, Operand::Reg(Register::Bp)),
            Instruction::zero(Opcode::Ret),
        ],
        Arch::Arm64 => vec![
            Instruction::new(Opcode::Ldp, vec![
                Operand::Reg(Register::X(29)),
                Operand::Reg(Register::X(30)),
            ]),
            Instruction::zero(Opcode::ArmRet),
        ],
        _ => vec![],
    }
}

/// Generate a syscall sequence (Linux x86_64)
pub fn linux_syscall(num: i64, arg1: Option<Operand>) -> Vec<Instruction> {
    let mut insts = vec![
        Instruction::two(Opcode::Mov, Operand::Reg(Register::Rax), Operand::Imm(num)),
    ];
    if let Some(a1) = arg1 {
        insts.push(Instruction::two(Opcode::Mov, Operand::Reg(Register::Rdi), a1));
    }
    insts.push(Instruction::zero(Opcode::Syscall));
    insts
}
