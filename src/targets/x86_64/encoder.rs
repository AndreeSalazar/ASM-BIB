//! x86-64 Native Machine Code Encoder
//! Transforms ASM-BIB IR Instructions into raw byte sequences for COFF sections.

use crate::ir::{Instruction, Opcode, Operand, Register};
use std::collections::HashMap;

/// Result of an encoding operation containing machine code and necessary relocations (e.g., symbol jumps)
pub struct EncodedInstruction {
    pub bytes: Vec<u8>,
    pub relocations: Vec<RelocationReq>,
}

pub struct RelocationReq {
    pub offset: u32,       // offset within the instruction bytes
    pub symbol: String,    // target symbol name
    pub rel_type: u16,     // COFF relocation type (e.g., IMAGE_REL_AMD64_REL32)
}

/// Convert Register to 3-bit ModR/M encoding value and determine if REX.B/X/R is needed
fn encode_reg(reg: &Register) -> (u8, bool) {
    match reg {
        // Core 64-bit
        Register::Rax | Register::Eax | Register::Ax | Register::Al => (0, false),
        Register::Rcx | Register::Ecx | Register::Cx | Register::Cl => (1, false),
        Register::Rdx | Register::Edx | Register::Dx | Register::Dl => (2, false),
        Register::Rbx | Register::Ebx | Register::Bx | Register::Bl => (3, false),
        Register::Rsp | Register::Esp | Register::Sp | Register::Ah => (4, false),
        Register::Rbp | Register::Ebp | Register::Bp | Register::Ch => (5, false),
        Register::Rsi | Register::Esi | Register::Si | Register::Dh => (6, false),
        Register::Rdi | Register::Edi | Register::Di | Register::Bh => (7, false),
        // Extended 64-bit
        Register::R8 | Register::R8d => (0, true),
        Register::R9 | Register::R9d => (1, true),
        Register::R10 | Register::R10d => (2, true),
        Register::R11 | Register::R11d => (3, true),
        Register::R12 | Register::R12d => (4, true),
        Register::R13 | Register::R13d => (5, true),
        Register::R14 | Register::R14d => (6, true),
        Register::R15 | Register::R15d => (7, true),
        
        // SSE
        Register::Xmm(n) => (*n & 7, *n > 7),
        
        _ => (0, false), // Default fallback
    }
}

/// Helper to generate ModR/M byte
fn modrm(mod_val: u8, reg: u8, rm: u8) -> u8 {
    ((mod_val & 3) << 6) | ((reg & 7) << 3) | (rm & 7)
}

/// Build REX prefix byte: 0100WRXB
fn rex(w: bool, r: bool, x: bool, b: bool) -> u8 {
    let mut prefix = 0x40;
    if w { prefix |= 0x08; }
    if r { prefix |= 0x04; }
    if x { prefix |= 0x02; }
    if b { prefix |= 0x01; }
    prefix
}

/// Translate IR to Machine Code
pub fn encode_instruction(inst: &Instruction) -> Result<EncodedInstruction, String> {
    let mut bytes = Vec::new();
    let mut relocations = Vec::new();
    
    // Simplistic structure logic for core instructions needed in `complete_demo.pasm`
    match inst.opcode {
        Opcode::Push => {
            if let Some(Operand::Reg(r)) = inst.operands.get(0) {
                let (reg_val, is_extended) = encode_reg(r);
                if is_extended {
                    bytes.push(rex(false, false, false, true));
                }
                bytes.push(0x50 + reg_val);
            }
        }
        Opcode::Pop => {
            if let Some(Operand::Reg(r)) = inst.operands.get(0) {
                let (reg_val, is_extended) = encode_reg(r);
                if is_extended {
                    bytes.push(rex(false, false, false, true));
                }
                bytes.push(0x58 + reg_val);
            }
        }
        Opcode::Ret => {
            bytes.push(0xC3);
        }
        Opcode::Leave => {
            bytes.push(0xC9);
        }
        Opcode::Xor => {
            if inst.operands.len() == 2 {
                if let (Operand::Reg(dst), Operand::Reg(src)) = (&inst.operands[0], &inst.operands[1]) {
                    // xor reg, reg (32-bit usually 31 r_rm, 64-bit needs REX.W)
                    // Simplified: Assuming 32-bit `xor ecx, ecx` which is 31 C9
                    let (dst_v, dst_ext) = encode_reg(dst);
                    let (src_v, src_ext) = encode_reg(src);
                    if dst_ext || src_ext {
                        bytes.push(rex(false, src_ext, false, dst_ext));
                    }
                    bytes.push(0x31);
                    bytes.push(modrm(3, src_v, dst_v)); 
                }
            }
        }
        Opcode::Mov => {
            if inst.operands.len() == 2 {
                match (&inst.operands[0], &inst.operands[1]) {
                    (Operand::Reg(dst), Operand::Reg(src)) => {
                        let (dst_v, dst_ext) = encode_reg(dst);
                        let (src_v, src_ext) = encode_reg(src);
                        // REX.W = 1 for 64-bit mov => 48 89 C3 (mov rbx, rax)
                        bytes.push(rex(true, src_ext, false, dst_ext));
                        bytes.push(0x89);
                        bytes.push(modrm(3, src_v, dst_v));
                    }
                    (Operand::Reg(dst), Operand::Imm(imm)) => {
                        let (dst_v, dst_ext) = encode_reg(dst);
                        // mov reg, imm => B8+reg (32-bit) OR REX.W B8+reg (64-bit)
                        bytes.push(rex(true, false, false, dst_ext));
                        bytes.push(0xB8 + dst_v);
                        bytes.extend_from_slice(&(*imm).to_le_bytes()); // Send 8 bytes for 64-bit
                    }
                    _ => {} // Other variants to be completed
                }
            }
        }
        Opcode::Call => {
            if let Some(Operand::Label(sym)) = inst.operands.get(0) {
                // E8 call rel32
                bytes.push(0xE8);
                bytes.extend_from_slice(&[0, 0, 0, 0]); // Blank offset for linker
                relocations.push(RelocationReq {
                    offset: 1, // After E8 byte
                    symbol: sym.clone(),
                    rel_type: 0x0004, // IMAGE_REL_AMD64_REL32
                });
            } else {
                return Err("Unsupported call operand".into());
            }
        }
        // Basic integration hook, the rest of the opcodes...
        _ => return Err(format!("Unimplemented binary encoding for {:?}", inst.opcode)),
    }
    
    Ok(EncodedInstruction { bytes, relocations })
}
