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

/// Convert Register to 3-bit ModR/M encoding value, and flags for extended / wide / 16-bit
fn encode_reg(reg: &Register) -> (u8, bool, bool, bool) {
    let mut is_wide = true;  // 64-bit default
    let mut is_32 = false;
    let mut is_ext = false; // R8-R15
    
    let val = match reg {
        // 64-bit Regs
        Register::Rax => 0, Register::Rcx => 1, Register::Rdx => 2, Register::Rbx => 3,
        Register::Rsp => 4, Register::Rbp => 5, Register::Rsi => 6, Register::Rdi => 7,
        Register::R8 => { is_ext=true; 0 }, Register::R9 => { is_ext=true; 1 },
        Register::R10 => { is_ext=true; 2 }, Register::R11 => { is_ext=true; 3 },
        Register::R12 => { is_ext=true; 4 }, Register::R13 => { is_ext=true; 5 },
        Register::R14 => { is_ext=true; 6 }, Register::R15 => { is_ext=true; 7 },
        
        // 32-bit Regs
        Register::Eax => { is_wide=false; is_32=true; 0 }, Register::Ecx => { is_wide=false; is_32=true; 1 },
        Register::Edx => { is_wide=false; is_32=true; 2 }, Register::Ebx => { is_wide=false; is_32=true; 3 },
        Register::Esp => { is_wide=false; is_32=true; 4 }, Register::Ebp => { is_wide=false; is_32=true; 5 },
        Register::Esi => { is_wide=false; is_32=true; 6 }, Register::Edi => { is_wide=false; is_32=true; 7 },
        Register::R8d => { is_wide=false; is_32=true; is_ext=true; 0 },
        Register::R9d => { is_wide=false; is_32=true; is_ext=true; 1 },
        Register::R10d => { is_wide=false; is_32=true; is_ext=true; 2 },
        Register::R11d => { is_wide=false; is_32=true; is_ext=true; 3 },
        Register::R12d => { is_wide=false; is_32=true; is_ext=true; 4 },
        Register::R13d => { is_wide=false; is_32=true; is_ext=true; 5 },
        Register::R14d => { is_wide=false; is_32=true; is_ext=true; 6 },
        Register::R15d => { is_wide=false; is_32=true; is_ext=true; 7 },
        
        // SSE XMM
        Register::Xmm(n) => { is_wide=false; is_ext = *n > 7; *n & 7 },
        
        _ => { is_wide = false; 0 }, // 16-bit / 8-bit fallback
    };
    
    (val, is_ext, is_wide, is_32)
}

/// Generate ModR/M byte
fn modrm(mod_val: u8, reg: u8, rm: u8) -> u8 {
    ((mod_val & 3) << 6) | ((reg & 7) << 3) | (rm & 7)
}

/// Determine REX prefix requirement
fn build_rex(w: bool, r: bool, x: bool, b: bool) -> Option<u8> {
    if !w && !r && !x && !b {
        None
    } else {
        let mut prefix = 0x40;
        if w { prefix |= 0x08; }
        if r { prefix |= 0x04; }
        if x { prefix |= 0x02; }
        if b { prefix |= 0x01; }
        Some(prefix)
    }
}

pub fn encode_instruction(inst: &Instruction) -> Result<EncodedInstruction, String> {
    let mut bytes = Vec::new();
    let mut relocations = Vec::new();

    match inst.opcode {
        Opcode::Push => {
            if let Some(Operand::Reg(r)) = inst.operands.get(0) {
                let (reg_val, is_ext, _, _) = encode_reg(r);
                if let Some(rex) = build_rex(false, false, false, is_ext) { bytes.push(rex); }
                bytes.push(0x50 + reg_val);
            }
        }
        Opcode::Pop => {
            if let Some(Operand::Reg(r)) = inst.operands.get(0) {
                let (reg_val, is_ext, _, _) = encode_reg(r);
                if let Some(rex) = build_rex(false, false, false, is_ext) { bytes.push(rex); }
                bytes.push(0x58 + reg_val);
            }
        }
        Opcode::Sub | Opcode::Add | Opcode::Cmp => {
            if inst.operands.len() == 2 {
                let opc_base = match inst.opcode {
                    Opcode::Add => 0x00,
                    Opcode::Sub => 0x28,
                    Opcode::Cmp => 0x38,
                    _ => unreachable!(),
                };
                
                match (&inst.operands[0], &inst.operands[1]) {
                    (Operand::Reg(dst), Operand::Imm(imm)) => {
                        let (dst_v, dst_e, w, dst_32) = encode_reg(dst);
                        let sub_op_ext = match inst.opcode {
                            Opcode::Add => 0,
                            Opcode::Sub => 5,
                            Opcode::Cmp => 7,
                            _ => 0,
                        };
                        
                        if let Some(rex) = build_rex(w, false, false, dst_e) { bytes.push(rex); }
                        
                        let imm_val = *imm;
                        if -128 <= imm_val && imm_val <= 127 { // 8-bit imm
                            bytes.push(0x83);
                            bytes.push(modrm(3, sub_op_ext, dst_v));
                            bytes.push((imm_val as i8) as u8);
                        } else { // 32-bit imm
                            bytes.push(0x81);
                            bytes.push(modrm(3, sub_op_ext, dst_v));
                            bytes.extend_from_slice(&(imm_val as i32).to_le_bytes());
                        }
                    }
                    (Operand::Reg(dst), Operand::Reg(src)) => {
                        let (dst_v, dst_e, w, dst_32) = encode_reg(dst);
                        let (src_v, src_e, _, _) = encode_reg(src);
                        if let Some(rex) = build_rex(w, src_e, false, dst_e) { bytes.push(rex); }
                        bytes.push(opc_base + 1); // e.g. 0x29 for Sub
                        bytes.push(modrm(3, src_v, dst_v));
                    }
                    _ => {}
                }
            }
        }
        Opcode::Dec => {
            if let Some(Operand::Reg(r)) = inst.operands.get(0) {
                let (reg_val, is_ext, w, _) = encode_reg(r);
                if let Some(rex) = build_rex(w, false, false, is_ext) { bytes.push(rex); }
                bytes.push(0xFF);
                bytes.push(modrm(3, 1, reg_val));
            }
        }
        Opcode::Test => {
            if let (Some(Operand::Reg(dst)), Some(Operand::Reg(src))) = (inst.operands.get(0), inst.operands.get(1)) {
                let (dst_v, dst_e, w, _) = encode_reg(dst);
                let (src_v, src_e, _, _) = encode_reg(src);
                if let Some(rex) = build_rex(w, src_e, false, dst_e) { bytes.push(rex); }
                bytes.push(0x85);
                bytes.push(modrm(3, src_v, dst_v));
            }
        }
        Opcode::Imul => {
            if inst.operands.len() == 2 {
                if let (Operand::Reg(dst), Operand::Reg(src)) = (&inst.operands[0], &inst.operands[1]) {
                    let (dst_v, dst_e, w, _) = encode_reg(dst);
                    let (src_v, src_e, _, _) = encode_reg(src);
                    if let Some(rex) = build_rex(w, dst_e, false, src_e) { bytes.push(rex); }
                    bytes.push(0x0F);
                    bytes.push(0xAF);
                    bytes.push(modrm(3, dst_v, src_v));
                }
            }
        }
        Opcode::Xor => {
            if let (Some(Operand::Reg(dst)), Some(Operand::Reg(src))) = (inst.operands.get(0), inst.operands.get(1)) {
                let (dst_v, dst_e, w, d32) = encode_reg(dst);
                let (src_v, src_e, _, _) = encode_reg(src);
                // For 'xor ecx, ecx', dropping REX.W is ideal even in 64-bit to clear full RCX
                if let Some(rex) = build_rex(w, src_e, false, dst_e) { bytes.push(rex); }
                bytes.push(0x31);
                bytes.push(modrm(3, src_v, dst_v));
            }
        }
        Opcode::Mov => {
            if let (Some(Operand::Reg(dst)), Some(Operand::Reg(src))) = (inst.operands.get(0), inst.operands.get(1)) {
                let (dst_v, dst_e, w, _) = encode_reg(dst);
                let (src_v, src_e, _, _) = encode_reg(src);
                if let Some(rex) = build_rex(w, src_e, false, dst_e) { bytes.push(rex); }
                bytes.push(0x89);
                bytes.push(modrm(3, src_v, dst_v));
            }
            if let (Some(Operand::Reg(dst)), Some(Operand::Imm(imm))) = (inst.operands.get(0), inst.operands.get(1)) {
                let (dst_v, dst_e, w, is32) = encode_reg(dst);
                let imm_val = *imm;
                
                if w && (imm_val > i32::MAX as i64 || imm_val < i32::MIN as i64) {
                    // Full 64-bit mov: REX.W B8+rg imm64
                    if let Some(rex) = build_rex(true, false, false, dst_e) { bytes.push(rex); }
                    bytes.push(0xB8 + dst_v);
                    bytes.extend_from_slice(&imm_val.to_le_bytes());
                } else if is32 {
                    // 32-bit imm to 32 bit register B8+rg imm32
                    if let Some(rex) = build_rex(false, false, false, dst_e) { bytes.push(rex); }
                    bytes.push(0xB8 + dst_v);
                    bytes.extend_from_slice(&(imm_val as i32).to_le_bytes());
                } else {
                    // C7 mov r/m, imm32 with REX.W
                    if let Some(rex) = build_rex(w, false, false, dst_e) { bytes.push(rex); }
                    bytes.push(0xC7);
                    bytes.push(modrm(3, 0, dst_v));
                    bytes.extend_from_slice(&(imm_val as i32).to_le_bytes());
                }
            }
            if let (Some(Operand::Reg(dst)), Some(Operand::Memory { base: Some(b), index: None, disp, .. })) = (inst.operands.get(0), inst.operands.get(1)) {
                let (dst_v, dst_e, w, _) = encode_reg(dst);
                let (base_v, base_e, _, _) = encode_reg(b);
                if let Some(rex) = build_rex(w, dst_e, false, base_e) { bytes.push(rex); }
                bytes.push(0x8B); // MOV r, r/m
                if *disp == 0 && base_v != 5 {
                    bytes.push(modrm(0, dst_v, base_v));
                } else if *disp >= -128 && *disp <= 127 {
                    bytes.push(modrm(1, dst_v, base_v));
                    bytes.push(*disp as i8 as u8);
                } else {
                    bytes.push(modrm(2, dst_v, base_v));
                    bytes.extend_from_slice(&(*disp as i32).to_le_bytes());
                }
            }
        }
        Opcode::Lea => {
            if let (Some(Operand::Reg(dst)), Some(Operand::Label(lbl))) = (inst.operands.get(0), inst.operands.get(1)) {
                let (dst_v, dst_e, w, _) = encode_reg(dst);
                if let Some(rex) = build_rex(w, dst_e, false, false) { bytes.push(rex); }
                bytes.push(0x8D);
                // RIP-relative: mod=0, rm=5 -> disp32
                bytes.push(modrm(0, dst_v, 5));
                bytes.extend_from_slice(&[0, 0, 0, 0]); // Blank offset for linker
                relocations.push(RelocationReq {
                    offset: bytes.len() as u32 - 4,
                    symbol: lbl.clone(),
                    rel_type: 0x0004, // IMAGE_REL_AMD64_REL32
                });
            }
        }
        Opcode::Jmp => {
            if let Some(Operand::Label(lbl)) = inst.operands.get(0) {
                bytes.push(0xE9);
                bytes.extend_from_slice(&[0, 0, 0, 0]);
                relocations.push(RelocationReq {
                    offset: 1,
                    symbol: lbl.clone(),
                    rel_type: 0x0004,
                });
            }
        }
        Opcode::Je | Opcode::Jne => {
            if let Some(Operand::Label(lbl)) = inst.operands.get(0) {
                bytes.push(0x0F);
                bytes.push(if inst.opcode == Opcode::Je { 0x84 } else { 0x85 });
                bytes.extend_from_slice(&[0, 0, 0, 0]);
                relocations.push(RelocationReq {
                    offset: 2,
                    symbol: lbl.clone(),
                    rel_type: 0x0004,
                });
            }
        }
        Opcode::Call => {
            if let Some(Operand::Label(sym)) = inst.operands.get(0) {
                bytes.push(0xE8);
                bytes.extend_from_slice(&[0, 0, 0, 0]);
                relocations.push(RelocationReq {
                    offset: 1,
                    symbol: sym.clone(),
                    rel_type: 0x0004,
                });
            }
        }
        
        // --- SSE Instructions ---
        Opcode::Cvtsi2ss => { // F3 0F 2A /r
            if let (Some(Operand::Reg(dst)), Some(Operand::Reg(src))) = (inst.operands.get(0), inst.operands.get(1)) {
                let (dst_v, dst_e, _, _) = encode_reg(dst);
                let (src_v, src_e, src_w, _) = encode_reg(src);
                bytes.push(0xF3);
                if let Some(rex) = build_rex(src_w, dst_e, false, src_e) { bytes.push(rex); }
                bytes.push(0x0F); bytes.push(0x2A);
                bytes.push(modrm(3, dst_v, src_v));
            }
        }
        Opcode::Cvtss2si => { // F3 0F 2D /r
            if let (Some(Operand::Reg(dst)), Some(Operand::Reg(src))) = (inst.operands.get(0), inst.operands.get(1)) {
                let (dst_v, dst_e, dst_w, _) = encode_reg(dst);
                let (src_v, src_e, _, _) = encode_reg(src);
                bytes.push(0xF3);
                if let Some(rex) = build_rex(dst_w, dst_e, false, src_e) { bytes.push(rex); }
                bytes.push(0x0F); bytes.push(0x2D);
                bytes.push(modrm(3, dst_v, src_v));
            }
        }
        Opcode::Sqrtss => { // F3 0F 51 /r
            if let (Some(Operand::Reg(dst)), Some(Operand::Reg(src))) = (inst.operands.get(0), inst.operands.get(1)) {
                let (dst_v, dst_e, _, _) = encode_reg(dst);
                let (src_v, src_e, _, _) = encode_reg(src);
                bytes.push(0xF3);
                if let Some(rex) = build_rex(false, dst_e, false, src_e) { bytes.push(rex); }
                bytes.push(0x0F); bytes.push(0x51);
                bytes.push(modrm(3, dst_v, src_v));
            }
        }
        Opcode::Movss => {
            bytes.push(0xF3);
            if let (Some(Operand::Reg(dst)), Some(Operand::Memory { base: Some(b), disp, .. })) = (inst.operands.get(0), inst.operands.get(1)) {
                let (dst_v, dst_e, _, _) = encode_reg(dst);
                let (base_v, base_e, _, _) = encode_reg(b);
                if let Some(rex) = build_rex(false, dst_e, false, base_e) { bytes.push(rex); }
                bytes.push(0x0F); bytes.push(0x10); // MOVSS xmm, m32
                if *disp == 0 && base_v != 5 {
                    bytes.push(modrm(0, dst_v, base_v));
                } else if *disp >= -128 && *disp <= 127 {
                    bytes.push(modrm(1, dst_v, base_v));
                    bytes.push(*disp as i8 as u8);
                } else {
                    bytes.push(modrm(2, dst_v, base_v));
                    bytes.extend_from_slice(&(*disp as i32).to_le_bytes());
                }
            }
        }
        Opcode::Addss => {
            bytes.push(0xF3);
            if let (Some(Operand::Reg(dst)), Some(Operand::Memory { base: Some(b), disp, .. })) = (inst.operands.get(0), inst.operands.get(1)) {
                let (dst_v, dst_e, _, _) = encode_reg(dst);
                let (base_v, base_e, _, _) = encode_reg(b);
                if let Some(rex) = build_rex(false, dst_e, false, base_e) { bytes.push(rex); }
                bytes.push(0x0F); bytes.push(0x58); // ADDSS xmm, m32
                if *disp == 0 && base_v != 5 {
                    bytes.push(modrm(0, dst_v, base_v));
                } else if *disp >= -128 && *disp <= 127 {
                    bytes.push(modrm(1, dst_v, base_v));
                    bytes.push(*disp as i8 as u8);
                } else {
                    bytes.push(modrm(2, dst_v, base_v));
                    bytes.extend_from_slice(&(*disp as i32).to_le_bytes());
                }
            }
        }

        Opcode::Ret => { bytes.push(0xC3); }
        Opcode::Leave => { bytes.push(0xC9); }
        
        _ => return Err(format!("Unimplemented binary encoding for {:?}", inst.opcode)),
    }
    
    Ok(EncodedInstruction { bytes, relocations })
}
