use crate::ir::{Arch, Instruction};
use crate::targets::ArchEncoder;

pub struct MipsEncoder;

impl ArchEncoder for MipsEncoder {
    fn validate(&self, _inst: &Instruction) -> Result<(), String> {
        Ok(())
    }
    fn arch(&self) -> Arch { Arch::Mips }
}
