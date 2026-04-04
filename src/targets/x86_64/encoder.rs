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
            } else if let Some(Operand::Imm(v)) = inst.operands.get(0) {
                let v = *v;
                if -128 <= v && v <= 127 {
                    bytes.push(0x6A);
                    bytes.push(v as i8 as u8);
                } else {
                    bytes.push(0x68);
                    bytes.extend_from_slice(&(v as i32).to_le_bytes());
                }
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
                        if !d_info.is_8 && -128 <= v && v <= 127 {
                            bytes.push(0x83);
                            bytes.push(sib::modrm(3, sub_op_ext, d_info.val));
                            bytes.push(v as i8 as u8);
                        } else if d_info.is_8 {
                            bytes.push(0x80);
                            bytes.push(sib::modrm(3, sub_op_ext, d_info.val));
                            bytes.push(v as u8);
                        } else {
                            if d_info.val == 0 {
                                bytes.push(opc_base + 5);
                                bytes.extend_from_slice(&(v as i32).to_le_bytes());
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
                        if d_info.is_16 { bytes.push(0x66); }
                        let w = !d_info.is_8 && !d_info.is_16 && !d_info.is_32;
                        let w_actual = if inst.opcode == Opcode::Xor && d_info.is_32 { false } else { w };
                        if let Some(rex) = sib::build_rex(w_actual, s_info.is_ext, false, d_info.is_ext) { bytes.push(rex); }
                        bytes.push(opc_base + if d_info.is_8 { 0 } else { 1 });
                        bytes.push(sib::modrm(3, s_info.val, d_info.val));
                    }
                    // FASE 7: reg, mem — e.g. add rax, [rbx+8]
                    (Operand::Reg(dst), Operand::Memory { base, index, scale, disp, .. }) => {
                        let d_info = sib::encode_reg(dst);
                        let mem = sib::resolve_memory(d_info.val, base.as_ref(), index.as_ref(), *scale, *disp);
                        if d_info.is_16 { bytes.push(0x66); }
                        let w = !d_info.is_8 && !d_info.is_16 && !d_info.is_32;
                        if let Some(rex) = sib::build_rex(w, d_info.is_ext, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                        bytes.push(opc_base + if d_info.is_8 { 2 } else { 3 });
                        bytes.extend(mem.payload);
                    }
                    // FASE 7: mem, reg — e.g. add [rsp+0x20], rcx
                    (Operand::Memory { base, index, scale, disp, .. }, Operand::Reg(src)) => {
                        let s_info = sib::encode_reg(src);
                        let mem = sib::resolve_memory(s_info.val, base.as_ref(), index.as_ref(), *scale, *disp);
                        if s_info.is_16 { bytes.push(0x66); }
                        let w = !s_info.is_8 && !s_info.is_16 && !s_info.is_32;
                        if let Some(rex) = sib::build_rex(w, s_info.is_ext, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                        bytes.push(opc_base + if s_info.is_8 { 0 } else { 1 });
                        bytes.extend(mem.payload);
                    }
                    // FASE 7+9: mem, imm — e.g. sub DWORD PTR [rsp+4], 1
                    (Operand::Memory { base, index, scale, disp, size }, Operand::Imm(imm)) => {
                        let sz = size.unwrap_or(8); // Default to QWORD
                        let mem = sib::resolve_memory(sub_op_ext, base.as_ref(), index.as_ref(), *scale, *disp);
                        let v = *imm;
                        
                        if sz == 1 {
                            if let Some(rex) = sib::build_rex(false, false, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                            bytes.push(0x80);
                            bytes.extend(mem.payload);
                            bytes.push(v as u8);
                        } else {
                            let (prefix, w) = match sz {
                                2 => (Some(0x66), false),
                                4 => (None, false),
                                _ => (None, true),
                            };
                            if let Some(p) = prefix { bytes.push(p); }
                            if let Some(rex) = sib::build_rex(w, false, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                            
                            if -128 <= v && v <= 127 {
                                bytes.push(0x83);
                                bytes.extend(mem.payload);
                                bytes.push(v as i8 as u8);
                            } else {
                                bytes.push(0x81);
                                bytes.extend(mem.payload);
                                if sz == 2 {
                                    bytes.extend_from_slice(&(v as i16).to_le_bytes());
                                } else {
                                    bytes.extend_from_slice(&(v as i32).to_le_bytes());
                                }
                            }
                        }
                    }
                    // FASE 7: reg, label (RIP-relative) — e.g. cmp rax, [global_var]
                    (Operand::Reg(dst), Operand::Label(lbl)) => {
                        let d_info = sib::encode_reg(dst);
                        let w = !d_info.is_8 && !d_info.is_16 && !d_info.is_32;
                        if let Some(rex) = sib::build_rex(w, d_info.is_ext, false, false) { bytes.push(rex); }
                        bytes.push(opc_base + if d_info.is_8 { 2 } else { 3 });
                        bytes.push(sib::modrm(0, d_info.val, 5));
                        bytes.extend_from_slice(&[0,0,0,0]);
                        relocations.push(RelocationReq { offset: bytes.len() as u32 - 4, symbol: lbl.clone(), rel_type: 4 });
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
                    (Operand::Reg(dst), Operand::Memory { base, index, scale, disp, .. }) => {
                        let d_info = sib::encode_reg(dst);
                        let mem = sib::resolve_memory(d_info.val, base.as_ref(), index.as_ref(), *scale, *disp);
                        if d_info.is_16 { bytes.push(0x66); }
                        let w = !d_info.is_8 && !d_info.is_16 && !d_info.is_32;
                        if let Some(rex) = sib::build_rex(w, d_info.is_ext, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                        bytes.push(0x8A + if d_info.is_8 { 0 } else { 1 });
                        bytes.extend(mem.payload);
                    }
                    (Operand::Memory { base, index, scale, disp, .. }, Operand::Reg(src)) => {
                        let s_info = sib::encode_reg(src);
                        let mem = sib::resolve_memory(s_info.val, base.as_ref(), index.as_ref(), *scale, *disp);
                        if s_info.is_16 { bytes.push(0x66); }
                        let w = !s_info.is_8 && !s_info.is_16 && !s_info.is_32;
                        if let Some(rex) = sib::build_rex(w, s_info.is_ext, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                        bytes.push(0x88 + if s_info.is_8 { 0 } else { 1 });
                        bytes.extend(mem.payload);
                    }
                    (Operand::Reg(dst), Operand::Label(lbl)) => {
                        let d_info = sib::encode_reg(dst);
                        if d_info.is_16 { bytes.push(0x66); }
                        let w = !d_info.is_8 && !d_info.is_16 && !d_info.is_32;
                        if let Some(rex) = sib::build_rex(w, d_info.is_ext, false, false) { bytes.push(rex); }
                        bytes.push(if d_info.is_8 { 0x8A } else { 0x8B });
                        bytes.push(sib::modrm(0, d_info.val, 5)); // RIP relative
                        bytes.extend_from_slice(&[0,0,0,0]);
                        relocations.push(RelocationReq { offset: bytes.len() as u32 - 4, symbol: lbl.clone(), rel_type: 4 });
                    }
                    (Operand::Label(lbl), Operand::Reg(src)) => {
                        let s_info = sib::encode_reg(src);
                        if s_info.is_16 { bytes.push(0x66); }
                        let w = !s_info.is_8 && !s_info.is_16 && !s_info.is_32;
                        if let Some(rex) = sib::build_rex(w, s_info.is_ext, false, false) { bytes.push(rex); }
                        bytes.push(if s_info.is_8 { 0x88 } else { 0x89 });
                        bytes.push(sib::modrm(0, s_info.val, 5));
                        bytes.extend_from_slice(&[0,0,0,0]);
                        relocations.push(RelocationReq { offset: bytes.len() as u32 - 4, symbol: lbl.clone(), rel_type: 4 });
                    }
                    // FASE 7+9: MOV mem, imm — e.g. mov QWORD PTR [rsp+8], 0
                    (Operand::Memory { base, index, scale, disp, size }, Operand::Imm(imm)) => {
                        let sz = size.unwrap_or(8); // Default to QWORD
                        let mem = sib::resolve_memory(0, base.as_ref(), index.as_ref(), *scale, *disp);
                        
                        if sz == 1 {
                            if let Some(rex) = sib::build_rex(false, false, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                            bytes.push(0xC6);
                            bytes.extend(mem.payload);
                            bytes.push(*imm as u8);
                        } else if sz == 2 {
                            bytes.push(0x66);
                            if let Some(rex) = sib::build_rex(false, false, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                            bytes.push(0xC7);
                            bytes.extend(mem.payload);
                            bytes.extend_from_slice(&(*imm as i16).to_le_bytes());
                        } else if sz == 4 {
                            if let Some(rex) = sib::build_rex(false, false, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                            bytes.push(0xC7);
                            bytes.extend(mem.payload);
                            bytes.extend_from_slice(&(*imm as i32).to_le_bytes());
                        } else {
                            if let Some(rex) = sib::build_rex(true, false, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                            bytes.push(0xC7);
                            bytes.extend(mem.payload);
                            bytes.extend_from_slice(&(*imm as i32).to_le_bytes());
                        }
                    }
                    // FASE 7: MOV label, imm — e.g. mov [global_var], 42
                    (Operand::Label(lbl), Operand::Imm(imm)) => {
                        if let Some(rex) = sib::build_rex(true, false, false, false) { bytes.push(rex); }
                        bytes.push(0xC7);
                        bytes.push(sib::modrm(0, 0, 5));
                        bytes.extend_from_slice(&[0,0,0,0]);
                        relocations.push(RelocationReq { offset: bytes.len() as u32 - 4, symbol: lbl.clone(), rel_type: 4 });
                        bytes.extend_from_slice(&(*imm as i32).to_le_bytes());
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
                } else if let Some(Operand::Memory { base, index, scale, disp, .. }) = inst.operands.get(1) {
                    let mem = sib::resolve_memory(d_info.val, base.as_ref(), index.as_ref(), *scale, *disp);
                    if let Some(rex) = sib::build_rex(d_info.is_wide, d_info.is_ext, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                    bytes.push(0x8D);
                    bytes.extend(mem.payload);
                }
            }
        }

        // --- 4. Advanced Floating Point (SSE / AVX2 VEX) ---
        Opcode::Cvtsi2ss | Opcode::Cvtss2si | Opcode::Cvttss2si | Opcode::Sqrtss | Opcode::Movss | Opcode::Addss => {
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
                        Opcode::Cvtsi2ss => 0x2A, Opcode::Cvtss2si => 0x2D, Opcode::Cvttss2si => 0x2C, Opcode::Sqrtss => 0x51,
                        Opcode::Movss => 0x10, Opcode::Addss => 0x58, _ => 0,
                    });
                    bytes.push(sib::modrm(3, d_info.val, s_info.val));
                    
                } else if let Some(Operand::Memory { base, index, scale, disp, .. }) = inst.operands.get(1) {
                    let w = d_info.is_wide; // Normally CVTSx2SI / CVTSI2SS might differ, but for Memory we assume 32-bit (false) unless wide dest.
                    let mem = sib::resolve_memory(d_info.val, base.as_ref(), index.as_ref(), *scale, *disp);
                    if let Some(rex) = sib::build_rex(w, d_info.is_ext, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                    bytes.push(0x0F);
                    bytes.push(match inst.opcode { 
                        Opcode::Cvtsi2ss => 0x2A, Opcode::Cvtss2si => 0x2D, Opcode::Cvttss2si => 0x2C, Opcode::Sqrtss => 0x51,
                        Opcode::Movss => 0x10, Opcode::Addss => 0x58, _ => 0 
                    });
                    bytes.extend(mem.payload);
                } else if let Some(Operand::Label(lbl)) = inst.operands.get(1) {
                    let w = d_info.is_wide;
                    if let Some(rex) = sib::build_rex(w, d_info.is_ext, false, false) { bytes.push(rex); }
                    bytes.push(0x0F);
                    bytes.push(match inst.opcode { 
                        Opcode::Cvtsi2ss => 0x2A, Opcode::Cvtss2si => 0x2D, Opcode::Cvttss2si => 0x2C, Opcode::Sqrtss => 0x51,
                        Opcode::Movss => 0x10, Opcode::Addss => 0x58, _ => 0 
                    });
                    bytes.push(sib::modrm(0, d_info.val, 5)); // RIP-relative Reg, 101
                    bytes.extend_from_slice(&[0,0,0,0]);
                    relocations.push(RelocationReq { offset: bytes.len() as u32 - 4, symbol: lbl.clone(), rel_type: 4 });
                }
            // FASE 7: MOVSS mem, xmm — F3 0F 11 (store direction)
            } else if let (Some(Operand::Memory { base, index, scale, disp, .. }), Some(Operand::Reg(src))) = (inst.operands.get(0), inst.operands.get(1)) {
                if inst.opcode == Opcode::Movss {
                    let s_info = sib::encode_reg(src);
                    let mem = sib::resolve_memory(s_info.val, base.as_ref(), index.as_ref(), *scale, *disp);
                    if let Some(rex) = sib::build_rex(false, s_info.is_ext, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                    bytes.push(0x0F); bytes.push(0x11); // store opcode
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
            } else if let Some(Operand::Memory { base, index, scale, disp, .. }) = inst.operands.get(0) {
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
            if inst.operands.len() == 3 {
                // FASE 7: IMUL reg, reg, imm (3-operand) — 6B/69
                if let (Some(Operand::Reg(d)), Some(Operand::Reg(s)), Some(Operand::Imm(imm))) =
                    (inst.operands.get(0), inst.operands.get(1), inst.operands.get(2)) {
                    let di = sib::encode_reg(d); let si = sib::encode_reg(s);
                    if let Some(rex) = sib::build_rex(di.is_wide, di.is_ext, false, si.is_ext) { bytes.push(rex); }
                    let v = *imm;
                    if -128 <= v && v <= 127 {
                        bytes.push(0x6B);
                        bytes.push(sib::modrm(3, di.val, si.val));
                        bytes.push(v as i8 as u8);
                    } else {
                        bytes.push(0x69);
                        bytes.push(sib::modrm(3, di.val, si.val));
                        bytes.extend_from_slice(&(v as i32).to_le_bytes());
                    }
                }
            } else if inst.operands.len() == 2 {
                if let (Some(Operand::Reg(d)), Some(Operand::Reg(s))) = (inst.operands.get(0), inst.operands.get(1)) {
                    // IMUL reg, reg — 0F AF /r
                    let di = sib::encode_reg(d); let si = sib::encode_reg(s);
                    if let Some(rex) = sib::build_rex(di.is_wide, di.is_ext, false, si.is_ext) { bytes.push(rex); }
                    bytes.push(0x0F); bytes.push(0xAF); bytes.push(sib::modrm(3, di.val, si.val));
                } else if let (Some(Operand::Reg(d)), Some(Operand::Memory { base, index, scale, disp, .. })) = (inst.operands.get(0), inst.operands.get(1)) {
                    // IMUL reg, mem — 0F AF /r
                    let di = sib::encode_reg(d);
                    let mem = sib::resolve_memory(di.val, base.as_ref(), index.as_ref(), *scale, *disp);
                    if let Some(rex) = sib::build_rex(di.is_wide, di.is_ext, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                    bytes.push(0x0F); bytes.push(0xAF);
                    bytes.extend(mem.payload);
                } else if let (Some(Operand::Reg(d)), Some(Operand::Imm(imm))) = (inst.operands.get(0), inst.operands.get(1)) {
                    // IMUL reg, imm — same as IMUL reg, reg, imm with dst=src
                    let di = sib::encode_reg(d);
                    if let Some(rex) = sib::build_rex(di.is_wide, di.is_ext, false, di.is_ext) { bytes.push(rex); }
                    let v = *imm;
                    if -128 <= v && v <= 127 {
                        bytes.push(0x6B);
                        bytes.push(sib::modrm(3, di.val, di.val));
                        bytes.push(v as i8 as u8);
                    } else {
                        bytes.push(0x69);
                        bytes.push(sib::modrm(3, di.val, di.val));
                        bytes.extend_from_slice(&(v as i32).to_le_bytes());
                    }
                }
            } else if inst.operands.len() == 1 {
                // IMUL reg (single operand: RDX:RAX = RAX * reg) — F7 /5
                if let Some(Operand::Reg(r)) = inst.operands.get(0) {
                    let ri = sib::encode_reg(r);
                    if let Some(rex) = sib::build_rex(ri.is_wide, false, false, ri.is_ext) { bytes.push(rex); }
                    bytes.push(0xF7); bytes.push(sib::modrm(3, 5, ri.val));
                }
            }
        }
        Opcode::Test => {
            if let (Some(Operand::Reg(d)), Some(src)) = (inst.operands.get(0), inst.operands.get(1)) {
                let di = sib::encode_reg(d);
                match src {
                    Operand::Reg(s) => {
                        let si = sib::encode_reg(s);
                        if let Some(rex) = sib::build_rex(di.is_wide, si.is_ext, false, di.is_ext) { bytes.push(rex); }
                        bytes.push(if di.is_8 { 0x84 } else { 0x85 });
                        bytes.push(sib::modrm(3, si.val, di.val));
                    }
                    // FASE 7: TEST reg, imm — F7 /0 (or A9 for RAX)
                    Operand::Imm(imm) => {
                        let v = *imm;
                        if let Some(rex) = sib::build_rex(di.is_wide, false, false, di.is_ext) { bytes.push(rex); }
                        if di.val == 0 {
                            // Optimized: TEST AL/EAX/RAX, imm
                            bytes.push(if di.is_8 { 0xA8 } else { 0xA9 });
                        } else {
                            bytes.push(if di.is_8 { 0xF6 } else { 0xF7 });
                            bytes.push(sib::modrm(3, 0, di.val));
                        }
                        if di.is_8 { bytes.push(v as u8); }
                        else { bytes.extend_from_slice(&(v as i32).to_le_bytes()); }
                    }
                    // FASE 7: TEST mem, reg
                    Operand::Memory { base, index, scale, disp, .. } => {
                        let mem = sib::resolve_memory(di.val, base.as_ref(), index.as_ref(), *scale, *disp);
                        if let Some(rex) = sib::build_rex(di.is_wide, di.is_ext, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                        bytes.push(if di.is_8 { 0x84 } else { 0x85 });
                        bytes.extend(mem.payload);
                    }
                    _ => {}
                }
            }
        }
        // === FASE 3: Core MSVC Completeness ===
        
        // --- INC (FF /0) ---
        Opcode::Inc => {
            if let Some(Operand::Reg(r)) = inst.operands.get(0) {
                let ri = sib::encode_reg(r);
                if let Some(rex) = sib::build_rex(ri.is_wide, false, false, ri.is_ext) { bytes.push(rex); }
                bytes.push(0xFF); bytes.push(sib::modrm(3, 0, ri.val));
            }
        }
        
        // --- NEG (F7 /3), NOT (F7 /2), MUL (F7 /4), DIV (F7 /6), IDIV (F7 /7) ---
        Opcode::Neg | Opcode::Not | Opcode::Mul | Opcode::Div | Opcode::Idiv => {
            if let Some(Operand::Reg(r)) = inst.operands.get(0) {
                let ri = sib::encode_reg(r);
                let ext = match inst.opcode {
                    Opcode::Neg => 3, Opcode::Not => 2, Opcode::Mul => 4,
                    Opcode::Div => 6, Opcode::Idiv => 7, _ => 0,
                };
                if let Some(rex) = sib::build_rex(ri.is_wide, false, false, ri.is_ext) { bytes.push(rex); }
                bytes.push(if ri.is_8 { 0xF6 } else { 0xF7 });
                bytes.push(sib::modrm(3, ext, ri.val));
            }
        }
        
        // --- AND, OR, ADC, SBB (extend ADD/SUB/CMP/XOR pattern) ---
        Opcode::And | Opcode::Or | Opcode::Adc | Opcode::Sbb => {
            if inst.operands.len() == 2 {
                let (opc_base, sub_op_ext) = match inst.opcode {
                    Opcode::And => (0x20, 4u8), Opcode::Or  => (0x08, 1u8),
                    Opcode::Adc => (0x10, 2u8), Opcode::Sbb => (0x18, 3u8),
                    _ => unreachable!(),
                };
                match (&inst.operands[0], &inst.operands[1]) {
                    (Operand::Reg(dst), Operand::Imm(imm)) => {
                        let d_info = sib::encode_reg(dst);
                        if let Some(rex) = sib::build_rex(d_info.is_wide, false, false, d_info.is_ext) { bytes.push(rex); }
                        let v = *imm;
                        if !d_info.is_8 && -128 <= v && v <= 127 {
                            bytes.push(0x83);
                            bytes.push(sib::modrm(3, sub_op_ext, d_info.val));
                            bytes.push(v as i8 as u8);
                        } else if d_info.is_8 {
                            bytes.push(0x80);
                            bytes.push(sib::modrm(3, sub_op_ext, d_info.val));
                            bytes.push(v as u8);
                        } else {
                            if d_info.val == 0 {
                                bytes.push(opc_base + 5);
                                bytes.extend_from_slice(&(v as i32).to_le_bytes());
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
                        if d_info.is_16 { bytes.push(0x66); }
                        let w = !d_info.is_8 && !d_info.is_16 && !d_info.is_32;
                        if let Some(rex) = sib::build_rex(w, s_info.is_ext, false, d_info.is_ext) { bytes.push(rex); }
                        bytes.push(opc_base + if d_info.is_8 { 0 } else { 1 });
                        bytes.push(sib::modrm(3, s_info.val, d_info.val));
                    }
                    (Operand::Reg(dst), Operand::Memory { base, index, scale, disp, .. }) => {
                        let d_info = sib::encode_reg(dst);
                        let mem = sib::resolve_memory(d_info.val, base.as_ref(), index.as_ref(), *scale, *disp);
                        let w = !d_info.is_8 && !d_info.is_16 && !d_info.is_32;
                        if let Some(rex) = sib::build_rex(w, d_info.is_ext, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                        bytes.push(opc_base + if d_info.is_8 { 2 } else { 3 });
                        bytes.extend(mem.payload);
                    }
                    _ => {}
                }
            }
        }
        
        // --- SHL, SHR, SAR, ROL, ROR, RCL, RCR ---
        Opcode::Shl | Opcode::Shr | Opcode::Sar | Opcode::Rol | Opcode::Ror | Opcode::Rcl | Opcode::Rcr => {
            if let Some(Operand::Reg(dst)) = inst.operands.get(0) {
                let di = sib::encode_reg(dst);
                let ext = match inst.opcode {
                    Opcode::Rol => 0, Opcode::Ror => 1, Opcode::Rcl => 2, Opcode::Rcr => 3,
                    Opcode::Shl => 4, Opcode::Shr => 5, Opcode::Sar => 7, _ => 0,
                };
                if let Some(rex) = sib::build_rex(di.is_wide, false, false, di.is_ext) { bytes.push(rex); }
                
                match inst.operands.get(1) {
                    Some(Operand::Imm(1)) => {
                        bytes.push(if di.is_8 { 0xD0 } else { 0xD1 });
                        bytes.push(sib::modrm(3, ext, di.val));
                    }
                    Some(Operand::Imm(n)) => {
                        bytes.push(if di.is_8 { 0xC0 } else { 0xC1 });
                        bytes.push(sib::modrm(3, ext, di.val));
                        bytes.push(*n as u8);
                    }
                    Some(Operand::Reg(_cl)) => {
                        // shift by CL register
                        bytes.push(if di.is_8 { 0xD2 } else { 0xD3 });
                        bytes.push(sib::modrm(3, ext, di.val));
                    }
                    _ => {
                        // Default: shift by 1
                        bytes.push(if di.is_8 { 0xD0 } else { 0xD1 });
                        bytes.push(sib::modrm(3, ext, di.val));
                    }
                }
            }
        }
        
        // --- MOVZX (0F B6/B7) ---
        Opcode::Movzx => {
            if let (Some(Operand::Reg(dst)), Some(src)) = (inst.operands.get(0), inst.operands.get(1)) {
                let di = sib::encode_reg(dst);
                match src {
                    Operand::Reg(s) => {
                        let si = sib::encode_reg(s);
                        let is_byte_src = si.is_8;
                        if let Some(rex) = sib::build_rex(di.is_wide, di.is_ext, false, si.is_ext) { bytes.push(rex); }
                        bytes.push(0x0F);
                        bytes.push(if is_byte_src { 0xB6 } else { 0xB7 }); // B6=byte, B7=word
                        bytes.push(sib::modrm(3, di.val, si.val));
                    }
                    Operand::Memory { base, index, scale, disp, .. } => {
                        let mem = sib::resolve_memory(di.val, base.as_ref(), index.as_ref(), *scale, *disp);
                        if let Some(rex) = sib::build_rex(di.is_wide, di.is_ext, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                        bytes.push(0x0F);
                        bytes.push(0xB6); // Assume byte source for memory
                        bytes.extend(mem.payload);
                    }
                    _ => {}
                }
            }
        }
        
        // --- MOVSX (0F BE/BF) ---
        Opcode::Movsx => {
            if let (Some(Operand::Reg(dst)), Some(src)) = (inst.operands.get(0), inst.operands.get(1)) {
                let di = sib::encode_reg(dst);
                match src {
                    Operand::Reg(s) => {
                        let si = sib::encode_reg(s);
                        let is_byte_src = si.is_8;
                        if let Some(rex) = sib::build_rex(di.is_wide, di.is_ext, false, si.is_ext) { bytes.push(rex); }
                        bytes.push(0x0F);
                        bytes.push(if is_byte_src { 0xBE } else { 0xBF });
                        bytes.push(sib::modrm(3, di.val, si.val));
                    }
                    Operand::Memory { base, index, scale, disp, .. } => {
                        let mem = sib::resolve_memory(di.val, base.as_ref(), index.as_ref(), *scale, *disp);
                        if let Some(rex) = sib::build_rex(di.is_wide, di.is_ext, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                        bytes.push(0x0F);
                        bytes.push(0xBE);
                        bytes.extend(mem.payload);
                    }
                    _ => {}
                }
            }
        }
        
        // --- XCHG (87 /r) ---
        Opcode::Xchg => {
            if let (Some(Operand::Reg(a)), Some(Operand::Reg(b))) = (inst.operands.get(0), inst.operands.get(1)) {
                let ai = sib::encode_reg(a);
                let bi = sib::encode_reg(b);
                if let Some(rex) = sib::build_rex(ai.is_wide, ai.is_ext, false, bi.is_ext) { bytes.push(rex); }
                bytes.push(if ai.is_8 { 0x86 } else { 0x87 });
                bytes.push(sib::modrm(3, ai.val, bi.val));
            }
        }
        

        
        // --- Zero-operand instructions ---
        Opcode::Nop    => { bytes.push(0x90); }
        Opcode::Cqo    => { bytes.push(0x48); bytes.push(0x99); }
        Opcode::Cdq    => { bytes.push(0x99); }
        Opcode::Cbw    => { bytes.push(0x66); bytes.push(0x98); }
        Opcode::Cwd    => { bytes.push(0x66); bytes.push(0x99); }
        Opcode::Cwde   => { bytes.push(0x98); }
        Opcode::Syscall => { bytes.push(0x0F); bytes.push(0x05); }
        Opcode::Hlt    => { bytes.push(0xF4); }
        Opcode::Cli    => { bytes.push(0xFA); }
        Opcode::Sti    => { bytes.push(0xFB); }
        Opcode::Cpuid  => { bytes.push(0x0F); bytes.push(0xA2); }
        Opcode::Rdtsc  => { bytes.push(0x0F); bytes.push(0x31); }
        Opcode::Rdtscp => { bytes.push(0x0F); bytes.push(0x01); bytes.push(0xF9); }
        Opcode::Cld    => { bytes.push(0xFC); }
        Opcode::Std    => { bytes.push(0xFD); }
        Opcode::Iretq  => { bytes.push(0x48); bytes.push(0xCF); }
        Opcode::Lahf   => { bytes.push(0x9F); }
        Opcode::Sahf   => { bytes.push(0x9E); }
        Opcode::Xlat   => { bytes.push(0xD7); }
        Opcode::Pushf  => { bytes.push(0x9C); }
        Opcode::Popf   => { bytes.push(0x9D); }
        Opcode::Vzeroall  => { bytes.extend_from_slice(&[0xC5, 0xFC, 0x77]); }
        Opcode::Vzeroupper => { bytes.extend_from_slice(&[0xC5, 0xF8, 0x77]); }
        
        // --- INT imm8 ---
        Opcode::Int => {
            if let Some(Operand::Imm(v)) = inst.operands.get(0) {
                if *v == 3 { bytes.push(0xCC); }
                else { bytes.push(0xCD); bytes.push(*v as u8); }
            }
        }
        
        // --- BSWAP (0F C8+rd) ---
        Opcode::Bswap => {
            if let Some(Operand::Reg(r)) = inst.operands.get(0) {
                let ri = sib::encode_reg(r);
                if let Some(rex) = sib::build_rex(ri.is_wide, false, false, ri.is_ext) { bytes.push(rex); }
                bytes.push(0x0F);
                bytes.push(0xC8 + ri.val);
            }
        }
        
        // === FASE 4: Conditional Branches + SETcc + CMOVcc ===
        
        // --- All conditional jumps (0F 8x family) ---
        Opcode::Jl | Opcode::Jle | Opcode::Jg | Opcode::Jge |
        Opcode::Jb | Opcode::Jbe | Opcode::Ja | Opcode::Jae |
        Opcode::Js | Opcode::Jns | Opcode::Jo | Opcode::Jno |
        Opcode::Jp | Opcode::Jnp => {
            let cc_byte: u8 = match inst.opcode {
                Opcode::Jo  => 0x80, Opcode::Jno => 0x81,
                Opcode::Jb  => 0x82, Opcode::Jae => 0x83,
                Opcode::Js  => 0x88, Opcode::Jns => 0x89,
                Opcode::Jp  => 0x8A, Opcode::Jnp => 0x8B,
                Opcode::Jl  => 0x8C, Opcode::Jge => 0x8D,
                Opcode::Jle => 0x8E, Opcode::Jg  => 0x8F,
                Opcode::Jbe => 0x86, Opcode::Ja  => 0x87,
                _ => unreachable!(),
            };
            if let Some(Operand::Label(lbl)) = inst.operands.get(0) {
                let mut resolved_locally = false;
                if let Some(map) = labels {
                    if let Some(&target_offset) = map.get(lbl) {
                        let near_jump_size = 6u32; // 0F 8x + 4-byte displacement
                        let delta = (target_offset as i64) - ((current_offset + near_jump_size) as i64);
                        bytes.push(0x0F);
                        bytes.push(cc_byte);
                        bytes.extend_from_slice(&(delta as i32).to_le_bytes());
                        resolved_locally = true;
                    }
                }
                if !resolved_locally {
                    bytes.push(0x0F);
                    bytes.push(cc_byte);
                    bytes.extend_from_slice(&[0,0,0,0]);
                    relocations.push(RelocationReq { offset: bytes.len() as u32 - 4, symbol: lbl.clone(), rel_type: 4 });
                }
            }
        }
        
        // --- SETcc (0F 9x /0) ---
        Opcode::Sete | Opcode::Setne | Opcode::Setl | Opcode::Setle |
        Opcode::Setg | Opcode::Setge | Opcode::Setb | Opcode::Setbe |
        Opcode::Seta | Opcode::Setae | Opcode::Sets | Opcode::Setns => {
            let cc_byte: u8 = match inst.opcode {
                Opcode::Sete  => 0x94, Opcode::Setne => 0x95,
                Opcode::Setl  => 0x9C, Opcode::Setle => 0x9E,
                Opcode::Setg  => 0x9F, Opcode::Setge => 0x9D,
                Opcode::Setb  => 0x92, Opcode::Setbe => 0x96,
                Opcode::Seta  => 0x97, Opcode::Setae => 0x93,
                Opcode::Sets  => 0x98, Opcode::Setns => 0x99,
                _ => unreachable!(),
            };
            if let Some(Operand::Reg(r)) = inst.operands.get(0) {
                let ri = sib::encode_reg(r);
                if ri.is_ext { bytes.push(0x41); } // REX.B for r8-r15 byte regs
                bytes.push(0x0F);
                bytes.push(cc_byte);
                bytes.push(sib::modrm(3, 0, ri.val));
            }
        }
        
        // --- CMOVcc (0F 4x /r) ---
        Opcode::Cmove | Opcode::Cmovne | Opcode::Cmovl | Opcode::Cmovle |
        Opcode::Cmovg | Opcode::Cmovge | Opcode::Cmovb | Opcode::Cmovbe |
        Opcode::Cmova | Opcode::Cmovae | Opcode::Cmovs | Opcode::Cmovns => {
            let cc_byte: u8 = match inst.opcode {
                Opcode::Cmove  => 0x44, Opcode::Cmovne => 0x45,
                Opcode::Cmovl  => 0x4C, Opcode::Cmovle => 0x4E,
                Opcode::Cmovg  => 0x4F, Opcode::Cmovge => 0x4D,
                Opcode::Cmovb  => 0x42, Opcode::Cmovbe => 0x46,
                Opcode::Cmova  => 0x47, Opcode::Cmovae => 0x43,
                Opcode::Cmovs  => 0x48, Opcode::Cmovns => 0x49,
                _ => unreachable!(),
            };
            if let (Some(Operand::Reg(dst)), Some(Operand::Reg(src))) = (inst.operands.get(0), inst.operands.get(1)) {
                let di = sib::encode_reg(dst);
                let si = sib::encode_reg(src);
                if let Some(rex) = sib::build_rex(di.is_wide, di.is_ext, false, si.is_ext) { bytes.push(rex); }
                bytes.push(0x0F);
                bytes.push(cc_byte);
                bytes.push(sib::modrm(3, di.val, si.val));
            }
        }
        
        // --- LOOP / LOOPE / LOOPNE (rel8) ---
        Opcode::Loop | Opcode::Loope | Opcode::Loopne => {
            let op_byte: u8 = match inst.opcode {
                Opcode::Loop => 0xE2, Opcode::Loope => 0xE1, Opcode::Loopne => 0xE0,
                _ => unreachable!(),
            };
            if let Some(Operand::Label(lbl)) = inst.operands.get(0) {
                if let Some(map) = labels {
                    if let Some(&target) = map.get(lbl) {
                        let delta = (target as i64) - ((current_offset + 2) as i64);
                        bytes.push(op_byte);
                        bytes.push(delta as i8 as u8);
                    }
                }
                if bytes.is_empty() {
                    bytes.push(op_byte);
                    bytes.push(0xFE); // -2 placeholder (jump to self)
                }
            }
        }
        
        // --- BT, BTS, BTR, BTC (0F BA /ext or 0F Ax /r) ---
        Opcode::Bt | Opcode::Bts | Opcode::Btr | Opcode::Btc => {
            if let (Some(Operand::Reg(dst)), Some(src)) = (inst.operands.get(0), inst.operands.get(1)) {
                let di = sib::encode_reg(dst);
                match src {
                    Operand::Imm(imm) => {
                        let ext = match inst.opcode {
                            Opcode::Bt => 4, Opcode::Bts => 5, Opcode::Btr => 6, Opcode::Btc => 7,
                            _ => 4,
                        };
                        if let Some(rex) = sib::build_rex(di.is_wide, false, false, di.is_ext) { bytes.push(rex); }
                        bytes.push(0x0F); bytes.push(0xBA);
                        bytes.push(sib::modrm(3, ext, di.val));
                        bytes.push(*imm as u8);
                    }
                    Operand::Reg(s) => {
                        let si = sib::encode_reg(s);
                        let op2 = match inst.opcode {
                            Opcode::Bt => 0xA3, Opcode::Bts => 0xAB, Opcode::Btr => 0xB3, Opcode::Btc => 0xBB,
                            _ => 0xA3,
                        };
                        if let Some(rex) = sib::build_rex(di.is_wide, si.is_ext, false, di.is_ext) { bytes.push(rex); }
                        bytes.push(0x0F); bytes.push(op2);
                        bytes.push(sib::modrm(3, si.val, di.val));
                    }
                    _ => {}
                }
            }
        }
        
        // --- BSF, BSR (0F BC/BD) ---
        Opcode::Bsf | Opcode::Bsr => {
            if let (Some(Operand::Reg(d)), Some(Operand::Reg(s))) = (inst.operands.get(0), inst.operands.get(1)) {
                let di = sib::encode_reg(d); let si = sib::encode_reg(s);
                if let Some(rex) = sib::build_rex(di.is_wide, di.is_ext, false, si.is_ext) { bytes.push(rex); }
                bytes.push(0x0F);
                bytes.push(if inst.opcode == Opcode::Bsf { 0xBC } else { 0xBD });
                bytes.push(sib::modrm(3, di.val, si.val));
            }
        }
        
        // --- POPCNT (F3 0F B8), LZCNT (F3 0F BD), TZCNT (F3 0F BC) ---
        Opcode::Popcnt | Opcode::Lzcnt | Opcode::Tzcnt => {
            if let (Some(Operand::Reg(d)), Some(Operand::Reg(s))) = (inst.operands.get(0), inst.operands.get(1)) {
                let di = sib::encode_reg(d); let si = sib::encode_reg(s);
                bytes.push(0xF3);
                if let Some(rex) = sib::build_rex(di.is_wide, di.is_ext, false, si.is_ext) { bytes.push(rex); }
                bytes.push(0x0F);
                bytes.push(match inst.opcode {
                    Opcode::Popcnt => 0xB8, Opcode::Lzcnt => 0xBD, Opcode::Tzcnt => 0xBC,
                    _ => 0xB8,
                });
                bytes.push(sib::modrm(3, di.val, si.val));
            }
        }
        
        // --- XADD (0F C0/C1), CMPXCHG (0F B0/B1) ---
        Opcode::Xadd | Opcode::Cmpxchg => {
            if let (Some(Operand::Reg(d)), Some(Operand::Reg(s))) = (inst.operands.get(0), inst.operands.get(1)) {
                let di = sib::encode_reg(d); let si = sib::encode_reg(s);
                if let Some(rex) = sib::build_rex(di.is_wide, si.is_ext, false, di.is_ext) { bytes.push(rex); }
                bytes.push(0x0F);
                let base = if inst.opcode == Opcode::Xadd { 0xC0u8 } else { 0xB0u8 };
                bytes.push(base + if di.is_8 { 0 } else { 1 });
                bytes.push(sib::modrm(3, si.val, di.val));
            }
        }

        // --- String Ops (zero-operand) ---
        Opcode::Movsb  => { bytes.push(0xA4); }
        Opcode::Movsw  => { bytes.push(0x66); bytes.push(0xA5); }
        Opcode::Movsd  => { bytes.push(0xA5); }
        Opcode::Movsq  => { bytes.push(0x48); bytes.push(0xA5); }
        Opcode::Stosb  => { bytes.push(0xAA); }
        Opcode::Stosw  => { bytes.push(0x66); bytes.push(0xAB); }
        Opcode::Stosd  => { bytes.push(0xAB); }
        Opcode::Stosq  => { bytes.push(0x48); bytes.push(0xAB); }
        Opcode::Lodsb  => { bytes.push(0xAC); }
        Opcode::Lodsw  => { bytes.push(0x66); bytes.push(0xAD); }
        Opcode::Lodsd  => { bytes.push(0xAD); }
        Opcode::Lodsq  => { bytes.push(0x48); bytes.push(0xAD); }
        Opcode::Scasb  => { bytes.push(0xAE); }
        Opcode::Scasw  => { bytes.push(0x66); bytes.push(0xAF); }
        Opcode::Scasd  => { bytes.push(0xAF); }
        Opcode::Cmpsb  => { bytes.push(0xA6); }
        Opcode::Cmpsw  => { bytes.push(0x66); bytes.push(0xA7); }
        Opcode::Cmpsd  => { bytes.push(0xA7); }

        // --- REP prefix string ops ---
        Opcode::RepMovsb => { bytes.push(0xF3); bytes.push(0xA4); }
        Opcode::RepMovsw => { bytes.push(0xF3); bytes.push(0x66); bytes.push(0xA5); }
        Opcode::RepMovsd => { bytes.push(0xF3); bytes.push(0xA5); }
        Opcode::RepMovsq => { bytes.push(0xF3); bytes.push(0x48); bytes.push(0xA5); }
        Opcode::RepStosb => { bytes.push(0xF3); bytes.push(0xAA); }
        Opcode::RepStosw => { bytes.push(0xF3); bytes.push(0x66); bytes.push(0xAB); }
        Opcode::RepStosd => { bytes.push(0xF3); bytes.push(0xAB); }
        Opcode::RepStosq => { bytes.push(0xF3); bytes.push(0x48); bytes.push(0xAB); }
        Opcode::RepeCmpsb => { bytes.push(0xF3); bytes.push(0xA6); }
        Opcode::RepeCmpsw => { bytes.push(0xF3); bytes.push(0x66); bytes.push(0xA7); }
        Opcode::RepeCmpsd => { bytes.push(0xF3); bytes.push(0xA7); }
        Opcode::RepneScasb => { bytes.push(0xF2); bytes.push(0xAE); }
        Opcode::RepneScasw => { bytes.push(0xF2); bytes.push(0x66); bytes.push(0xAF); }
        Opcode::RepneScasd => { bytes.push(0xF2); bytes.push(0xAF); }

        // === FASE 5: SSE Packed Float ===
        Opcode::Movaps | Opcode::Movups | Opcode::Addps | Opcode::Subps |
        Opcode::Mulps | Opcode::Divps | Opcode::Xorps | Opcode::Andps |
        Opcode::Orps | Opcode::Andnps | Opcode::Sqrtps | Opcode::Minps |
        Opcode::Maxps | Opcode::Rsqrtps | Opcode::Rcpps => {
            let op2 = match inst.opcode {
                Opcode::Movaps => 0x28, Opcode::Movups => 0x10,
                Opcode::Addps => 0x58, Opcode::Subps => 0x5C,
                Opcode::Mulps => 0x59, Opcode::Divps => 0x5E,
                Opcode::Xorps => 0x57, Opcode::Andps => 0x54,
                Opcode::Orps  => 0x56, Opcode::Andnps => 0x55,
                Opcode::Sqrtps => 0x51, Opcode::Minps => 0x5D,
                Opcode::Maxps => 0x5F, Opcode::Rsqrtps => 0x52, Opcode::Rcpps => 0x53,
                _ => 0,
            };
            if let (Some(Operand::Reg(d)), Some(Operand::Reg(s))) = (inst.operands.get(0), inst.operands.get(1)) {
                let di = sib::encode_reg(d); let si = sib::encode_reg(s);
                if let Some(rex) = sib::build_rex(false, di.is_ext, false, si.is_ext) { bytes.push(rex); }
                bytes.push(0x0F); bytes.push(op2);
                bytes.push(sib::modrm(3, di.val, si.val));
            } else if let (Some(Operand::Reg(d)), Some(Operand::Memory { base, index, scale, disp, .. })) = (inst.operands.get(0), inst.operands.get(1)) {
                let di = sib::encode_reg(d);
                let mem = sib::resolve_memory(di.val, base.as_ref(), index.as_ref(), *scale, *disp);
                if let Some(rex) = sib::build_rex(false, di.is_ext, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                bytes.push(0x0F); bytes.push(op2);
                bytes.extend(mem.payload);
            // FASE 7: SSE store — mem, xmm (MOVAPS→0x29, MOVUPS→0x11)
            } else if let (Some(Operand::Memory { base, index, scale, disp, .. }), Some(Operand::Reg(s))) = (inst.operands.get(0), inst.operands.get(1)) {
                let si = sib::encode_reg(s);
                let mem = sib::resolve_memory(si.val, base.as_ref(), index.as_ref(), *scale, *disp);
                if let Some(rex) = sib::build_rex(false, si.is_ext, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                bytes.push(0x0F);
                // Store opcodes: MOVAPS→0x29, MOVUPS→0x11, others are not stores
                let store_op = match inst.opcode {
                    Opcode::Movaps => 0x29, Opcode::Movups => 0x11,
                    _ => op2, // fallback to same opcode
                };
                bytes.push(store_op);
                bytes.extend(mem.payload);
            }
        }

        // === SSE2 Scalar Double (F2 prefix) ===
        Opcode::Addsd | Opcode::Subsd | Opcode::Mulsd | Opcode::Divsd |
        Opcode::Sqrtsd | Opcode::Minsd | Opcode::Maxsd |
        Opcode::Cvtsi2sd | Opcode::Cvtsd2si | Opcode::Cvttsd2si |
        Opcode::Movsd2 | Opcode::Comisd | Opcode::Ucomisd => {
            let op2 = match inst.opcode {
                Opcode::Addsd => 0x58, Opcode::Subsd => 0x5C,
                Opcode::Mulsd => 0x59, Opcode::Divsd => 0x5E,
                Opcode::Sqrtsd => 0x51, Opcode::Minsd => 0x5D, Opcode::Maxsd => 0x5F,
                Opcode::Cvtsi2sd => 0x2A, Opcode::Cvtsd2si => 0x2D, Opcode::Cvttsd2si => 0x2C,
                Opcode::Movsd2 => 0x10, Opcode::Comisd => 0x2F, Opcode::Ucomisd => 0x2E,
                _ => 0,
            };
            // COMISD/UCOMISD use 66 prefix, all others use F2
            let is_comi = inst.opcode == Opcode::Comisd || inst.opcode == Opcode::Ucomisd;
            if !is_comi { bytes.push(0xF2); } else { bytes.push(0x66); }
            
            if let (Some(Operand::Reg(d)), Some(Operand::Reg(s))) = (inst.operands.get(0), inst.operands.get(1)) {
                let di = sib::encode_reg(d); let si = sib::encode_reg(s);
                let w = inst.opcode == Opcode::Cvtsi2sd && si.is_wide;
                if let Some(rex) = sib::build_rex(w, di.is_ext, false, si.is_ext) { bytes.push(rex); }
                bytes.push(0x0F); bytes.push(op2);
                bytes.push(sib::modrm(3, di.val, si.val));
            } else if let (Some(Operand::Reg(d)), Some(Operand::Memory { base, index, scale, disp, .. })) = (inst.operands.get(0), inst.operands.get(1)) {
                let di = sib::encode_reg(d);
                let mem = sib::resolve_memory(di.val, base.as_ref(), index.as_ref(), *scale, *disp);
                if let Some(rex) = sib::build_rex(false, di.is_ext, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                bytes.push(0x0F); bytes.push(op2);
                bytes.extend(mem.payload);
            } else if let (Some(Operand::Reg(d)), Some(Operand::Label(lbl))) = (inst.operands.get(0), inst.operands.get(1)) {
                let di = sib::encode_reg(d);
                if let Some(rex) = sib::build_rex(false, di.is_ext, false, false) { bytes.push(rex); }
                bytes.push(0x0F); bytes.push(op2);
                bytes.push(sib::modrm(0, di.val, 5));
                bytes.extend_from_slice(&[0,0,0,0]);
                relocations.push(RelocationReq { offset: bytes.len() as u32 - 4, symbol: lbl.clone(), rel_type: 4 });
            // FASE 7: MOVSD mem, xmm — F2 0F 11 (store direction)
            } else if let (Some(Operand::Memory { base, index, scale, disp, .. }), Some(Operand::Reg(s))) = (inst.operands.get(0), inst.operands.get(1)) {
                if inst.opcode == Opcode::Movsd2 {
                    let si = sib::encode_reg(s);
                    let mem = sib::resolve_memory(si.val, base.as_ref(), index.as_ref(), *scale, *disp);
                    bytes.push(0xF2);
                    if let Some(rex) = sib::build_rex(false, si.is_ext, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                    bytes.push(0x0F); bytes.push(0x11);
                    bytes.extend(mem.payload);
                }
            }
        }

        // === SSE2 Packed Double (66 prefix) ===
        Opcode::Movapd | Opcode::Movupd | Opcode::Addpd | Opcode::Subpd |
        Opcode::Mulpd | Opcode::Divpd | Opcode::Xorpd | Opcode::Andpd |
        Opcode::Orpd | Opcode::Andnpd | Opcode::Sqrtpd | Opcode::Minpd |
        Opcode::Maxpd => {
            let op2 = match inst.opcode {
                Opcode::Movapd => 0x28, Opcode::Movupd => 0x10,
                Opcode::Addpd => 0x58, Opcode::Subpd => 0x5C,
                Opcode::Mulpd => 0x59, Opcode::Divpd => 0x5E,
                Opcode::Xorpd => 0x57, Opcode::Andpd => 0x54,
                Opcode::Orpd  => 0x56, Opcode::Andnpd => 0x55,
                Opcode::Sqrtpd => 0x51, Opcode::Minpd => 0x5D, Opcode::Maxpd => 0x5F,
                _ => 0,
            };
            bytes.push(0x66);
            if let (Some(Operand::Reg(d)), Some(Operand::Reg(s))) = (inst.operands.get(0), inst.operands.get(1)) {
                let di = sib::encode_reg(d); let si = sib::encode_reg(s);
                if let Some(rex) = sib::build_rex(false, di.is_ext, false, si.is_ext) { bytes.push(rex); }
                bytes.push(0x0F); bytes.push(op2);
                bytes.push(sib::modrm(3, di.val, si.val));
            } else if let (Some(Operand::Reg(d)), Some(Operand::Memory { base, index, scale, disp, .. })) = (inst.operands.get(0), inst.operands.get(1)) {
                let di = sib::encode_reg(d);
                let mem = sib::resolve_memory(di.val, base.as_ref(), index.as_ref(), *scale, *disp);
                if let Some(rex) = sib::build_rex(false, di.is_ext, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                bytes.push(0x0F); bytes.push(op2);
                bytes.extend(mem.payload);
            // FASE 7: SSE2 packed double store — mem, xmm (MOVAPD→0x29, MOVUPD→0x11)
            } else if let (Some(Operand::Memory { base, index, scale, disp, .. }), Some(Operand::Reg(s))) = (inst.operands.get(0), inst.operands.get(1)) {
                let si = sib::encode_reg(s);
                let mem = sib::resolve_memory(si.val, base.as_ref(), index.as_ref(), *scale, *disp);
                bytes.push(0x66);
                if let Some(rex) = sib::build_rex(false, si.is_ext, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                bytes.push(0x0F);
                let store_op = match inst.opcode {
                    Opcode::Movapd => 0x29, Opcode::Movupd => 0x11,
                    _ => op2,
                };
                bytes.push(store_op);
                bytes.extend(mem.payload);
            }
        }

        // === SSE2 Integer (66 prefix) ===
        Opcode::Movdqa | Opcode::Movdqu |
        Opcode::Paddb | Opcode::Paddw | Opcode::Paddd | Opcode::Paddq |
        Opcode::Psubb | Opcode::Psubw | Opcode::Psubd | Opcode::Psubq |
        Opcode::Pmullw | Opcode::Pmulld |
        Opcode::Pand | Opcode::Por | Opcode::Pxor | Opcode::Pandn |
        Opcode::Pcmpeqb | Opcode::Pcmpeqw | Opcode::Pcmpeqd |
        Opcode::Pcmpgtb | Opcode::Pcmpgtw | Opcode::Pcmpgtd => {
            let op2 = match inst.opcode {
                Opcode::Movdqa => 0x6F, Opcode::Movdqu => 0x6F,
                Opcode::Paddb => 0xFC, Opcode::Paddw => 0xFD, Opcode::Paddd => 0xFE, Opcode::Paddq => 0xD4,
                Opcode::Psubb => 0xF8, Opcode::Psubw => 0xF9, Opcode::Psubd => 0xFA, Opcode::Psubq => 0xFB,
                Opcode::Pmullw => 0xD5, Opcode::Pmulld => 0x40, // pmulld needs 0F 38 prefix
                Opcode::Pand => 0xDB, Opcode::Por => 0xEB, Opcode::Pxor => 0xEF, Opcode::Pandn => 0xDF,
                Opcode::Pcmpeqb => 0x74, Opcode::Pcmpeqw => 0x75, Opcode::Pcmpeqd => 0x76,
                Opcode::Pcmpgtb => 0x64, Opcode::Pcmpgtw => 0x65, Opcode::Pcmpgtd => 0x66,
                _ => 0,
            };
            // MOVDQU uses F3 prefix instead of 66
            if inst.opcode == Opcode::Movdqu { bytes.push(0xF3); } else { bytes.push(0x66); }
            
            if let (Some(Operand::Reg(d)), Some(Operand::Reg(s))) = (inst.operands.get(0), inst.operands.get(1)) {
                let di = sib::encode_reg(d); let si = sib::encode_reg(s);
                if let Some(rex) = sib::build_rex(false, di.is_ext, false, si.is_ext) { bytes.push(rex); }
                if inst.opcode == Opcode::Pmulld { bytes.push(0x0F); bytes.push(0x38); } else { bytes.push(0x0F); }
                bytes.push(op2);
                bytes.push(sib::modrm(3, di.val, si.val));
            } else if let (Some(Operand::Reg(d)), Some(Operand::Memory { base, index, scale, disp, .. })) = (inst.operands.get(0), inst.operands.get(1)) {
                let di = sib::encode_reg(d);
                let mem = sib::resolve_memory(di.val, base.as_ref(), index.as_ref(), *scale, *disp);
                if let Some(rex) = sib::build_rex(false, di.is_ext, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                if inst.opcode == Opcode::Pmulld { bytes.push(0x0F); bytes.push(0x38); } else { bytes.push(0x0F); }
                bytes.push(op2);
                bytes.extend(mem.payload);
            // FASE 7: SSE2 integer store — mem, xmm (MOVDQA→66 0F 7F, MOVDQU→F3 0F 7F)
            } else if let (Some(Operand::Memory { base, index, scale, disp, .. }), Some(Operand::Reg(s))) = (inst.operands.get(0), inst.operands.get(1)) {
                if inst.opcode == Opcode::Movdqa || inst.opcode == Opcode::Movdqu {
                    let si = sib::encode_reg(s);
                    let mem = sib::resolve_memory(si.val, base.as_ref(), index.as_ref(), *scale, *disp);
                    if inst.opcode == Opcode::Movdqu { bytes.push(0xF3); } else { bytes.push(0x66); }
                    if let Some(rex) = sib::build_rex(false, si.is_ext, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                    bytes.push(0x0F); bytes.push(0x7F); // store opcode
                    bytes.extend(mem.payload);
                }
            }
        }

        // --- MOVD (66 0F 6E/7E), MOVQ (66 REX.W 0F 6E/7E) ---
        Opcode::Movd | Opcode::Movq => {
            let is_q = inst.opcode == Opcode::Movq;
            if let (Some(Operand::Reg(d)), Some(Operand::Reg(s))) = (inst.operands.get(0), inst.operands.get(1)) {
                let di = sib::encode_reg(d);
                let si = sib::encode_reg(s);
                bytes.push(0x66);
                if let Some(rex) = sib::build_rex(is_q, di.is_ext, false, si.is_ext) { bytes.push(rex); }
                bytes.push(0x0F);
                bytes.push(0x6E); // xmm <- gpr direction
                bytes.push(sib::modrm(3, di.val, si.val));
            }
        }
        
        // --- SSE conversions between float/double ---
        Opcode::Cvtss2sd => {
            bytes.push(0xF3);
            if let (Some(Operand::Reg(d)), Some(Operand::Reg(s))) = (inst.operands.get(0), inst.operands.get(1)) {
                let di = sib::encode_reg(d); let si = sib::encode_reg(s);
                if let Some(rex) = sib::build_rex(false, di.is_ext, false, si.is_ext) { bytes.push(rex); }
                bytes.push(0x0F); bytes.push(0x5A);
                bytes.push(sib::modrm(3, di.val, si.val));
            }
        }
        Opcode::Cvtsd2ss => {
            bytes.push(0xF2);
            if let (Some(Operand::Reg(d)), Some(Operand::Reg(s))) = (inst.operands.get(0), inst.operands.get(1)) {
                let di = sib::encode_reg(d); let si = sib::encode_reg(s);
                if let Some(rex) = sib::build_rex(false, di.is_ext, false, si.is_ext) { bytes.push(rex); }
                bytes.push(0x0F); bytes.push(0x5A);
                bytes.push(sib::modrm(3, di.val, si.val));
            }
        }
        // --- COMISS/UCOMISS (NP 0F 2F/2E) ---
        Opcode::Comiss | Opcode::Ucomiss => {
            if let (Some(Operand::Reg(d)), Some(Operand::Reg(s))) = (inst.operands.get(0), inst.operands.get(1)) {
                let di = sib::encode_reg(d); let si = sib::encode_reg(s);
                if let Some(rex) = sib::build_rex(false, di.is_ext, false, si.is_ext) { bytes.push(rex); }
                bytes.push(0x0F);
                bytes.push(if inst.opcode == Opcode::Comiss { 0x2F } else { 0x2E });
                bytes.push(sib::modrm(3, di.val, si.val));
            }
        }

        // --- SSE Scalar: SUBSS, MULSS, DIVSS, MINSS, MAXSS ---
        Opcode::Subss | Opcode::Mulss | Opcode::Divss | Opcode::Minss | Opcode::Maxss => {
            let op2 = match inst.opcode {
                Opcode::Subss => 0x5C, Opcode::Mulss => 0x59,
                Opcode::Divss => 0x5E, Opcode::Minss => 0x5D, Opcode::Maxss => 0x5F,
                _ => 0,
            };
            bytes.push(0xF3);
            if let (Some(Operand::Reg(d)), Some(Operand::Reg(s))) = (inst.operands.get(0), inst.operands.get(1)) {
                let di = sib::encode_reg(d); let si = sib::encode_reg(s);
                if let Some(rex) = sib::build_rex(false, di.is_ext, false, si.is_ext) { bytes.push(rex); }
                bytes.push(0x0F); bytes.push(op2);
                bytes.push(sib::modrm(3, di.val, si.val));
            } else if let (Some(Operand::Reg(d)), Some(Operand::Memory { base, index, scale, disp, .. })) = (inst.operands.get(0), inst.operands.get(1)) {
                let di = sib::encode_reg(d);
                let mem = sib::resolve_memory(di.val, base.as_ref(), index.as_ref(), *scale, *disp);
                if let Some(rex) = sib::build_rex(false, di.is_ext, mem.rex_x, mem.rex_b) { bytes.push(rex); }
                bytes.push(0x0F); bytes.push(op2);
                bytes.extend(mem.payload);
            }
        }

        _ => return Err(format!("Unimplemented binary encoding for {:?}", inst.opcode)),
    }

    Ok(EncodedInstruction { bytes, relocations })
}

