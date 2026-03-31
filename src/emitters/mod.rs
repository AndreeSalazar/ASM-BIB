pub mod nasm;
pub mod gas;
pub mod masm;
pub mod fasm;
pub mod flat;

use crate::ir::Program;

/// Output format selection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Nasm,
    Gas,
    Masm,
    Fasm,
    Flat,
}

/// Trait all emitters must implement
pub trait Emitter {
    fn emit(&self, program: &Program) -> String;
    fn format(&self) -> OutputFormat;
}

/// Get the appropriate emitter for a format
pub fn get_emitter(format: OutputFormat) -> Box<dyn Emitter> {
    match format {
        OutputFormat::Nasm => Box::new(nasm::NasmEmitter),
        OutputFormat::Gas => Box::new(gas::GasEmitter),
        OutputFormat::Masm => Box::new(masm::MasmEmitter),
        OutputFormat::Fasm => Box::new(fasm::FasmEmitter),
        OutputFormat::Flat => Box::new(flat::FlatEmitter),
    }
}
