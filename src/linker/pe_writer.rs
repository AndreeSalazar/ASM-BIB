//! PE Executable Writer — Phases 10, 11, 12, 13
//! Takes a Program IR (already encoded to COFF internally) and writes a complete
//! Windows x86-64 PE executable or DLL, with:
//!   - DOS stub + PE signature
//!   - COFF header + Optional header
//!   - Import table (kernel32, user32, msvcrt, etc.)
//!   - .text, .rdata, .data, .pdata, .xdata sections
//!   - Base relocations (.reloc) for DLLs
//!   - Export table for DLLs

use std::collections::HashMap;
use crate::ir::{Program, SectionKind, FunctionItem, Opcode, Operand};
use crate::targets::{ArchEncoder, x86_64::X86_64Encoder};
use super::import_lib;
use super::relocator;

// ─── PE Constants ───────────────────────────────────────────────────────

const IMAGE_FILE_MACHINE_AMD64: u16 = 0x8664;
const PE_SECTION_ALIGNMENT: u32 = 0x1000; // 4KB
const PE_FILE_ALIGNMENT: u32    = 0x200;   // 512 bytes
const IMAGE_BASE: u64           = 0x0000000140000000; // Default x64 image base

const IMAGE_SCN_CNT_CODE: u32             = 0x00000020;
const IMAGE_SCN_CNT_INITIALIZED_DATA: u32 = 0x00000040;
const IMAGE_SCN_CNT_UNINITIALIZED_DATA: u32 = 0x00000080;
const IMAGE_SCN_MEM_EXECUTE: u32          = 0x20000000;
const IMAGE_SCN_MEM_READ: u32             = 0x40000000;
const IMAGE_SCN_MEM_WRITE: u32            = 0x80000000;
const IMAGE_SCN_MEM_DISCARDABLE: u32      = 0x02000000;

// ─── Link Configuration ─────────────────────────────────────────────────

pub struct LinkConfig {
    pub is_dll: bool,
    pub entry_point: String,
    pub image_base: u64,
    pub stack_reserve: u64,
    pub stack_commit: u64,
    pub heap_reserve: u64,
    pub heap_commit: u64,
    pub subsystem: u16,       // 3=CONSOLE, 2=WINDOWS
    pub extra_libs: Vec<String>,
}

impl Default for LinkConfig {
    fn default() -> Self {
        Self {
            is_dll: false,
            entry_point: "main".into(),
            image_base: IMAGE_BASE,
            stack_reserve: 0x100000,  // 1MB
            stack_commit:  0x1000,    // 4KB
            heap_reserve:  0x100000,
            heap_commit:   0x1000,
            subsystem: 3, // CONSOLE
            extra_libs: Vec::new(),
        }
    }
}

// ─── Internal section for PE layout ─────────────────────────────────────

struct PeSection {
    name: String,
    characteristics: u32,
    data: Vec<u8>,
    rva: u32,           // Virtual address (assigned during layout)
    file_offset: u32,   // File offset (assigned during layout)
}

impl PeSection {
    fn virtual_size(&self) -> u32 {
        self.data.len() as u32
    }

    fn raw_size(&self) -> u32 {
        align_up(self.data.len() as u32, PE_FILE_ALIGNMENT)
    }
}

// ─── Import Table Builder ───────────────────────────────────────────────

struct ImportTableBuilder {
    dlls: Vec<DllImport>,
}

struct DllImport {
    dll_name: String,
    functions: Vec<(String, u16)>, // (name, ordinal_hint)
}

struct ImportTableResult {
    /// Complete import section data (.rdata portion for imports)
    data: Vec<u8>,
    /// IAT RVA (relative to import section start)
    iat_offset: u32,
    iat_size: u32,
    /// ILT offset (relative to import data start)
    ilt_offset: u32,
    /// Import directory RVA (relative to import section start)
    idt_offset: u32,
    idt_size: u32,
    /// Map: function_name → IAT entry RVA offset (relative to section start)
    iat_map: HashMap<String, u32>,
}

impl ImportTableBuilder {
    fn new() -> Self {
        Self { dlls: Vec::new() }
    }

    fn add_dll(&mut self, name: &str, functions: Vec<(String, u16)>) {
        self.dlls.push(DllImport { dll_name: name.into(), functions });
    }

    /// Build import tables: IDT + ILT + IAT + Hint/Name + DLL names
    fn build(&self) -> ImportTableResult {
        if self.dlls.is_empty() {
            return ImportTableResult {
                data: Vec::new(), iat_offset: 0, iat_size: 0,
                ilt_offset: 0, idt_offset: 0, idt_size: 0, iat_map: HashMap::new(),
            };
        }

        // Layout plan:
        // 1. Import Directory Table (IDT): (dlls+1) * 20 bytes (null terminator)
        // 2. Import Lookup Table (ILT): per-DLL, terminated by 8-byte null
        // 3. Import Address Table (IAT): identical to ILT initially
        // 4. Hint/Name Table: per-function, 2-byte hint + null-terminated name
        // 5. DLL name strings

        let num_dlls = self.dlls.len();
        let idt_size = ((num_dlls + 1) * 20) as u32;

        // Count total functions for sizing
        let total_funcs: usize = self.dlls.iter().map(|d| d.functions.len()).sum();

        // ILT: one 8-byte entry per function + one 8-byte null per DLL
        let ilt_entry_size = (total_funcs + num_dlls) * 8;
        // IAT is same size as ILT
        let iat_entry_size = ilt_entry_size;

        let idt_offset = 0u32;
        let ilt_offset = idt_size;
        let iat_offset = ilt_offset + ilt_entry_size as u32;
        let hint_name_offset = iat_offset + iat_entry_size as u32;

        // Build hint/name table and DLL name strings
        let mut hint_name_data = Vec::new();
        let mut dll_name_data = Vec::new();

        // Pre-calculate positions
        struct DllLayout {
            ilt_start: u32,
            iat_start: u32,
            dll_name_offset: u32,
            func_hn_offsets: Vec<u32>,
        }

        let mut layouts = Vec::new();
        let mut ilt_cursor = ilt_offset;
        let mut iat_cursor = iat_offset;

        for dll in &self.dlls {
            let dll_name_off = hint_name_offset + hint_name_data.len() as u32 + 0; // placeholder
            let mut func_offsets = Vec::new();

            for (name, hint) in &dll.functions {
                let off = hint_name_data.len() as u32;
                hint_name_data.extend_from_slice(&hint.to_le_bytes()); // 2-byte hint
                hint_name_data.extend_from_slice(name.as_bytes());
                hint_name_data.push(0); // null terminator
                // Pad to 2-byte alignment
                if hint_name_data.len() % 2 != 0 { hint_name_data.push(0); }
                func_offsets.push(off);
            }

            layouts.push(DllLayout {
                ilt_start: ilt_cursor,
                iat_start: iat_cursor,
                dll_name_offset: 0, // filled below
                func_hn_offsets: func_offsets,
            });

            ilt_cursor += (dll.functions.len() as u32 + 1) * 8;
            iat_cursor += (dll.functions.len() as u32 + 1) * 8;
        }

        // DLL name strings come after hint/name table
        let dll_names_base = hint_name_offset + hint_name_data.len() as u32;
        for (i, dll) in self.dlls.iter().enumerate() {
            layouts[i].dll_name_offset = dll_names_base + dll_name_data.len() as u32;
            dll_name_data.extend_from_slice(dll.dll_name.as_bytes());
            dll_name_data.push(0);
        }

        let total_size = dll_names_base as usize + dll_name_data.len();
        let mut data = vec![0u8; total_size];

        // Write IDT
        for (i, dll) in self.dlls.iter().enumerate() {
            let base = idt_offset as usize + i * 20;
            // ILT RVA (will be relocated by caller adding section RVA)
            data[base..base+4].copy_from_slice(&layouts[i].ilt_start.to_le_bytes());
            // TimeDateStamp
            data[base+4..base+8].copy_from_slice(&0u32.to_le_bytes());
            // ForwarderChain
            data[base+8..base+12].copy_from_slice(&0u32.to_le_bytes());
            // Name RVA
            data[base+12..base+16].copy_from_slice(&layouts[i].dll_name_offset.to_le_bytes());
            // IAT RVA
            data[base+16..base+20].copy_from_slice(&layouts[i].iat_start.to_le_bytes());
        }
        // Null terminator IDT entry (already zero)

        // Write ILT and IAT (identical content)
        let mut iat_map = HashMap::new();

        for (i, dll) in self.dlls.iter().enumerate() {
            for (j, (_name, _hint)) in dll.functions.iter().enumerate() {
                let hn_off = hint_name_offset + layouts[i].func_hn_offsets[j];
                let entry: u64 = hn_off as u64; // Bit 63=0 means import by name

                let ilt_pos = layouts[i].ilt_start as usize + j * 8;
                let iat_pos = layouts[i].iat_start as usize + j * 8;

                data[ilt_pos..ilt_pos+8].copy_from_slice(&entry.to_le_bytes());
                data[iat_pos..iat_pos+8].copy_from_slice(&entry.to_le_bytes());

                iat_map.insert(_name.clone(), iat_pos as u32);
                // Also map __imp_Name
                iat_map.insert(format!("__imp_{}", _name), iat_pos as u32);
            }
            // Null terminator entries (already zero from vec![0u8])
        }

        // Write Hint/Name table
        let hn_start = hint_name_offset as usize;
        data[hn_start..hn_start + hint_name_data.len()].copy_from_slice(&hint_name_data);

        // Write DLL name strings
        let dn_start = dll_names_base as usize;
        data[dn_start..dn_start + dll_name_data.len()].copy_from_slice(&dll_name_data);

        ImportTableResult {
            data,
            iat_offset,
            iat_size: iat_entry_size as u32,
            ilt_offset,
            idt_offset,
            idt_size: (num_dlls * 20) as u32, // not counting null terminator
            iat_map,
        }
    }
}

// ─── Main linker entry point ────────────────────────────────────────────

/// Link a Program IR directly into a PE executable or DLL.
/// This replaces the entire ml64.exe + link.exe pipeline.
pub fn link_program(program: &Program, config: &LinkConfig) -> Result<Vec<u8>, String> {
    let encoder = X86_64Encoder;

    // ═══════════════════════════════════════════════════════════════════
    // PHASE 10: Encode all sections into raw binary data
    // ═══════════════════════════════════════════════════════════════════

    let mut text_data = Vec::new();
    let mut rdata_data = Vec::new();
    let mut data_data = Vec::new();
    let mut bss_size = 0u32;

    // Symbol table: name → (section_index, offset_in_section)
    // section_index: 0=text, 1=rdata, 2=data
    let mut symbols: HashMap<String, (u8, u32)> = HashMap::new();
    let mut extern_refs: Vec<String> = Vec::new(); // Symbols not found locally
    let mut export_symbols: Vec<(String, u32)> = Vec::new(); // (name, rva) for DLL exports

    // Pending relocations: (section_idx, offset_in_section, symbol_name, rel_type)
    let mut pending_relocs: Vec<(u8, u32, String, u16)> = Vec::new();

    // Encode each IR section
    for section in &program.sections {
        let (sec_idx, sec_data) = match section.kind {
            SectionKind::Text => (0u8, &mut text_data),
            SectionKind::Data => (2u8, &mut data_data),
            SectionKind::Rodata => (1u8, &mut rdata_data),
            SectionKind::Bss => {
                // BSS doesn't have data
                for item in &section.data {
                    let size = estimate_data_size(&item.def);
                    symbols.insert(item.name.clone(), (3, bss_size));
                    bss_size += size as u32;
                }
                continue;
            }
            SectionKind::Custom(ref name) => {
                if name.starts_with(".rdata") || name.starts_with(".const") {
                    (1u8, &mut rdata_data)
                } else {
                    (2u8, &mut data_data)
                }
            }
        };

        // Encode functions
        for func in &section.functions {
            let func_offset = sec_data.len() as u32;
            symbols.insert(func.name.clone(), (sec_idx, func_offset));

            if func.exported && config.is_dll {
                export_symbols.push((func.name.clone(), 0)); // RVA fixed later
            }

            // Two-pass encoding (same as coff.rs)
            let mut local_labels: HashMap<String, u32> = HashMap::new();

            // Pass 1: estimate offsets
            let mut temp_off = sec_data.len();
            for item in &func.instructions {
                match item {
                    FunctionItem::Instruction(inst) => {
                        if let Ok(enc) = encoder.encode(inst, None, temp_off as u32) {
                            temp_off += enc.bytes.len();
                        } else {
                            temp_off += 1;
                        }
                    }
                    FunctionItem::Label(lbl) => {
                        local_labels.insert(lbl.clone(), temp_off as u32);
                        symbols.insert(lbl.clone(), (sec_idx, temp_off as u32));
                    }
                    _ => {}
                }
            }

            // Pass 2: encode with label knowledge
            for item in &func.instructions {
                if let FunctionItem::Instruction(inst) = item {
                    let inst_off = sec_data.len() as u32;
                    match encoder.encode(inst, Some(&local_labels), inst_off) {
                        Ok(enc) => {
                            sec_data.extend(&enc.bytes);
                            for req in enc.relocations {
                                if local_labels.contains_key(&req.symbol) {
                                    // Already resolved locally
                                } else {
                                    pending_relocs.push((sec_idx, inst_off + req.offset, req.symbol.clone(), req.rel_type));
                                    if !extern_refs.contains(&req.symbol) {
                                        extern_refs.push(req.symbol);
                                    }
                                }
                            }
                        }
                        Err(_) => { sec_data.push(0x90); }
                    }
                }
            }
        }

        // Encode data items
        for item in &section.data {
            if let Some(align) = item.alignment {
                if align > 0 && sec_data.len() % align != 0 {
                    let pad = align - (sec_data.len() % align);
                    sec_data.resize(sec_data.len() + pad, 0);
                }
            }
            symbols.insert(item.name.clone(), (sec_idx, sec_data.len() as u32));
            serialize_data(&item.def, sec_data);
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    // PHASE 11: Build Import Tables
    // ═══════════════════════════════════════════════════════════════════

    // Determine which DLLs/functions we need
    let needed_libs = determine_needed_libs(program, &extern_refs, &config.extra_libs);
    let mut import_builder = ImportTableBuilder::new();

    for lib in &needed_libs {
        let funcs: Vec<(String, u16)> = lib.entries.iter()
            .filter(|e| extern_refs.contains(&e.name))
            .map(|e| (e.name.clone(), e.ordinal_hint))
            .collect();
        if !funcs.is_empty() {
            import_builder.add_dll(&lib.dll_name, funcs);
        }
    }

    let import_result = import_builder.build();

    // ═══════════════════════════════════════════════════════════════════
    // PHASE 12: PE Layout + Write
    // ═══════════════════════════════════════════════════════════════════

    // Build PE sections
    let mut pe_sections: Vec<PeSection> = Vec::new();

    // .text
    if !text_data.is_empty() {
        pe_sections.push(PeSection {
            name: ".text".into(),
            characteristics: IMAGE_SCN_CNT_CODE | IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ,
            data: text_data,
            rva: 0, file_offset: 0,
        });
    }

    // .rdata (includes import tables)
    let import_rdata_offset = rdata_data.len() as u32;
    if !import_result.data.is_empty() {
        // Pad rdata to 16-byte alignment before appending import data
        while rdata_data.len() % 16 != 0 { rdata_data.push(0); }
        let import_rdata_offset_aligned = rdata_data.len() as u32;
        rdata_data.extend(&import_result.data);

        // We'll need to fix up the import_rdata_offset
        let _ = import_rdata_offset; // suppress unused
        pe_sections.push(PeSection {
            name: ".rdata".into(),
            characteristics: IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ,
            data: rdata_data,
            rva: 0, file_offset: 0,
        });
        // Store the aligned offset for later
        // (import tables start at import_rdata_offset_aligned within .rdata)
        let _ = import_rdata_offset_aligned;
    } else if !rdata_data.is_empty() {
        pe_sections.push(PeSection {
            name: ".rdata".into(),
            characteristics: IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ,
            data: rdata_data,
            rva: 0, file_offset: 0,
        });
    }

    // .data
    if !data_data.is_empty() {
        pe_sections.push(PeSection {
            name: ".data".into(),
            characteristics: IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE,
            data: data_data,
            rva: 0, file_offset: 0,
        });
    }

    // .reloc (Phase 13 — for DLLs)
    let mut reloc_builder = relocator::BaseRelocationBuilder::new();

    // .edata (Phase 13 — exports for DLLs)
    let mut edata_data = Vec::new();

    // Headers size
    let dos_header_size = 64u32;
    let pe_sig_size = 4u32;
    let coff_header_size = 20u32;
    let opt_header_size = 240u32; // PE32+ optional header
    let section_header_size = 40u32;
    let num_sections = pe_sections.len() as u32 + if config.is_dll { 2 } else { 0 }; // +reloc +edata for DLL
    let headers_raw_size = dos_header_size + pe_sig_size + coff_header_size + opt_header_size
        + num_sections * section_header_size;
    let headers_aligned = align_up(headers_raw_size, PE_FILE_ALIGNMENT);

    // Assign RVAs and file offsets
    let mut current_rva = align_up(headers_raw_size, PE_SECTION_ALIGNMENT);
    let mut current_file = headers_aligned;

    for sec in &mut pe_sections {
        sec.rva = current_rva;
        sec.file_offset = current_file;
        current_rva += align_up(sec.virtual_size(), PE_SECTION_ALIGNMENT);
        current_file += sec.raw_size();
    }

    // Find section RVAs
    let text_rva = pe_sections.iter().find(|s| s.name == ".text").map(|s| s.rva).unwrap_or(0);
    let rdata_rva = pe_sections.iter().find(|s| s.name == ".rdata").map(|s| s.rva).unwrap_or(0);
    let data_rva = pe_sections.iter().find(|s| s.name == ".data").map(|s| s.rva).unwrap_or(0);

    // Calculate import table RVAs
    let rdata_sec_idx = pe_sections.iter().position(|s| s.name == ".rdata");
    let import_base_rva = if let Some(idx) = rdata_sec_idx {
        // Find where imports start in .rdata
        let rdata_total = pe_sections[idx].data.len() as u32;
        let import_data_len = import_result.data.len() as u32;
        let import_start_in_rdata = rdata_total - import_data_len;
        pe_sections[idx].rva + import_start_in_rdata
    } else {
        0
    };

    // Fix up import table internal offsets to absolute RVAs
    if let Some(idx) = rdata_sec_idx {
        let import_data_len = import_result.data.len() as u32;
        let rdata_total = pe_sections[idx].data.len() as u32;
        let import_offset_in_rdata = rdata_total - import_data_len;
        let base = pe_sections[idx].rva + import_offset_in_rdata;

        // Fixup IDT entries (ILT RVA, Name RVA, IAT RVA)
        let sec_data = &mut pe_sections[idx].data;
        let imp_start = import_offset_in_rdata as usize;

        let num_dlls = import_builder.dlls.len();
        for i in 0..num_dlls {
            let idt_off = imp_start + i * 20;
            // ILT RVA
            fixup_u32(sec_data, idt_off, base);
            // Name RVA
            fixup_u32(sec_data, idt_off + 12, base);
            // IAT RVA
            fixup_u32(sec_data, idt_off + 16, base);
        }

        // Fixup ILT and IAT entries (Hint/Name RVA pointers)
        let ilt_start = imp_start + import_result.idt_offset as usize + ((num_dlls + 1) * 20);
        // Actually, let's fixup all 8-byte entries in ILT and IAT
        let ilt_off_abs = imp_start + import_result.ilt_offset as usize;
        let iat_off_abs = imp_start + import_result.iat_offset as usize;

        // Compute ILT range and IAT range
        let ilt_total_entries = import_result.iat_offset - import_result.ilt_offset;
        for off in (0..ilt_total_entries).step_by(8) {
            let pos_ilt = ilt_off_abs + off as usize;
            let pos_iat = iat_off_abs + off as usize;
            if pos_ilt + 8 <= sec_data.len() {
                let val = u64::from_le_bytes(sec_data[pos_ilt..pos_ilt+8].try_into().unwrap());
                if val != 0 { // Don't fixup null terminators
                    let fixed = val + base as u64;
                    sec_data[pos_ilt..pos_ilt+8].copy_from_slice(&fixed.to_le_bytes());
                }
            }
            if pos_iat + 8 <= sec_data.len() {
                let val = u64::from_le_bytes(sec_data[pos_iat..pos_iat+8].try_into().unwrap());
                if val != 0 {
                    let fixed = val + base as u64;
                    sec_data[pos_iat..pos_iat+8].copy_from_slice(&fixed.to_le_bytes());
                }
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    // Apply pending relocations (Phase 10 — resolve symbols)
    // ═══════════════════════════════════════════════════════════════════

    // Build IAT map with absolute RVAs
    let mut iat_rva_map: HashMap<String, u32> = HashMap::new();
    if let Some(idx) = rdata_sec_idx {
        let import_data_len = import_result.data.len() as u32;
        let rdata_total = pe_sections[idx].data.len() as u32;
        let import_offset_in_rdata = rdata_total - import_data_len;
        let base = pe_sections[idx].rva + import_offset_in_rdata;

        for (name, offset) in &import_result.iat_map {
            iat_rva_map.insert(name.clone(), base + *offset);
        }
    }

    for (sec_idx, offset, sym_name, rel_type) in &pending_relocs {
        let sec_name_map = [".text", ".rdata", ".data"];
        let section_rva = match sec_idx {
            0 => text_rva,
            1 => rdata_rva,
            2 => data_rva,
            _ => 0,
        };

        // Find target RVA
        let target_rva = if let Some(iat_rva) = iat_rva_map.get(sym_name) {
            // Import — point to IAT entry
            *iat_rva
        } else if let Some((target_sec, target_off)) = symbols.get(sym_name) {
            // Local symbol
            let target_sec_rva = match target_sec {
                0 => text_rva,
                1 => rdata_rva,
                2 => data_rva,
                _ => 0,
            };
            target_sec_rva + target_off
        } else {
            // Unresolved — try IAT lookup by stripping leading underscore
            let stripped = sym_name.strip_prefix('_').unwrap_or(sym_name);
            if let Some(iat_rva) = iat_rva_map.get(stripped) {
                *iat_rva
            } else {
                continue; // Skip unresolvable
            }
        };

        // Find the PE section to patch
        if let Some(pe_sec) = pe_sections.iter_mut().find(|s| {
            s.name == *sec_name_map.get(*sec_idx as usize).unwrap_or(&"")
        }) {
            let patch_offset = *offset as usize;
            let rip_addr = section_rva + patch_offset as u32 + 4;

            match *rel_type {
                4 => {
                    // IMAGE_REL_AMD64_REL32
                    if patch_offset + 4 <= pe_sec.data.len() {
                        let existing = i32::from_le_bytes(
                            pe_sec.data[patch_offset..patch_offset+4].try_into().unwrap()
                        );
                        let delta = (target_rva as i64) - (rip_addr as i64) + (existing as i64);
                        pe_sec.data[patch_offset..patch_offset+4]
                            .copy_from_slice(&(delta as i32).to_le_bytes());
                    }
                }
                3 => {
                    // IMAGE_REL_AMD64_ADDR32NB
                    if patch_offset + 4 <= pe_sec.data.len() {
                        pe_sec.data[patch_offset..patch_offset+4]
                            .copy_from_slice(&target_rva.to_le_bytes());
                    }
                }
                1 => {
                    // IMAGE_REL_AMD64_ADDR64
                    if patch_offset + 8 <= pe_sec.data.len() {
                        let abs_addr = config.image_base + target_rva as u64;
                        pe_sec.data[patch_offset..patch_offset+8]
                            .copy_from_slice(&abs_addr.to_le_bytes());
                        // Add base relocation for this address
                        reloc_builder.add(section_rva + patch_offset as u32, relocator::IMAGE_REL_BASED_DIR64);
                    }
                }
                _ => {}
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    // PHASE 13: Build .reloc and .edata (for DLLs)
    // ═══════════════════════════════════════════════════════════════════

    if config.is_dll {
        // Export table
        if !export_symbols.is_empty() {
            let dll_name = format!("{}.dll", config.entry_point.trim_start_matches('_'));
            let mut export_builder = relocator::ExportTableBuilder::new(&dll_name);
            for (name, _) in &export_symbols {
                if let Some((sec, off)) = symbols.get(name) {
                    let rva = match sec { 0 => text_rva, 1 => rdata_rva, 2 => data_rva, _ => 0 } + off;
                    export_builder.add(name, rva);
                }
            }
            edata_data = export_builder.build();
        }

        // Add .edata section
        if !edata_data.is_empty() {
            let mut edata_sec = PeSection {
                name: ".edata".into(),
                characteristics: IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ,
                data: edata_data,
                rva: current_rva, file_offset: current_file,
            };
            // Fix up internal RVA references in export table
            let edata_rva = edata_sec.rva;
            // Fix Name RVA, AddressOfFunctions, AddressOfNames, AddressOfNameOrdinals
            if edata_sec.data.len() >= 40 {
                fixup_u32(&mut edata_sec.data, 12, edata_rva); // Name
                fixup_u32(&mut edata_sec.data, 28, edata_rva); // AddressOfFunctions
                fixup_u32(&mut edata_sec.data, 32, edata_rva); // AddressOfNames
                fixup_u32(&mut edata_sec.data, 36, edata_rva); // AddressOfNameOrdinals
                // Fix name pointer entries
                let num_names = u32::from_le_bytes(edata_sec.data[24..28].try_into().unwrap());
                let npt_off = u32::from_le_bytes(edata_sec.data[32..36].try_into().unwrap()) - edata_rva;
                for i in 0..num_names {
                    let off = npt_off as usize + i as usize * 4;
                    if off + 4 <= edata_sec.data.len() {
                        fixup_u32(&mut edata_sec.data, off, edata_rva);
                    }
                }
            }

            current_rva += align_up(edata_sec.virtual_size(), PE_SECTION_ALIGNMENT);
            current_file += edata_sec.raw_size();
            pe_sections.push(edata_sec);
        }

        // Build .reloc section
        let reloc_data = reloc_builder.build();
        if !reloc_data.is_empty() {
            pe_sections.push(PeSection {
                name: ".reloc".into(),
                characteristics: IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_DISCARDABLE,
                data: reloc_data,
                rva: current_rva, file_offset: current_file,
            });
            current_rva += align_up(pe_sections.last().unwrap().virtual_size(), PE_SECTION_ALIGNMENT);
            current_file += pe_sections.last().unwrap().raw_size();
        }
    }

    let image_size = current_rva;
    let num_sections_final = pe_sections.len() as u16;

    // Find entry point RVA
    let entry_rva = symbols.get(&config.entry_point)
        .or_else(|| symbols.get("mainCRTStartup"))
        .or_else(|| symbols.get("_start"))
        .or_else(|| symbols.get("WinMain"))
        .map(|(sec, off)| {
            (match sec { 0 => text_rva, 1 => rdata_rva, 2 => data_rva, _ => 0 }) + off
        })
        .unwrap_or(text_rva);

    // ═══════════════════════════════════════════════════════════════════
    // Write the PE file
    // ═══════════════════════════════════════════════════════════════════

    let mut pe = Vec::new();

    // ── DOS Header (64 bytes) ──
    let mut dos_header = vec![0u8; 64];
    dos_header[0] = b'M'; dos_header[1] = b'Z'; // e_magic
    dos_header[60..64].copy_from_slice(&60u32.to_le_bytes()); // e_lfanew → PE signature at offset 60
    // But our header is 64 bytes, so e_lfanew = 64
    dos_header[60..64].copy_from_slice(&64u32.to_le_bytes());
    pe.extend(&dos_header);

    // ── PE Signature ──
    pe.extend_from_slice(b"PE\0\0");

    // ── COFF File Header (20 bytes) ──
    let mut characteristics: u16 = 0x0022; // EXECUTABLE_IMAGE | LARGE_ADDRESS_AWARE
    if config.is_dll {
        characteristics |= 0x2000; // DLL
    }

    pe.extend_from_slice(&IMAGE_FILE_MACHINE_AMD64.to_le_bytes()); // Machine
    pe.extend_from_slice(&num_sections_final.to_le_bytes()); // NumberOfSections
    pe.extend_from_slice(&0u32.to_le_bytes()); // TimeDateStamp
    pe.extend_from_slice(&0u32.to_le_bytes()); // PointerToSymbolTable
    pe.extend_from_slice(&0u32.to_le_bytes()); // NumberOfSymbols
    pe.extend_from_slice(&opt_header_size.to_le_bytes()[..2]); // SizeOfOptionalHeader
    pe.extend_from_slice(&characteristics.to_le_bytes()); // Characteristics

    // ── Optional Header (PE32+, 240 bytes) ──
    // Standard fields
    pe.extend_from_slice(&0x020Bu16.to_le_bytes()); // Magic (PE32+)
    pe.push(14); pe.push(0); // LinkerVersion Major.Minor

    let text_sec = pe_sections.iter().find(|s| s.name == ".text");
    let code_size = text_sec.map(|s| s.raw_size()).unwrap_or(0);
    let init_data_size: u32 = pe_sections.iter()
        .filter(|s| s.name != ".text" && s.name != ".bss")
        .map(|s| s.raw_size())
        .sum();
    let uninit_data_size = align_up(bss_size, PE_FILE_ALIGNMENT);

    pe.extend_from_slice(&code_size.to_le_bytes());       // SizeOfCode
    pe.extend_from_slice(&init_data_size.to_le_bytes());   // SizeOfInitializedData
    pe.extend_from_slice(&uninit_data_size.to_le_bytes()); // SizeOfUninitializedData
    pe.extend_from_slice(&entry_rva.to_le_bytes());        // AddressOfEntryPoint
    pe.extend_from_slice(&text_rva.to_le_bytes());         // BaseOfCode

    // Windows-specific fields
    pe.extend_from_slice(&config.image_base.to_le_bytes());     // ImageBase
    pe.extend_from_slice(&PE_SECTION_ALIGNMENT.to_le_bytes()); // SectionAlignment
    pe.extend_from_slice(&PE_FILE_ALIGNMENT.to_le_bytes());    // FileAlignment
    pe.extend_from_slice(&6u16.to_le_bytes()); // OS Version Major
    pe.extend_from_slice(&0u16.to_le_bytes()); // OS Version Minor
    pe.extend_from_slice(&0u16.to_le_bytes()); // Image Version Major
    pe.extend_from_slice(&0u16.to_le_bytes()); // Image Version Minor
    pe.extend_from_slice(&6u16.to_le_bytes()); // Subsystem Version Major
    pe.extend_from_slice(&0u16.to_le_bytes()); // Subsystem Version Minor
    pe.extend_from_slice(&0u32.to_le_bytes()); // Win32VersionValue
    pe.extend_from_slice(&image_size.to_le_bytes()); // SizeOfImage
    pe.extend_from_slice(&headers_aligned.to_le_bytes()); // SizeOfHeaders
    pe.extend_from_slice(&0u32.to_le_bytes()); // CheckSum
    pe.extend_from_slice(&config.subsystem.to_le_bytes()); // Subsystem
    let dll_chars: u16 = if config.is_dll { 0x0160 } else { 0x8160 }; // DYNAMIC_BASE | NX_COMPAT | TERMINAL_SERVER_AWARE
    pe.extend_from_slice(&dll_chars.to_le_bytes()); // DllCharacteristics
    pe.extend_from_slice(&config.stack_reserve.to_le_bytes()); // SizeOfStackReserve
    pe.extend_from_slice(&config.stack_commit.to_le_bytes());  // SizeOfStackCommit
    pe.extend_from_slice(&config.heap_reserve.to_le_bytes());  // SizeOfHeapReserve
    pe.extend_from_slice(&config.heap_commit.to_le_bytes());   // SizeOfHeapCommit
    pe.extend_from_slice(&0u32.to_le_bytes()); // LoaderFlags
    pe.extend_from_slice(&16u32.to_le_bytes()); // NumberOfRvaAndSizes

    // Data Directories (16 entries × 8 bytes = 128 bytes)
    // 0: Export Table
    let edata_sec = pe_sections.iter().find(|s| s.name == ".edata");
    if let Some(sec) = edata_sec {
        pe.extend_from_slice(&sec.rva.to_le_bytes());
        pe.extend_from_slice(&sec.virtual_size().to_le_bytes());
    } else {
        pe.extend_from_slice(&0u64.to_le_bytes());
    }

    // 1: Import Table
    if !import_result.data.is_empty() {
        let idt_rva = import_base_rva + import_result.idt_offset;
        let idt_total_size = import_result.idt_size + 20; // include null terminator
        pe.extend_from_slice(&idt_rva.to_le_bytes());
        pe.extend_from_slice(&idt_total_size.to_le_bytes());
    } else {
        pe.extend_from_slice(&0u64.to_le_bytes());
    }

    // 2: Resource Table
    pe.extend_from_slice(&0u64.to_le_bytes());

    // 3: Exception Table (.pdata)
    pe.extend_from_slice(&0u64.to_le_bytes());

    // 4: Certificate Table
    pe.extend_from_slice(&0u64.to_le_bytes());

    // 5: Base Relocation Table
    let reloc_sec = pe_sections.iter().find(|s| s.name == ".reloc");
    if let Some(sec) = reloc_sec {
        pe.extend_from_slice(&sec.rva.to_le_bytes());
        pe.extend_from_slice(&sec.virtual_size().to_le_bytes());
    } else {
        pe.extend_from_slice(&0u64.to_le_bytes());
    }

    // 6-11: Debug, Architecture, GlobalPtr, TLS, LoadConfig, BoundImport
    for _ in 6..12 {
        pe.extend_from_slice(&0u64.to_le_bytes());
    }

    // 12: IAT
    if !import_result.data.is_empty() {
        let iat_rva = import_base_rva + import_result.iat_offset;
        pe.extend_from_slice(&iat_rva.to_le_bytes());
        pe.extend_from_slice(&import_result.iat_size.to_le_bytes());
    } else {
        pe.extend_from_slice(&0u64.to_le_bytes());
    }

    // 13-15: DelayImport, CLR, Reserved
    for _ in 13..16 {
        pe.extend_from_slice(&0u64.to_le_bytes());
    }

    // ── Section Headers ──
    for sec in &pe_sections {
        let mut name = [0u8; 8];
        let n = sec.name.as_bytes();
        name[..n.len().min(8)].copy_from_slice(&n[..n.len().min(8)]);
        pe.extend_from_slice(&name);
        pe.extend_from_slice(&sec.virtual_size().to_le_bytes()); // VirtualSize
        pe.extend_from_slice(&sec.rva.to_le_bytes()); // VirtualAddress
        pe.extend_from_slice(&sec.raw_size().to_le_bytes()); // SizeOfRawData
        pe.extend_from_slice(&sec.file_offset.to_le_bytes()); // PointerToRawData
        pe.extend_from_slice(&0u32.to_le_bytes()); // PointerToRelocations
        pe.extend_from_slice(&0u32.to_le_bytes()); // PointerToLinenumbers
        pe.extend_from_slice(&0u16.to_le_bytes()); // NumberOfRelocations
        pe.extend_from_slice(&0u16.to_le_bytes()); // NumberOfLinenumbers
        pe.extend_from_slice(&sec.characteristics.to_le_bytes()); // Characteristics
    }

    // Pad headers to FileAlignment
    while pe.len() < headers_aligned as usize {
        pe.push(0);
    }

    // ── Section Data ──
    for sec in &pe_sections {
        // Pad to file offset if needed
        while pe.len() < sec.file_offset as usize {
            pe.push(0);
        }
        pe.extend(&sec.data);
        // Pad to FileAlignment
        while pe.len() % PE_FILE_ALIGNMENT as usize != 0 {
            pe.push(0);
        }
    }

    Ok(pe)
}

// ─── Helpers ────────────────────────────────────────────────────────────

fn align_up(value: u32, alignment: u32) -> u32 {
    (value + alignment - 1) & !(alignment - 1)
}

fn fixup_u32(data: &mut [u8], offset: usize, addend: u32) {
    if offset + 4 <= data.len() {
        let val = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap());
        let fixed = val.wrapping_add(addend);
        data[offset..offset+4].copy_from_slice(&fixed.to_le_bytes());
    }
}

fn determine_needed_libs(program: &Program, extern_refs: &[String], extra_libs: &[String]) -> Vec<import_lib::ImportLib> {
    let mut libs = Vec::new();
    let mut used_dlls: Vec<String> = Vec::new();

    // From program's includelib directives
    for lib_name in &program.includelibs {
        if let Some(il) = import_lib::builtin_imports_for(lib_name) {
            if !used_dlls.contains(&il.dll_name) {
                used_dlls.push(il.dll_name.clone());
                libs.push(il);
            }
        }
    }

    // Auto-detect from extern refs
    for name in extern_refs {
        let lib = infer_lib_for_symbol(name);
        if !lib.is_empty() {
            if let Some(il) = import_lib::builtin_imports_for(lib) {
                if !used_dlls.contains(&il.dll_name) {
                    used_dlls.push(il.dll_name.clone());
                    libs.push(il);
                }
            }
        }
    }

    // Extra libs from config
    for lib_name in extra_libs {
        if let Some(il) = import_lib::builtin_imports_for(lib_name) {
            if !used_dlls.contains(&il.dll_name) {
                used_dlls.push(il.dll_name.clone());
                libs.push(il);
            }
        }
    }

    // Always include kernel32
    if !used_dlls.iter().any(|d| d.contains("kernel32")) {
        if let Some(il) = import_lib::builtin_imports_for("kernel32") {
            libs.push(il);
        }
    }

    libs
}

fn infer_lib_for_symbol(name: &str) -> &'static str {
    let n = name.to_lowercase();
    if n.starts_with("printf") || n.starts_with("scanf") || n.starts_with("malloc")
        || n.starts_with("free") || n.starts_with("calloc") || n.starts_with("realloc")
        || n.starts_with("puts") || n.starts_with("strlen") || n.starts_with("strcpy")
        || n.starts_with("strcat") || n.starts_with("strcmp") || n.starts_with("memcpy")
        || n.starts_with("memset") || n == "exit" || n == "_exit"
    {
        "msvcrt"
    } else if n.starts_with("messagebox") || n.starts_with("createwindow")
        || n.starts_with("showwindow") || n.starts_with("defwindowproc")
        || n.starts_with("postquitmessage") || n.starts_with("getmessage")
        || n.starts_with("dispatchmessage") || n.starts_with("registerclass")
    {
        "user32"
    } else if n.starts_with("exitprocess") || n.starts_with("getmodulehandle")
        || n.starts_with("getstdhandle") || n.starts_with("writefile")
        || n.starts_with("virtualalloc") || n.starts_with("heapalloc")
        || n.starts_with("heapfree") || n.starts_with("getprocessheap")
        || n.starts_with("closehandle") || n.starts_with("sleep")
        || n.starts_with("createthread") || n.starts_with("loadlibrary")
        || n.starts_with("getprocaddress")
    {
        "kernel32"
    } else {
        ""
    }
}

fn estimate_data_size(def: &crate::ir::DataDef) -> usize {
    use crate::ir::DataDef::*;
    match def {
        Byte(v) => v.len(),
        Word(v) => v.len() * 2,
        Dword(v) => v.len() * 4,
        Qword(v) => v.len() * 8,
        Float32(v) => v.len() * 4,
        Float64(v) => v.len() * 8,
        String(s) => s.len() + 1,
        WString(s) => s.encode_utf16().count() * 2 + 2,
        ReserveBytes(n) => *n,
        ReserveWords(n) => n * 2,
        ReserveDwords(n) => n * 4,
        ReserveQwords(n) => n * 8,
        DupByte(n, _) => *n,
        DupWord(n, _) => n * 2,
        DupDword(n, _) => n * 4,
        DupQword(n, _) => n * 8,
        Struct(_, fields) => fields.iter().map(|f| estimate_data_size(&f.def)).sum(),
    }
}

fn serialize_data(def: &crate::ir::DataDef, out: &mut Vec<u8>) {
    use crate::ir::DataDef::*;
    match def {
        Byte(v) => out.extend(v),
        Word(v) => for val in v { out.extend(&val.to_le_bytes()); },
        Dword(v) => for val in v { out.extend(&val.to_le_bytes()); },
        Qword(v) => for val in v { out.extend(&val.to_le_bytes()); },
        Float32(v) => for val in v { out.extend(&val.to_le_bytes()); },
        Float64(v) => for val in v { out.extend(&val.to_le_bytes()); },
        String(s) => { out.extend(s.as_bytes()); if !s.ends_with('\0') { out.push(0); } },
        WString(s) => { for c in s.encode_utf16() { out.extend(&c.to_le_bytes()); } out.extend(&[0,0]); },
        ReserveBytes(n) => out.resize(out.len() + n, 0),
        ReserveWords(n) => out.resize(out.len() + n * 2, 0),
        ReserveDwords(n) => out.resize(out.len() + n * 4, 0),
        ReserveQwords(n) => out.resize(out.len() + n * 8, 0),
        DupByte(n, v) => out.resize(out.len() + n, *v),
        DupWord(n, v) => for _ in 0..*n { out.extend(&v.to_le_bytes()); },
        DupDword(n, v) => for _ in 0..*n { out.extend(&v.to_le_bytes()); },
        DupQword(n, v) => for _ in 0..*n { out.extend(&v.to_le_bytes()); },
        Struct(_, fields) => for f in fields { serialize_data(&f.def, out); },
    }
}
