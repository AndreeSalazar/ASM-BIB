pub mod registers;
pub mod instructions;
pub mod encoder;
pub mod sib;
pub mod vex;

use crate::ir::{Arch, Instruction};
use crate::targets::ArchEncoder;

pub struct X86_64Encoder;

use std::collections::HashMap;

impl ArchEncoder for X86_64Encoder {
    fn validate(&self, _inst: &Instruction) -> Result<(), String> {
        // Base validation - accept all for now
        Ok(())
    }
    
    fn encode(&self, inst: &Instruction, labels: Option<&HashMap<String, u32>>, current_offset: u32) -> Result<crate::targets::x86_64::encoder::EncodedInstruction, String> {
        encoder::encode_instruction(inst, labels, current_offset)
    }

    fn arch(&self) -> Arch { Arch::X86_64 }
}
