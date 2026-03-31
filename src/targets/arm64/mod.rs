use crate::ir::{Arch, Instruction};
use crate::targets::ArchEncoder;

pub struct Arm64Encoder;

impl ArchEncoder for Arm64Encoder {
    fn validate(&self, _inst: &Instruction) -> Result<(), String> {
        Ok(())
    }
    fn arch(&self) -> Arch { Arch::Arm64 }
}
