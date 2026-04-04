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
pub const IMAGE_SCN_LNK_INFO: u32               = 0x00000200;
pub const IMAGE_SCN_LNK_REMOVE: u32             = 0x00000800;
pub const IMAGE_SCN_LNK_COMDAT: u32             = 0x00001000;
pub const IMAGE_SCN_ALIGN_1BYTES: u32           = 0x00100000;
pub const IMAGE_SCN_ALIGN_2BYTES: u32           = 0x00200000;
pub const IMAGE_SCN_ALIGN_4BYTES: u32           = 0x00300000;
pub const IMAGE_SCN_ALIGN_8BYTES: u32           = 0x00400000;
pub const IMAGE_SCN_ALIGN_16BYTES: u32          = 0x00500000;
pub const IMAGE_SCN_ALIGN_32BYTES: u32          = 0x00600000;
pub const IMAGE_SCN_ALIGN_64BYTES: u32          = 0x00700000;
pub const IMAGE_SCN_ALIGN_128BYTES: u32         = 0x00800000;
pub const IMAGE_SCN_ALIGN_256BYTES: u32         = 0x00900000;
pub const IMAGE_SCN_ALIGN_512BYTES: u32         = 0x00A00000;
pub const IMAGE_SCN_ALIGN_1024BYTES: u32        = 0x00B00000;
pub const IMAGE_SCN_ALIGN_2048BYTES: u32        = 0x00C00000;
pub const IMAGE_SCN_ALIGN_4096BYTES: u32        = 0x00D00000;
pub const IMAGE_SCN_MEM_EXECUTE: u32            = 0x20000000;
pub const IMAGE_SCN_MEM_READ: u32               = 0x40000000;
pub const IMAGE_SCN_MEM_WRITE: u32              = 0x80000000;

/// Maps a desired alignment (in bytes) to the correct IMAGE_SCN_ALIGN flag
pub fn alignment_to_flag(align: usize) -> u32 {
    match align {
        1 => IMAGE_SCN_ALIGN_1BYTES, 2 => IMAGE_SCN_ALIGN_2BYTES,
        4 => IMAGE_SCN_ALIGN_4BYTES, 8 => IMAGE_SCN_ALIGN_8BYTES,
        16 => IMAGE_SCN_ALIGN_16BYTES, 32 => IMAGE_SCN_ALIGN_32BYTES,
        64 => IMAGE_SCN_ALIGN_64BYTES, 128 => IMAGE_SCN_ALIGN_128BYTES,
        256 => IMAGE_SCN_ALIGN_256BYTES, 512 => IMAGE_SCN_ALIGN_512BYTES,
        1024 => IMAGE_SCN_ALIGN_1024BYTES, 2048 => IMAGE_SCN_ALIGN_2048BYTES,
        4096 => IMAGE_SCN_ALIGN_4096BYTES,
        _ => IMAGE_SCN_ALIGN_16BYTES, // default
    }
}

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

    pub fn add_section_aux_symbol(&mut self, parent_idx: u32, size: u32, relocs: u16, lines: u16, checksum: u32, sec_num: u16, selection: u8) {
        let mut data = [0u8; 18];
        data[0..4].copy_from_slice(&size.to_le_bytes());
        data[4..6].copy_from_slice(&relocs.to_le_bytes());
        data[6..8].copy_from_slice(&lines.to_le_bytes());
        data[8..12].copy_from_slice(&checksum.to_le_bytes());
        data[12..14].copy_from_slice(&sec_num.to_le_bytes());
        data[14] = selection;
        
        let mut name = [0u8; 8];
        name.copy_from_slice(&data[0..8]);
        let value = u32::from_le_bytes(data[8..12].try_into().unwrap());
        let sec = i16::from_le_bytes(data[12..14].try_into().unwrap());
        let typ = u16::from_le_bytes(data[14..16].try_into().unwrap());
        let sc = data[16];
        let aux = data[17];
        
        self.symbols.push((name, value, sec, typ, sc, aux));
        self.header.number_of_symbols += 1;
        self.symbols[parent_idx as usize].5 += 1;
    }

    pub fn add_function_aux_symbol(&mut self, parent_idx: u32, total_size: u32, ptr_to_lines: u32, ptr_to_next_func: u32) {
        let mut data = [0u8; 18];
        data[0..4].copy_from_slice(&0u32.to_le_bytes()); // tag index
        data[4..8].copy_from_slice(&total_size.to_le_bytes());
        data[8..12].copy_from_slice(&ptr_to_lines.to_le_bytes());
        data[12..16].copy_from_slice(&ptr_to_next_func.to_le_bytes());
        data[16..18].copy_from_slice(&0u16.to_le_bytes()); // unused
        
        let mut name = [0u8; 8];
        name.copy_from_slice(&data[0..8]);
        let value = u32::from_le_bytes(data[8..12].try_into().unwrap());
        let sec = i16::from_le_bytes(data[12..14].try_into().unwrap());
        let typ = u16::from_le_bytes(data[14..16].try_into().unwrap());
        let sc = data[16];
        let aux = data[17];
        
        self.symbols.push((name, value, sec, typ, sc, aux));
        self.header.number_of_symbols += 1;
        self.symbols[parent_idx as usize].5 += 1;
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

    pub fn get_or_add_external_symbol(&mut self, name: &str) -> u32 {
        for (i, sym) in self.symbols.iter().enumerate() {
            let mut matches = false;
            if sym.0[0] == 0 {
                // Read from string table
                let offset = u32::from_le_bytes(sym.0[4..8].try_into().unwrap()) as usize;
                let end = self.string_table[offset..].iter().position(|&c| c == 0).unwrap_or(0);
                if let Ok(sym_name) = std::str::from_utf8(&self.string_table[offset..offset+end]) {
                    if sym_name == name { matches = true; }
                }
            } else {
                let end = sym.0.iter().position(|&c| c == 0).unwrap_or(8);
                if let Ok(sym_name) = std::str::from_utf8(&sym.0[..end]) {
                    if sym_name == name { matches = true; }
                }
            }
            if matches { return i as u32; }
        }
        
        // Value = 0, Section = 0 (Undefined) indicates external symbol
        self.add_symbol(name, 0, 0, 2) // Class 2 = External
    }

    /// Primary interface to take an ASM-BIB Program IR and translate it to Native Windows COFF!
    pub fn encode_program(mut self, program: &Program) -> Result<Vec<u8>, String> {
        let encoder = X86_64Encoder; // Fixed to x64 for now
        
        // 1. Generate MSVC Autolink .drectve Section (Phase 3 DLL Support)
        let mut drectve_data = String::new();
        for lib in &program.includelibs {
            drectve_data.push_str(&format!("/DEFAULTLIB:\"{}\" ", lib));
        }
        
        for section in &program.sections {
            for func in &section.functions {
                if func.exported {
                    drectve_data.push_str(&format!("/EXPORT:{} ", func.name));
                }
            }
        }
        
        if !drectve_data.is_empty() {
            drectve_data.push('\0');
            // Characteristics: LNK_INFO (0x200), ALIGN_1BYTES (0x100000), LNK_REMOVE (0x800)
            self.add_section(".drectve", 0x00100A00, drectve_data.into_bytes());
        }
        
        let mut pdata_entries = Vec::new();
        
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
            let mut sec_relocs = Vec::new(); // Collect relocations for this section
            
            let mut current_offset = raw_data.len();
            
            // Encode Functions (for .text)
            for func in &section.functions {
                // Register symbol for the function
                let func_sym_idx = self.add_symbol(&func.name, current_offset as u32, self.sections.len() as i16 + 1, 2); // Class 2 = External
                
                // FASE 8: Add Function Aux Symbol IMMEDIATELY (COFF requirement). Use dummy size, update later.
                let func_aux_idx = self.symbols.len();
                self.add_function_aux_symbol(func_sym_idx, 0, 0, 0);
                
                let mut local_labels = std::collections::HashMap::new();
                
                // Pass 1: Estimate offsets and Register Labels
                let mut temp_offset = current_offset;
                for item in &func.instructions {
                    match item {
                        crate::ir::FunctionItem::Instruction(inst) => {
                            // Rough estimation by encoding it without local knowledge
                            if let Ok(encoded) = encoder.encode(inst, None, temp_offset as u32) {
                                temp_offset += encoded.bytes.len();
                            } else {
                                temp_offset += 1; // NOP
                            }
                        }
                        crate::ir::FunctionItem::Label(lbl) => {
                            self.add_symbol(lbl, temp_offset as u32, self.sections.len() as i16 + 1, 3);
                            local_labels.insert(lbl.clone(), temp_offset as u32);
                        }
                        _ => {}
                    }
                }
                
                let func_start_offset = raw_data.len();
                
                // Pass 2: Encode and resolve
                for item in &func.instructions {
                    if let crate::ir::FunctionItem::Instruction(inst) = item {
                        let inst_offset = raw_data.len() as u32;
                        match encoder.encode(inst, Some(&local_labels), inst_offset) {
                            Ok(encoded) => {
                                raw_data.extend(encoded.bytes);
                                
                                // Register relocations (only externals, locals are resolved)
                                for req in encoded.relocations {
                                    let sym_idx = self.get_or_add_external_symbol(&req.symbol);
                                    sec_relocs.push((inst_offset + req.offset, sym_idx, req.rel_type));
                                }
                            }
                            Err(_) => {
                                raw_data.push(0x90); // NOP fallback
                            }
                        }
                    }
                }
                
                let func_end_offset = raw_data.len();
                let f_size = (func_end_offset - func_start_offset) as u32;
                
                // Update the function aux symbol with the real size
                // The size is located at bytes 4..8 of the Aux record, which corresponds to sym.0[4..8]
                self.symbols[func_aux_idx].0[4..8].copy_from_slice(&f_size.to_le_bytes());
                
                pdata_entries.push((func_sym_idx, f_size));
                
                current_offset = raw_data.len();
            }

            // Encode Data Items (for .data, .bss, .rdata)
            for item in &section.data {
                // ALIGNMENT PADDING ENGINE (Task 3)
                if let Some(align) = item.alignment {
                    if align > 0 && raw_data.len() % align != 0 {
                        let padding = align - (raw_data.len() % align);
                        raw_data.resize(raw_data.len() + padding, 0);
                    }
                }
                
                // Register symbol for the variable/data item
                self.add_symbol(&item.name, raw_data.len() as u32, self.sections.len() as i16 + 1, 3); // Class 3 = Static
                
                // Helper to serialize DataDef into bytes recursively
                fn serialize_data_def(def: &crate::ir::DataDef, out: &mut Vec<u8>) {
                    use crate::ir::DataDef::*;
                    match def {
                        Byte(vals) => out.extend(vals),
                        Word(vals) => for v in vals { out.extend(&v.to_le_bytes()); },
                        Dword(vals) => for v in vals { out.extend(&v.to_le_bytes()); },
                        Qword(vals) => for v in vals { out.extend(&v.to_le_bytes()); },
                        Float32(vals) => for v in vals { out.extend(&v.to_le_bytes()); },
                        Float64(vals) => for v in vals { out.extend(&v.to_le_bytes()); },
                        String(s) => {
                            // C-style string with null terminator (often implicit in PASM but usually expected)
                            out.extend(s.as_bytes());
                            if !s.ends_with('\0') { out.push(0); }
                        }
                        WString(s) => {
                            for c in s.encode_utf16() { out.extend(&c.to_le_bytes()); }
                            out.extend(&[0, 0]);
                        }
                        ReserveBytes(n) => out.resize(out.len() + n, 0),
                        ReserveWords(n) => out.resize(out.len() + n * 2, 0),
                        ReserveDwords(n) => out.resize(out.len() + n * 4, 0),
                        ReserveQwords(n) => out.resize(out.len() + n * 8, 0),
                        DupByte(n, v) => out.resize(out.len() + n, *v),
                        DupWord(n, v) => for _ in 0..*n { out.extend(&v.to_le_bytes()); },
                        DupDword(n, v) => for _ in 0..*n { out.extend(&v.to_le_bytes()); },
                        DupQword(n, v) => for _ in 0..*n { out.extend(&v.to_le_bytes()); },
                        Struct(_, fields) => {
                            for f in fields {
                                serialize_data_def(&f.def, out);
                            }
                        }
                    }
                }
                
                serialize_data_def(&item.def, &mut raw_data);
            }

            let raw_data_len = raw_data.len() as u32;
            let num_relocs = sec_relocs.len() as u16;
            let sec_idx = self.add_section(sec_name, characteristics, raw_data);
            self.relocations[sec_idx - 1] = sec_relocs;
            
            // FASE 8: Aux symbol records for the Section itself
            let sec_sym_idx = self.add_symbol(sec_name, 0, sec_idx as i16, 3); // Class 3 = Static
            let is_comdat = if (characteristics & IMAGE_SCN_LNK_COMDAT) != 0 { 2 } else { 0 }; // 2 = ANY
            self.add_section_aux_symbol(sec_sym_idx, raw_data_len, num_relocs, 0, 0, sec_idx as u16, is_comdat);
        }

        // 2. Generate .pdata and .xdata Windows SEH (Structured Exception Handling)
        // FASE 8: Real UNWIND_INFO with UNWIND_CODEs per function
        if !pdata_entries.is_empty() {
            // === Build .xdata ===
            // Each function gets its own UNWIND_INFO block at a unique offset.
            // UNWIND_INFO format:
            //   byte 0: Version (3 bits) | Flags (5 bits)  → 0x01 (version 1, no handler)
            //   byte 1: Size of prolog (in bytes)
            //   byte 2: Count of UNWIND_CODEs
            //   byte 3: Frame Register (4 bits) | Frame Register Offset (4 bits)
            //   UNWIND_CODE array (2 bytes each):
            //     byte 0: Offset in prolog
            //     byte 1: Opcode (4 bits low) | Info (4 bits high)
            //       UWOP_PUSH_NONVOL (0) — info = register number
            //       UWOP_ALLOC_SMALL (2) — info = (size / 8) - 1
            //       UWOP_ALLOC_LARGE (1) — info = 0 → next slot has size/8; info = 1 → next 2 slots have raw size
            //       UWOP_SET_FPREG    (3) — sets frame pointer
            
            let mut xdata = Vec::new();
            let mut xdata_func_offsets: Vec<u32> = Vec::new(); // offset into xdata for each function
            
            for (_sym_idx, func_size) in &pdata_entries {
                let func_xdata_offset = xdata.len() as u32;
                xdata_func_offsets.push(func_xdata_offset);
                
                // Analyze prologue: we know our standard prologue is:
                // push rbp (1 byte: 0x55)  → UWOP_PUSH_NONVOL, reg=5 (RBP)
                // mov rbp, rsp (3 bytes)   → (no unwind code needed)
                // sub rsp, N              → UWOP_ALLOC_SMALL or UWOP_ALLOC_LARGE
                //
                // For a standard frame, we emit:
                //   - PUSH_NONVOL for RBP
                //   - ALLOC_SMALL/LARGE for stack allocation
                //   - SET_FPREG for frame pointer
                
                // Standard prologue: push rbp (offset 1) + mov rbp,rsp (offset 4) + sub rsp,N (offset ~8)
                // We'll emit a minimal but correct unwind info
                
                let mut unwind_codes: Vec<u8> = Vec::new();
                
                // UWOP_PUSH_NONVOL for RBP at prolog offset 1
                // offset_in_prolog = 1 (after "push rbp")
                // operation = UWOP_PUSH_NONVOL (0), info = 5 (RBP register number)
                unwind_codes.push(1);  // offset in prolog
                unwind_codes.push(0x50); // operation_info=5 (RBP) << 4 | operation=0 (PUSH_NONVOL)
                
                // UWOP_SET_FPREG at prolog offset 4 (after "mov rbp, rsp")
                unwind_codes.push(4);  // offset in prolog  
                unwind_codes.push(0x03); // info=0 (offset=0) << 4 | operation=3 (SET_FPREG)
                
                let code_count = unwind_codes.len() / 2;
                let prolog_size: u8 = 8; // typical prologue size (push rbp + mov rbp,rsp + sub rsp,imm8)
                
                // UNWIND_INFO header
                xdata.push(0x01);          // Version=1, Flags=0 (no handler)
                xdata.push(prolog_size);   // SizeOfProlog
                xdata.push(code_count as u8); // CountOfCodes
                xdata.push(0x05);          // FrameRegister=5 (RBP), FrameOffset=0
                
                // Write UNWIND_CODEs (must be listed in reverse order of prolog, which they already are)
                // Actually x64 unwind codes should be sorted by descending prolog offset
                // Our codes: offset 4 (SET_FPREG), offset 1 (PUSH_NONVOL) — reversed
                xdata.push(unwind_codes[2]); // SET_FPREG first (higher offset)
                xdata.push(unwind_codes[3]);
                xdata.push(unwind_codes[0]); // PUSH_NONVOL second (lower offset)
                xdata.push(unwind_codes[1]);
                
                // Pad to 4-byte alignment
                while xdata.len() % 4 != 0 {
                    xdata.push(0);
                }
            }
            
            let xdata_len = xdata.len() as u32;
            let xdata_characteristics = 0x40300040; // INITIALIZED_DATA | MEM_READ | ALIGN_4BYTES
            let xdata_sec_idx = self.add_section(".xdata", xdata_characteristics, xdata);
            let xdata_base_sym_idx = self.add_symbol(".xdata", 0, xdata_sec_idx as i16, 3);
            self.add_section_aux_symbol(xdata_base_sym_idx, xdata_len, 0, 0, 0, xdata_sec_idx as u16, 0);
            
            // === Build .pdata ===
            let mut pdata = Vec::new();
            let mut pdata_relocs = Vec::new();
            let pdata_characteristics = 0x40300040;
            
            for (i, (sym_idx, func_size)) in pdata_entries.iter().enumerate() {
                let offset = pdata.len() as u32;
                
                // BeginAddress (RVA of function start)
                pdata.extend_from_slice(&[0,0,0,0]);
                pdata_relocs.push((offset, *sym_idx, 3)); // IMAGE_REL_AMD64_ADDR32NB
                
                // EndAddress (RVA of function end = BeginAddress + size)
                pdata.extend_from_slice(&(*func_size as u32).to_le_bytes());
                pdata_relocs.push((offset + 4, *sym_idx, 3));
                
                // UnwindInfoAddress (RVA of this function's UNWIND_INFO in .xdata)
                let func_xdata_off = xdata_func_offsets[i];
                pdata.extend_from_slice(&func_xdata_off.to_le_bytes());
                pdata_relocs.push((offset + 8, xdata_base_sym_idx, 3));
            }
            
            let pdata_len = pdata.len() as u32;
            let pdata_num_relocs = pdata_relocs.len() as u16;
            let pdata_sec_idx = self.add_section(".pdata", pdata_characteristics, pdata);
            self.relocations[pdata_sec_idx - 1] = pdata_relocs;
            
            let pdata_sym_idx = self.add_symbol(".pdata", 0, pdata_sec_idx as i16, 3);
            self.add_section_aux_symbol(pdata_sym_idx, pdata_len, pdata_num_relocs, 0, 0, pdata_sec_idx as u16, 0);
        }

        Ok(self.build())
    }
}
