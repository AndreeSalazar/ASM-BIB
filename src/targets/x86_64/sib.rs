use crate::ir::{Register, Operand};

/// Core details about an encoded register
pub struct RegInfo {
    pub val: u8,
    pub is_ext: bool, // Requires REX.R/X/B
    pub is_wide: bool, // 64-bit size (Requires REX.W)
    pub is_32: bool,   // 32-bit size
    pub is_16: bool,   // 16-bit size (Requires 0x66 prefix)
    pub is_8: bool,    // 8-bit size
}

pub fn encode_reg(reg: &Register) -> RegInfo {
    let mut ri = RegInfo { val: 0, is_ext: false, is_wide: true, is_32: false, is_16: false, is_8: false };
    
    ri.val = match reg {
        // 64-bit Regs
        Register::Rax => 0, Register::Rcx => 1, Register::Rdx => 2, Register::Rbx => 3,
        Register::Rsp => 4, Register::Rbp => 5, Register::Rsi => 6, Register::Rdi => 7,
        Register::R8 => { ri.is_ext=true; 0 }, Register::R9 => { ri.is_ext=true; 1 },
        Register::R10 => { ri.is_ext=true; 2 }, Register::R11 => { ri.is_ext=true; 3 },
        Register::R12 => { ri.is_ext=true; 4 }, Register::R13 => { ri.is_ext=true; 5 },
        Register::R14 => { ri.is_ext=true; 6 }, Register::R15 => { ri.is_ext=true; 7 },
        
        // 32-bit Regs
        Register::Eax => { ri.is_wide=false; ri.is_32=true; 0 }, Register::Ecx => { ri.is_wide=false; ri.is_32=true; 1 },
        Register::Edx => { ri.is_wide=false; ri.is_32=true; 2 }, Register::Ebx => { ri.is_wide=false; ri.is_32=true; 3 },
        Register::Esp => { ri.is_wide=false; ri.is_32=true; 4 }, Register::Ebp => { ri.is_wide=false; ri.is_32=true; 5 },
        Register::Esi => { ri.is_wide=false; ri.is_32=true; 6 }, Register::Edi => { ri.is_wide=false; ri.is_32=true; 7 },
        Register::R8d => { ri.is_wide=false; ri.is_32=true; ri.is_ext=true; 0 },
        Register::R9d => { ri.is_wide=false; ri.is_32=true; ri.is_ext=true; 1 },
        Register::R10d => { ri.is_wide=false; ri.is_32=true; ri.is_ext=true; 2 },
        Register::R11d => { ri.is_wide=false; ri.is_32=true; ri.is_ext=true; 3 },
        Register::R12d => { ri.is_wide=false; ri.is_32=true; ri.is_ext=true; 4 },
        Register::R13d => { ri.is_wide=false; ri.is_32=true; ri.is_ext=true; 5 },
        Register::R14d => { ri.is_wide=false; ri.is_32=true; ri.is_ext=true; 6 },
        Register::R15d => { ri.is_wide=false; ri.is_32=true; ri.is_ext=true; 7 },
        
        // 16-bit Regs
        Register::Ax => { ri.is_wide=false; ri.is_16=true; 0 }, Register::Cx => { ri.is_wide=false; ri.is_16=true; 1 },
        Register::Dx => { ri.is_wide=false; ri.is_16=true; 2 }, Register::Bx => { ri.is_wide=false; ri.is_16=true; 3 },
        Register::Sp => { ri.is_wide=false; ri.is_16=true; 4 }, Register::Bp => { ri.is_wide=false; ri.is_16=true; 5 },
        Register::Si => { ri.is_wide=false; ri.is_16=true; 6 }, Register::Di => { ri.is_wide=false; ri.is_16=true; 7 },
        
        // 8-bit Regs
        Register::Al => { ri.is_wide=false; ri.is_8=true; 0 }, Register::Cl => { ri.is_wide=false; ri.is_8=true; 1 },
        Register::Dl => { ri.is_wide=false; ri.is_8=true; 2 }, Register::Bl => { ri.is_wide=false; ri.is_8=true; 3 },
        
        // SSE/AVX
        Register::Xmm(n) => { ri.is_wide=false; ri.is_ext = *n > 7; *n & 7 },
        Register::Ymm(n) => { ri.is_wide=false; ri.is_ext = *n > 7; *n & 7 },
        
        _ => { ri.is_wide = false; 0 },
    };
    ri
}

pub fn modrm(mod_val: u8, reg: u8, rm: u8) -> u8 {
    ((mod_val & 3) << 6) | ((reg & 7) << 3) | (rm & 7)
}

pub fn build_rex(w: bool, r: bool, x: bool, b: bool) -> Option<u8> {
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

/// Represents a fully resolved memory payload containing SIB and Displacements
pub struct MemPayload {
    pub rex_b: bool,
    pub rex_x: bool,
    pub payload: Vec<u8>,
}

/// Resolves a Memory Operand into the required ModR/M, SIB, and Displacement bytes
pub fn resolve_memory(modrm_reg: u8, base: Option<&Register>, index: Option<&Register>, scale: u8, disp: i64) -> MemPayload {
    let mut payload = Vec::new();
    let mut rex_b = false;
    let mut rex_x = false;
    
    // Scale encoding mapping
    let scale_encoded = match scale {
        1 => 0,
        2 => 1,
        4 => 2,
        8 => 3,
        _ => 0,
    };

    if let Some(b) = base {
        let b_info = encode_reg(b);
        rex_b = b_info.is_ext;
        
        if let Some(i) = index {
            let i_info = encode_reg(i);
            rex_x = i_info.is_ext;
            
            // Requires SIB (rm=4)
            let mod_b = if disp == 0 && b_info.val != 5 { 0 } else if disp >= -128 && disp <= 127 { 1 } else { 2 };
            payload.push(modrm(mod_b, modrm_reg, 4));
            
            // SIB byte: scale | index | base
            let sib = (scale_encoded << 6) | ((i_info.val & 7) << 3) | (b_info.val & 7);
            payload.push(sib);
            
            if mod_b == 1 {
                payload.push(disp as i8 as u8);
            } else if mod_b == 2 || (mod_b == 0 && b_info.val == 5) {
                payload.extend_from_slice(&(disp as i32).to_le_bytes());
            }
        } else {
            // No index, just base
            let mod_b = if disp == 0 && b_info.val != 5 { 0 } else if disp >= -128 && disp <= 127 { 1 } else { 2 };
            
            if b_info.val == 4 {
                // Base is RSP/R12, MUST use SIB regardless of index
                payload.push(modrm(mod_b, modrm_reg, 4));
                // SIB: scale=0, index=4 (none), base=4
                payload.push(modrm(0, 4, 4));
            } else {
                payload.push(modrm(mod_b, modrm_reg, b_info.val));
            }
            
            if mod_b == 1 {
                payload.push(disp as i8 as u8);
            } else if mod_b == 2 || (mod_b == 0 && b_info.val == 5) {
                payload.extend_from_slice(&(disp as i32).to_le_bytes());
            }
        }
    } else {
        // No Base. Absolute memory (disp32)
        // Mod=0, R/M=4 (SIB) -> Base=5, Index=4
        if let Some(i) = index {
            let i_info = encode_reg(i);
            rex_x = i_info.is_ext;
            payload.push(modrm(0, modrm_reg, 4)); // SIB mandatory
            let sib = (scale_encoded << 6) | ((i_info.val & 7) << 3) | 5; // Base=5 means disp32
            payload.push(sib);
            payload.extend_from_slice(&(disp as i32).to_le_bytes());
        } else {
            // pure displacement: mod=0, rm=5 (RIP-relative or absolute depending on 64-bit OS mapping)
            payload.push(modrm(0, modrm_reg, 5));
            payload.extend_from_slice(&(disp as i32).to_le_bytes());
        }
    }
    
    MemPayload { rex_b, rex_x, payload }
}
