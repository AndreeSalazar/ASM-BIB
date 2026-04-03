pub mod x86_64;
pub mod coff;

use crate::ir::{Arch, Instruction};

pub trait ArchEncoder {
    fn validate(&self, inst: &Instruction) -> Result<(), String>;
    fn encode(&self, inst: &Instruction) -> Result<Vec<u8>, String>;
    fn arch(&self) -> Arch;
}

pub fn get_encoder(arch: Arch) -> Box<dyn ArchEncoder> {
    match arch {
        Arch::X86_64 | Arch::X86_16 | Arch::X86_32 => Box::new(x86_64::X86_64Encoder),
    }
}
