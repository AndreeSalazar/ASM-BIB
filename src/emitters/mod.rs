pub mod nasm;
pub mod masm;

use crate::ir::Program;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Nasm,
    Masm,
}

pub trait Emitter {
    fn emit(&self, program: &Program) -> String;
    fn format(&self) -> OutputFormat;
}

pub fn get_emitter(format: OutputFormat) -> Box<dyn Emitter> {
    match format {
        OutputFormat::Nasm => Box::new(nasm::NasmEmitter),
        OutputFormat::Masm => Box::new(masm::MasmEmitter),
    }
}
