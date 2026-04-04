//! ASM-BIB Internal Linker — Phases 10-13
//! Replaces link.exe for x86-64 Windows PE executables and DLLs.
//!
//! Pipeline:
//!   .obj (COFF) → parse → resolve symbols → layout → relocate → PE write

pub mod coff_reader;
pub mod import_lib;
pub mod pe_writer;
pub mod relocator;

pub use pe_writer::link_program;
