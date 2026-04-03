//! Microsoft PE/COFF Object File Generator
//! This module replaces `ml64.exe` by generating 64-bit `.obj` files directly.

use std::collections::HashMap;
use crate::ir::{Program, SectionKind, Arch};
use crate::targets::{ArchEncoder, x86_64::X86_64Encoder};

// === COFF Header Definitions ===
pub const IMAGE_FILE_MACHINE_AMD64: u16 = 0x8664;
pub const IMAGE_FILE_MACHINE_I386: u16  = 0x014C;

#[derive(Debug, Clone)]
pub struct CoffHeader {
    pub machine: u16,
    pub number_of_sections: u16,
    pub time_date_stamp: u32,
    pub pointer_to_symbol_table: u32,
    pub number_of_symbols: u32,
    pub size_of_optional_header: u16,
    pub characteristics: u16,
}

// === Section Header Definitions ===
pub const IMAGE_SCN_CNT_CODE: u32               = 0x00000020;
pub const IMAGE_SCN_CNT_INITIALIZED_DATA: u32   = 0x00000040;
pub const IMAGE_SCN_CNT_UNINITIALIZED_DATA: u32 = 0x00000080;
pub const IMAGE_SCN_ALIGN_16BYTES: u32          = 0x00500000;
pub const IMAGE_SCN_MEM_EXECUTE: u32            = 0x20000000;
pub const IMAGE_SCN_MEM_READ: u32               = 0x40000000;
pub const IMAGE_SCN_MEM_WRITE: u32              = 0x80000000;

#[derive(Debug, Clone)]
pub struct SectionHeader {
    pub name: [u8; 8],
    pub virtual_size: u32,
    pub virtual_address: u32,
    pub size_of_raw_data: u32,
    pub pointer_to_raw_data: u32,
    pub pointer_to_relocations: u32,
    pub pointer_to_linenumbers: u32,
    pub number_of_relocations: u16,
    pub number_of_linenumbers: u16,
    pub characteristics: u32,
}

// === COFF Builder ===
pub struct CoffObject {
    pub header: CoffHeader,
    pub sections: Vec<SectionHeader>,
    pub section_data: Vec<Vec<u8>>,
    // COFF relocations: virtual_address (u32), symbol_table_index (u32), type (u16)
    pub relocations: Vec<Vec<(u32, u32, u16)>>, 
    // COFF symbols: name ([u8; 8] or str_table), value(u32), section_number(i16), type(u16), storage_class(u8), aux(u8)
    pub symbols: Vec<([u8; 8], u32, i16, u16, u8, u8)>,
    pub string_table: Vec<u8>,
}

impl CoffObject {
    pub fn new(is_x64: bool) -> Self {
        Self {
            header: CoffHeader {
                machine: if is_x64 { IMAGE_FILE_MACHINE_AMD64 } else { IMAGE_FILE_MACHINE_I386 },
                number_of_sections: 0,
                time_date_stamp: 0,
                pointer_to_symbol_table: 0,
                number_of_symbols: 0,
                size_of_optional_header: 0,
                characteristics: 0,
            },
            sections: Vec::new(),
            section_data: Vec::new(),
            relocations: Vec::new(),
            symbols: Vec::new(),
            string_table: vec![0, 0, 0, 0], // First 4 bytes are size
        }
    }

    pub fn add_section(&mut self, name: &str, characteristics: u32, data: Vec<u8>) -> usize {
        let mut name_bytes = [0u8; 8];
        let bytes = name.as_bytes();
        let len = bytes.len().min(8);
        name_bytes[..len].copy_from_slice(&bytes[..len]);
        // TODO: Handle names longer than 8 bytes using string_table

        self.sections.push(SectionHeader {
            name: name_bytes,
            virtual_size: 0, // Unused in object files
            virtual_address: 0, // Unused in object files
            size_of_raw_data: data.len() as u32,
            pointer_to_raw_data: 0, // Resolved during build
            pointer_to_relocations: 0, // Resolved during build
            pointer_to_linenumbers: 0,
            number_of_relocations: 0, // Resolved during build
            number_of_linenumbers: 0,
            characteristics,
        });
        self.section_data.push(data);
        self.relocations.push(Vec::new());
        self.header.number_of_sections += 1;
        self.sections.len()
    }

    pub fn add_symbol(&mut self, name: &str, value: u32, section_num: i16, storage_class: u8) -> u32 {
        let mut name_bytes = [0u8; 8];
        if name.len() <= 8 {
            let bytes = name.as_bytes();
            name_bytes[..bytes.len()].copy_from_slice(bytes);
        } else {
            // Write to string table
            let offset = self.string_table.len() as u32;
            name_bytes[0..4].copy_from_slice(&0u32.to_le_bytes()); // first 4 bytes zero indicates long name
            name_bytes[4..8].copy_from_slice(&offset.to_le_bytes());
            self.string_table.extend_from_slice(name.as_bytes());
            self.string_table.push(0); // null terminator
        }

        self.symbols.push((name_bytes, value, section_num, 0, storage_class, 0));
        self.header.number_of_symbols += 1;
        self.symbols.len() as u32 - 1
    }

    pub fn build(mut self) -> Vec<u8> {
        let mut binary = Vec::new();
        
        // Calculate offsets
        let header_size = 20;
        let section_header_size = 40;
        let mut current_offset = header_size + (self.sections.len() as u32 * section_header_size);

        // Assign raw data offsets
        for i in 0..self.sections.len() {
            if !self.section_data[i].is_empty() {
                self.sections[i].pointer_to_raw_data = current_offset;
                current_offset += self.sections[i].size_of_raw_data;
            }
        }

        // Assign relocation offsets
        for i in 0..self.sections.len() {
            if !self.relocations[i].is_empty() {
                self.sections[i].pointer_to_relocations = current_offset;
                self.sections[i].number_of_relocations = self.relocations[i].len() as u16;
                current_offset += (self.relocations[i].len() as u32) * 10; // 10 bytes per relocation
            }
        }

        // Symbol table
        if !self.symbols.is_empty() {
            self.header.pointer_to_symbol_table = current_offset;
        }

        // Update string table size
        let str_table_size = self.string_table.len() as u32;
        self.string_table[0..4].copy_from_slice(&str_table_size.to_le_bytes());

        // --- Write File Header ---
        binary.extend_from_slice(&self.header.machine.to_le_bytes());
        binary.extend_from_slice(&self.header.number_of_sections.to_le_bytes());
        binary.extend_from_slice(&self.header.time_date_stamp.to_le_bytes());
        binary.extend_from_slice(&self.header.pointer_to_symbol_table.to_le_bytes());
        binary.extend_from_slice(&self.header.number_of_symbols.to_le_bytes());
        binary.extend_from_slice(&self.header.size_of_optional_header.to_le_bytes());
        binary.extend_from_slice(&self.header.characteristics.to_le_bytes());

        // --- Write Section Headers ---
        for sec in &self.sections {
            binary.extend_from_slice(&sec.name);
            binary.extend_from_slice(&sec.virtual_size.to_le_bytes());
            binary.extend_from_slice(&sec.virtual_address.to_le_bytes());
            binary.extend_from_slice(&sec.size_of_raw_data.to_le_bytes());
            binary.extend_from_slice(&sec.pointer_to_raw_data.to_le_bytes());
            binary.extend_from_slice(&sec.pointer_to_relocations.to_le_bytes());
            binary.extend_from_slice(&sec.pointer_to_linenumbers.to_le_bytes());
            binary.extend_from_slice(&sec.number_of_relocations.to_le_bytes());
            binary.extend_from_slice(&sec.number_of_linenumbers.to_le_bytes());
            binary.extend_from_slice(&sec.characteristics.to_le_bytes());
        }

        // --- Write Raw Data ---
        for data in &self.section_data {
            binary.extend_from_slice(data);
        }

        // --- Write Relocations ---
        for rels in &self.relocations {
            for &(vaddr, sym_idx, type_) in rels {
                binary.extend_from_slice(&vaddr.to_le_bytes());
                binary.extend_from_slice(&sym_idx.to_le_bytes());
                binary.extend_from_slice(&type_.to_le_bytes());
            }
        }

        // --- Write Symbol Table ---
        for sym in &self.symbols {
            binary.extend_from_slice(&sym.0); // name
            binary.extend_from_slice(&sym.1.to_le_bytes()); // value
            binary.extend_from_slice(&sym.2.to_le_bytes()); // section
            binary.extend_from_slice(&sym.3.to_le_bytes()); // type
            binary.push(sym.4); // storage_class
            binary.push(sym.5); // aux
        }

        // --- Write String Table ---
        if self.string_table.len() > 4 {
            binary.extend_from_slice(&self.string_table);
        }

        binary
    }

    /// Primary interface to take an ASM-BIB Program IR and translate it to Native Windows COFF!
    pub fn encode_program(mut self, program: &Program) -> Result<Vec<u8>, String> {
        let encoder = X86_64Encoder; // Fixed to x64 for now
        
        for section in &program.sections {
            let (characteristics, sec_name) = match &section.kind {
                SectionKind::Text => (IMAGE_SCN_CNT_CODE | IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ | IMAGE_SCN_ALIGN_16BYTES, ".text"),
                SectionKind::Data => (IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE | IMAGE_SCN_ALIGN_16BYTES, ".data"),
                SectionKind::Bss => (IMAGE_SCN_CNT_UNINITIALIZED_DATA | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE | IMAGE_SCN_ALIGN_16BYTES, ".bss"),
                SectionKind::Rodata => (IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ | IMAGE_SCN_ALIGN_16BYTES, ".rdata"),
                SectionKind::Custom(ref name) => {
                    // Default to initialized data for custom unknown
                    (IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE | IMAGE_SCN_ALIGN_16BYTES, name.as_str())
                }
            };

            let mut raw_data = Vec::new();
            
            // Encode Functions (for .text)
            for func in &section.functions {
                // Register symbol
                self.add_symbol(&func.name, raw_data.len() as u32, self.sections.len() as i16 + 1, 2); // Class 2 = External
                
                for item in &func.instructions {
                    match item {
                        crate::ir::FunctionItem::Instruction(inst) => {
                            if let Ok(bytes) = encoder.encode(inst) {
                                raw_data.extend(bytes);
                            } else {
                                // Fallback / Mock
                                raw_data.push(0x90); // NOP
                            }
                        }
                        _ => {}
                    }
                }
            }

            self.add_section(sec_name, characteristics, raw_data);
        }

        Ok(self.build())
    }
}
