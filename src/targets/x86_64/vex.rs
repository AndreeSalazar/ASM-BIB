use crate::ir::Register;
use super::sib::{encode_reg, RegInfo};

/// Builds the VEX Payload (2-byte or 3-byte prefix) for AVX commands.
/// `w_bit`: REX.W equivalent
/// `r_bit`, `x_bit`, `b_bit`: Extensions for Reg, Index, Base
/// `m_map`: usually 1 (0F), 2 (0F 38), or 3 (0F 3A)
/// `vvvv`: VEX inversed register bits for 3-operand instructions
/// `l_bit`: 0 for 128-bit (xmm), 1 for 256-bit (ymm)
/// `pp`: 00 (None), 01 (66), 10 (F3), 11 (F2)
pub fn build_vex(w_bit: bool, r_bit: bool, x_bit: bool, b_bit: bool, m_map: u8, v_reg: Option<&Register>, l_bit: bool, pp: u8) -> Vec<u8> {
    let mut payload = Vec::new();
    
    let vvvv = if let Some(v) = v_reg {
        let ri = encode_reg(v);
        // Inverted representation
        (!ri.val) & 0x0F
    } else {
        0b1111 // Unused
    };
    
    // Check if we can use 2-byte VEX (C5)
    // C5 requires m_map == 1, x_bit == false, b_bit == false, w_bit == false
    if m_map == 1 && !x_bit && !b_bit && !w_bit {
        payload.push(0xC5);
        
        let mut byte1 = 0;
        if !r_bit { byte1 |= 0x80; } // Inverted R
        byte1 |= (vvvv << 3);
        if l_bit { byte1 |= 0x04; }
        byte1 |= (pp & 0x03);
        
        payload.push(byte1);
    } else {
        // 3-byte VEX (C4)
        payload.push(0xC4);
        
        let mut byte1 = 0;
        if !r_bit { byte1 |= 0x80; }
        if !x_bit { byte1 |= 0x40; }
        if !b_bit { byte1 |= 0x20; }
        byte1 |= (m_map & 0x1F);
        payload.push(byte1);
        
        let mut byte2 = 0;
        if w_bit { byte2 |= 0x80; }
        byte2 |= (vvvv << 3);
        if l_bit { byte2 |= 0x04; }
        byte2 |= (pp & 0x03);
        payload.push(byte2);
    }
    
    payload
}
