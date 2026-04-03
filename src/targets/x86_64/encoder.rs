use crate::ir::{Instruction, Opcode, Operand};

use super::sib;
use super::vex;

pub struct EncodedInstruction {
    pub bytes: Vec<u8>,
    pub relocations: Vec<RelocationReq>,
}

pub struct RelocationReq {
    pub offset: u32,
    pub symbol: String,
    pub rel_type: u16,
}

use std::collections::HashMap;

pub fn encode_instruction(inst: &Instruction, labels: Option<&HashMap<String, u32>>, current_offset: u32) -> Result<EncodedInstruction, String> {
    let mut bytes = Vec::new();
    let mut relocations = Vec::new();

    match inst.opcode {
        // --- 1. Basic Flow ---
        Opcode::Push => {
            if let Some(Operand::Reg(r)) = inst.operands.get(0) {
                let ri = sib::encode_reg(r);
                if let Some(rex) = sib::build_rex(false, false, false, ri.is_ext) { bytes.push(rex); }
                bytes.push(0x50 + ri.val);
            }
        }
        Opcode::Pop => {
            if let Some(Operand::Reg(r)) = inst.operands.get(0) {
                let ri = sib::encode_reg(r);
                if let Some(rex) = sib::build_rex(false, false, false, ri.is_ext) { bytes.push(rex); }
                bytes.push(0x58 + ri.val);
            }
        }
        Opcode::Ret => { bytes.push(0xC3); }
        Opcode::Leave => { bytes.push(0xC9); }

        // --- 2. Advanced Arithmetic & Logic (SUB/ADD/CMP) ---
        Opcode::Sub | Opcode::Add | Opcode::Cmp | Opcode::Xor => {
            if inst.operands.len() == 2 {
                let opc_base = match inst.opcode {
                    Opcode::Add => 0x00, Opcode::Sub => 0x28, Opcode::Cmp => 0x38, Opcode::Xor => 0x30,
                    _ => unreachable!(),
                };
                let sub_op_ext = match inst.opcode {
                    Opcode::Add => 0, Opcode::Sub => 5, Opcode::Cmp => 7, Opcode::Xor => 6,
                    _ => 0,
                };
                match (&inst.operands[0], &inst.operands[1]) {
                    (Operand::Reg(dst), Operand::Imm(imm)) => {
                        let d_info = sib::encode_reg(dst);
                        if let Some(rex) = sib::build_rex(d_info.is_wide, false, false, d_info.is_ext) { bytes.push(rex); }
                        let v = *imm;
                        if -128 <= v && v <= 127 {
                            bytes.push(0x83);
                            bytes.push(sib::modrm(3, sub_op_ext, d_info.val));
                            bytes.push(v as i8 as u8);
                        } else {
                            if d_info.val == 0 { // special optimized AL/AX/EAX/RAX
                                bytes.push(if d_info.is_8 { 0x04 | opc_base } else { 0x05 | opc_base });
                                bytes.extend_from_slice(&(v as i32).to_le_bytes()); // RAX also uses 32bit sign-extended
                            } else {
                                bytes.push(0x81);
                                bytes.push(sib::modrm(3, sub_op_ext, d_info.val));
                                bytes.extend_from_slice(&(v as i32).to_le_bytes());
                            }
                        }
                    }
                    (Operand::Reg(dst), Operand::Reg(src)) => {
                        let d_info = sib::encode_reg(dst);
                        let s_info = sib::encode_reg(src);
                        // Prefix 0x66 for 16 bit
                        if d_info.is_16 { bytes.push(0x66); }
                        
                        let w = if d_info.is_32 || d_info.is_16 || d_info.is_8 { false } else { true };
                        // optimization: XOR reg, reg (like xor ecx, ecx) drops REX.W
                        let w_actual = if inst.opcode == Opcode::Xor { false } else { w };
                        
                        if let Some(rex) = sib::build_rex(w_actual, s_info.is_ext, false, d_info.is_ext) { bytes.push(rex); }
                        bytes.push(opc_base + if d_info.is_8 { 0 } else { 1 });
                        bytes.push(sib::modrm(3, s_info.val, d_info.val));
                    }
                    _ => {}
                }
            }
        }
        
        // --- 3. Dynamic Memory Loading (MOV / LEA SIB mapping) ---
        Opcode::Mov => {
            if inst.operands.len() == 2 {
                match (&inst.operands[0], &inst.operands[1]) {
                    (Operand::Reg(dst), Operand::Reg(src)) => {
                        let d_info = sib::encode_reg(dst);
                        let s_info = sib::encode_reg(src);
                        if d_info.is_16 { bytes.push(0x66); }
                        let w = !d_info.is_8 && !d_info.is_16 && !d_info.is_32;
                        if let Some(rex) = sib::build_rex(w, s_info.is_ext, false, d_info.is_ext) { bytes.push(rex); }
                        bytes.push(0x88 + if d_info.is_8 { 0 } else { 1 });
                        bytes.push(sib::modrm(3, s_info.val, d_info.val));
                    }
                    (Operand::Reg(dst), Operand::Imm(imm)) => {
                        let d_info = sib::encode_reg(dst);
                        let v = *imm;
                        if d_info.is_16 { bytes.push(0x66); }
                        let w = !d_info.is_8 && !d_info.is_16 && !d_info.is_32;
                        
                        if w {
                            if v > i32::MAX as i64 || v < i32::MIN as i64 {
                                // 64 bit absolute mov B8+rd
                                if let Some(rex) = sib::build_rex(true, false, false, d_info.is_ext) { bytes.push(rex); }
                                bytes.push(0xB8 + d_info.val);
                                bytes.extend_from_slice(&v.to_le_bytes());
                            } else {
                                // 64 bit with 32 bit sign-extended immediate: C7 /0
                                if let Some(rex) = sib::build_rex(true, false, false, d_info.is_ext) { bytes.push(rex); }
                                bytes.push(0xC7);
                                bytes.push(sib::modrm(3, 0, d_info.val));
                                bytes.extend_from_slice(&(v as i32).to_le_bytes());
                            }
                        } else {
                            if let Some(rex) = sib::build_rex(false, false, false, d_info.is_ext) { bytes.push(rex); }
                            bytes.push(if d_info.is_8 { 0xB0 } else { 0xB8 } + d_info.val);
                            if d_info.is_8 { bytes.push(v as u8); } 
                            else if d_info.is_16 { bytes.extend_from_slice(&(v as u16).to_le_bytes()); }
                            else { bytes.extend_from_slice(&(v as i32).to_le_bytes()); }
                        }
                    }
                    (Operand::Reg(dst), Operand::Memory { base, index, scale, disp }) => {
                        let d_info = sib::encode_reg(dst);
                        let mem = sib::resolve_memory(d_info.val, base.as_ref(), index.as_ref(), *scale, *disp);
                        if d_info.is_16 { bytes.push(0x66); }
                        let w = !d_info.is_8 && !d_info.is_16 && !d_info.is_32;
                        if let Some(rex) = sib::build_rex(w, d_info.is_ext, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                        bytes.push(0x8A + if d_info.is_8 { 0 } else { 1 });
                        bytes.extend(mem.payload);
                    }
                    (Operand::Memory { base, index, scale, disp }, Operand::Reg(src)) => {
                        let s_info = sib::encode_reg(src);
                        let mem = sib::resolve_memory(s_info.val, base.as_ref(), index.as_ref(), *scale, *disp);
                        if s_info.is_16 { bytes.push(0x66); }
                        let w = !s_info.is_8 && !s_info.is_16 && !s_info.is_32;
                        if let Some(rex) = sib::build_rex(w, s_info.is_ext, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                        bytes.push(0x88 + if s_info.is_8 { 0 } else { 1 });
                        bytes.extend(mem.payload);
                    }
                    _ => {}
                }
            }
        }
        Opcode::Lea => {
            if let Some(Operand::Reg(dst)) = inst.operands.get(0) {
                let d_info = sib::encode_reg(dst);
                if let Some(Operand::Label(lbl)) = inst.operands.get(1) {
                    if let Some(rex) = sib::build_rex(d_info.is_wide, d_info.is_ext, false, false) { bytes.push(rex); }
                    bytes.push(0x8D);
                    bytes.push(sib::modrm(0, d_info.val, 5)); // RIP relative
                    bytes.extend_from_slice(&[0,0,0,0]);
                    relocations.push(RelocationReq { offset: bytes.len() as u32 - 4, symbol: lbl.clone(), rel_type: 4 });
                } else if let Some(Operand::Memory { base, index, scale, disp }) = inst.operands.get(1) {
                    let mem = sib::resolve_memory(d_info.val, base.as_ref(), index.as_ref(), *scale, *disp);
                    if let Some(rex) = sib::build_rex(d_info.is_wide, d_info.is_ext, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                    bytes.push(0x8D);
                    bytes.extend(mem.payload);
                }
            }
        }

        // --- 4. Advanced Floating Point (SSE / AVX2 VEX) ---
        Opcode::Cvtsi2ss | Opcode::Cvtss2si | Opcode::Sqrtss | Opcode::Movss | Opcode::Addss => {
            let (is_movss, is_addss, is_cvtsi) = (inst.opcode == Opcode::Movss, inst.opcode == Opcode::Addss, inst.opcode == Opcode::Cvtsi2ss);
            
            bytes.push(0xF3); // SSE Prefix
            
            if let Some(Operand::Reg(dst)) = inst.operands.get(0) {
                let d_info = sib::encode_reg(dst);
                let reg_val = d_info.val;
                let r_ext = d_info.is_ext;
                
                if let Some(Operand::Reg(src)) = inst.operands.get(1) {
                    let s_info = sib::encode_reg(src);
                    let (rex_r, rex_b) = if is_cvtsi { (d_info.is_ext, s_info.is_ext) } else { (d_info.is_ext, s_info.is_ext) };
                    let w = if is_cvtsi { s_info.is_wide } else { d_info.is_wide };
                    if let Some(rex) = sib::build_rex(w, rex_r, false, rex_b) { bytes.push(rex); }
                    
                    bytes.push(0x0F);
                    bytes.push(match inst.opcode {
                        Opcode::Cvtsi2ss => 0x2A, Opcode::Cvtss2si => 0x2D, Opcode::Sqrtss => 0x51,
                        Opcode::Movss => 0x10, Opcode::Addss => 0x58, _ => 0,
                    });
                    bytes.push(sib::modrm(3, d_info.val, s_info.val));
                    
                } else if let Some(Operand::Memory { base, index, scale, disp }) = inst.operands.get(1) {
                    let mem = sib::resolve_memory(d_info.val, base.as_ref(), index.as_ref(), *scale, *disp);
                    if let Some(rex) = sib::build_rex(false, d_info.is_ext, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                    bytes.push(0x0F);
                    bytes.push(match inst.opcode { Opcode::Movss => 0x10, Opcode::Addss => 0x58, _ => 0 });
                    bytes.extend(mem.payload);
                }
            }
        }
        
        Opcode::Vaddps => {
            // AVX implementation example: vaddps ymm0, ymm1, ymm2
            if let (Some(Operand::Reg(dst)), Some(Operand::Reg(src1)), Some(Operand::Reg(src2))) = (inst.operands.get(0), inst.operands.get(1), inst.operands.get(2)) {
                let d = sib::encode_reg(dst);
                let s1 = sib::encode_reg(src1);
                let s2 = sib::encode_reg(src2);
                
                let is_256 = true; // YMM usage
                let vex = vex::build_vex(false, !d.is_ext, !s2.is_ext, !s2.is_ext, 1, Some(src1), is_256, 0);
                bytes.extend(vex);
                bytes.push(0x58);
                bytes.push(sib::modrm(3, d.val, s2.val));
            }
        }

        // --- 5. Flow Control (JMP, Calls, Terminals) ---
        Opcode::Jmp | Opcode::Je | Opcode::Jne => {
            if let Some(Operand::Label(lbl)) = inst.operands.get(0) {
                let mut resolved_locally = false;
                
                if let Some(map) = labels {
                    if let Some(&target_offset) = map.get(lbl) {
                        // Pass 2: We use invariant 32-bit near jumps to match Pass 1 sizes perfectly
                        // but we resolve the relative delta LOCALLY instead of asking the Linker.
                        let near_jump_size = if inst.opcode == Opcode::Jmp { 5 } else { 6 };
                        let delta32 = (target_offset as i64) - ((current_offset + near_jump_size) as i64);
                        
                        if inst.opcode == Opcode::Jmp {
                            bytes.push(0xE9);
                        } else {
                            bytes.push(0x0F);
                            bytes.push(if inst.opcode == Opcode::Je { 0x84 } else { 0x85 });
                        }
                        bytes.extend_from_slice(&(delta32 as i32).to_le_bytes());
                        resolved_locally = true;
                    }
                }
                
                if !resolved_locally {
                    if inst.opcode == Opcode::Jmp {
                        bytes.push(0xE9);
                    } else {
                        bytes.push(0x0F);
                        bytes.push(if inst.opcode == Opcode::Je { 0x84 } else { 0x85 });
                    }
                    bytes.extend_from_slice(&[0,0,0,0]);
                    relocations.push(RelocationReq { offset: bytes.len() as u32 - 4, symbol: lbl.clone(), rel_type: 4 });
                }
            }
        }
        Opcode::Call => {
            if let Some(Operand::Label(lbl)) = inst.operands.get(0) {
                bytes.push(0xE8);
                bytes.extend_from_slice(&[0,0,0,0]);
                relocations.push(RelocationReq { offset: 1, symbol: lbl.clone(), rel_type: 4 });
            } else if let Some(Operand::Memory { base, index, scale, disp }) = inst.operands.get(0) {
                // Invocación a Funciones de VTable COM DirectX 12 
                // e.g. CALL QWORD PTR [RAX + 0x60] -> FF /2
                let mem = sib::resolve_memory(2, base.as_ref(), index.as_ref(), *scale, *disp);
                if let Some(rex) = sib::build_rex(false, false, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                bytes.push(0xFF);
                bytes.extend(mem.payload);
            } else if let Some(Operand::Reg(r)) = inst.operands.get(0) {
                // Invocación a registro: CALL RCX -> FF D1
                let ri = sib::encode_reg(r);
                if let Some(rex) = sib::build_rex(false, false, false, ri.is_ext) { bytes.push(rex); }
                bytes.push(0xFF);
                bytes.push(sib::modrm(3, 2, ri.val));
            }
        }
        Opcode::Dec => {
            if let Some(Operand::Reg(r)) = inst.operands.get(0) {
                let ri = sib::encode_reg(r);
                if let Some(rex) = sib::build_rex(ri.is_wide, false, false, ri.is_ext) { bytes.push(rex); }
                bytes.push(0xFF); bytes.push(sib::modrm(3, 1, ri.val));
            }
        }
        Opcode::Imul => {
            if let (Some(Operand::Reg(d)), Some(Operand::Reg(s))) = (inst.operands.get(0), inst.operands.get(1)) {
                let di = sib::encode_reg(d); let si = sib::encode_reg(s);
                if let Some(rex) = sib::build_rex(di.is_wide, di.is_ext, false, si.is_ext) { bytes.push(rex); }
                bytes.push(0x0F); bytes.push(0xAF); bytes.push(sib::modrm(3, di.val, si.val));
            }
        }
        Opcode::Test => {
            if let (Some(Operand::Reg(d)), Some(Operand::Reg(s))) = (inst.operands.get(0), inst.operands.get(1)) {
                let di = sib::encode_reg(d); let si = sib::encode_reg(s);
                if let Some(rex) = sib::build_rex(di.is_wide, si.is_ext, false, di.is_ext) { bytes.push(rex); }
                bytes.push(0x85); bytes.push(sib::modrm(3, si.val, di.val));
            }
        }

        _ => return Err(format!("Unimplemented binary encoding for {:?}", inst.opcode)),
    }

    Ok(EncodedInstruction { bytes, relocations })
}
