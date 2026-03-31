use crate::ir::{Arch, Instruction};
use crate::targets::ArchEncoder;

pub struct RiscVEncoder;

impl ArchEncoder for RiscVEncoder {
    fn validate(&self, _inst: &Instruction) -> Result<(), String> {
        Ok(())
    }
    fn arch(&self) -> Arch { Arch::RiscV64 }
}
