pub mod registers;
pub mod instructions;
pub mod encoder;

use crate::ir::{Arch, Instruction};
use crate::targets::ArchEncoder;

pub struct X86_64Encoder;

impl ArchEncoder for X86_64Encoder {
    fn validate(&self, _inst: &Instruction) -> Result<(), String> {
        // Base validation - accept all for now
        Ok(())
    }
    
    fn encode(&self, inst: &Instruction) -> Result<Vec<u8>, String> {
        encoder::encode_instruction(inst)
    }

    fn arch(&self) -> Arch { Arch::X86_64 }
}
