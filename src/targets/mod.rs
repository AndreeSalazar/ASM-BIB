pub mod x86_64;
pub mod arm64;
pub mod riscv;
pub mod mips;

use crate::ir::{Arch, Instruction};

/// Trait that all architecture encoders must implement
pub trait ArchEncoder {
    /// Validate that an instruction is valid for this architecture
    fn validate(&self, inst: &Instruction) -> Result<(), String>;

    /// Get the architecture this encoder handles
    fn arch(&self) -> Arch;
}

/// Get the appropriate encoder for an architecture
pub fn get_encoder(arch: Arch) -> Box<dyn ArchEncoder> {
    match arch {
        Arch::X86_64 | Arch::X86_16 | Arch::X86_32 => Box::new(x86_64::X86_64Encoder),
        Arch::Arm64 => Box::new(arm64::Arm64Encoder),
        Arch::RiscV64 => Box::new(riscv::RiscVEncoder),
        Arch::Mips => Box::new(mips::MipsEncoder),
    }
}
