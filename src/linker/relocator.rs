//! Relocation applier — Phase 10/13
//! Applies COFF relocations to section data after layout is finalized.
//! Also generates base relocation table (.reloc) for DLLs/ASLR.

use super::coff_reader::*;

/// Apply a single relocation to section data
pub fn apply_relocation(
    section_data: &mut [u8],
    reloc: &CoffRelocation,
    symbol_rva: u32,          // Final RVA of the target symbol
    section_rva: u32,         // RVA of the section containing the relocation
    iat_rva: Option<u32>,     // RVA of IAT entry (for __imp_ symbols)
) -> Result<(), String> {
    let patch_offset = reloc.virtual_address as usize;

    match reloc.rel_type {
        IMAGE_REL_AMD64_ABSOLUTE => {
            // No-op
        }
        IMAGE_REL_AMD64_ADDR64 => {
            // 64-bit absolute address
            if patch_offset + 8 > section_data.len() {
                return Err(format!("ADDR64 reloc out of bounds at offset {}", patch_offset));
            }
            let target = iat_rva.unwrap_or(symbol_rva) as u64;
            // Add the existing addend
            let existing = u64::from_le_bytes(section_data[patch_offset..patch_offset+8].try_into().unwrap());
            let final_val = target.wrapping_add(existing);
            section_data[patch_offset..patch_offset+8].copy_from_slice(&final_val.to_le_bytes());
        }
        IMAGE_REL_AMD64_ADDR32 => {
            // 32-bit absolute address
            if patch_offset + 4 > section_data.len() {
                return Err(format!("ADDR32 reloc out of bounds at offset {}", patch_offset));
            }
            let target = iat_rva.unwrap_or(symbol_rva);
            let existing = u32::from_le_bytes(section_data[patch_offset..patch_offset+4].try_into().unwrap());
            let final_val = target.wrapping_add(existing);
            section_data[patch_offset..patch_offset+4].copy_from_slice(&final_val.to_le_bytes());
        }
        IMAGE_REL_AMD64_ADDR32NB => {
            // 32-bit address without base (RVA)
            if patch_offset + 4 > section_data.len() {
                return Err(format!("ADDR32NB reloc out of bounds at offset {}", patch_offset));
            }
            let target = iat_rva.unwrap_or(symbol_rva);
            let existing = u32::from_le_bytes(section_data[patch_offset..patch_offset+4].try_into().unwrap());
            let final_val = target.wrapping_add(existing);
            section_data[patch_offset..patch_offset+4].copy_from_slice(&final_val.to_le_bytes());
        }
        IMAGE_REL_AMD64_REL32 | IMAGE_REL_AMD64_REL32_1 | IMAGE_REL_AMD64_REL32_2 |
        IMAGE_REL_AMD64_REL32_3 | IMAGE_REL_AMD64_REL32_4 | IMAGE_REL_AMD64_REL32_5 => {
            // RIP-relative 32-bit displacement
            if patch_offset + 4 > section_data.len() {
                return Err(format!("REL32 reloc out of bounds at offset {}", patch_offset));
            }
            let addend_extra = match reloc.rel_type {
                IMAGE_REL_AMD64_REL32_1 => 1u32,
                IMAGE_REL_AMD64_REL32_2 => 2,
                IMAGE_REL_AMD64_REL32_3 => 3,
                IMAGE_REL_AMD64_REL32_4 => 4,
                IMAGE_REL_AMD64_REL32_5 => 5,
                _ => 0,
            };
            let target = iat_rva.unwrap_or(symbol_rva);
            let rip = section_rva + patch_offset as u32 + 4 + addend_extra;
            let existing = i32::from_le_bytes(section_data[patch_offset..patch_offset+4].try_into().unwrap());
            let delta = (target as i64) - (rip as i64) + (existing as i64);
            section_data[patch_offset..patch_offset+4].copy_from_slice(&(delta as i32).to_le_bytes());
        }
        other => {
            return Err(format!("Unsupported relocation type: 0x{:04X}", other));
        }
    }

    Ok(())
}

/// Generate .reloc section (base relocation table) for DLLs/ASLR — Phase 13
/// Scans all sections for absolute address relocations that need fixups at load time.
pub struct BaseRelocationBuilder {
    /// Entries: (rva, type)  — type is IMAGE_REL_BASED_DIR64 (10) or HIGHLOW (3)
    entries: Vec<(u32, u8)>,
}

pub const IMAGE_REL_BASED_ABSOLUTE: u8 = 0;
pub const IMAGE_REL_BASED_HIGHLOW: u8  = 3;
pub const IMAGE_REL_BASED_DIR64: u8    = 10;

impl BaseRelocationBuilder {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn add(&mut self, rva: u32, rel_type: u8) {
        self.entries.push((rva, rel_type));
    }

    /// Build the .reloc section data
    /// Format: blocks of (PageRVA: u32, BlockSize: u32, entries: [u16]...)
    /// Each entry: (type:4 bits << 12) | (offset:12 bits)
    pub fn build(&self) -> Vec<u8> {
        if self.entries.is_empty() {
            return Vec::new();
        }

        let mut sorted = self.entries.clone();
        sorted.sort_by_key(|e| e.0);

        let mut result = Vec::new();
        let mut i = 0;

        while i < sorted.len() {
            let page_rva = sorted[i].0 & !0xFFF; // Round down to 4K page
            let block_start = result.len();

            // Placeholder for PageRVA and BlockSize
            result.extend_from_slice(&page_rva.to_le_bytes());
            result.extend_from_slice(&0u32.to_le_bytes()); // BlockSize placeholder

            while i < sorted.len() && (sorted[i].0 & !0xFFF) == page_rva {
                let offset = sorted[i].0 & 0xFFF;
                let entry = ((sorted[i].1 as u16) << 12) | (offset as u16);
                result.extend_from_slice(&entry.to_le_bytes());
                i += 1;
            }

            // Pad to 4-byte alignment
            if (result.len() - block_start) % 4 != 0 {
                result.extend_from_slice(&0u16.to_le_bytes());
            }

            // Write block size
            let block_size = (result.len() - block_start) as u32;
            result[block_start+4..block_start+8].copy_from_slice(&block_size.to_le_bytes());
        }

        result
    }
}

/// Generate export table (.edata) — Phase 13
pub struct ExportTableBuilder {
    pub dll_name: String,
    pub exports: Vec<(String, u32)>, // (name, rva)
}

impl ExportTableBuilder {
    pub fn new(dll_name: &str) -> Self {
        Self {
            dll_name: dll_name.into(),
            exports: Vec::new(),
        }
    }

    pub fn add(&mut self, name: &str, rva: u32) {
        self.exports.push((name.into(), rva));
    }

    /// Build .edata section
    /// Returns (section_data, name_rvas_to_fixup)
    pub fn build(&self) -> Vec<u8> {
        if self.exports.is_empty() {
            return Vec::new();
        }

        let mut sorted = self.exports.clone();
        sorted.sort_by(|a, b| a.0.cmp(&b.0));

        let num_funcs = sorted.len() as u32;

        // Layout:
        // [Export Directory Table (40 bytes)]
        // [Address Table: num_funcs * 4 bytes]
        // [Name Pointer Table: num_funcs * 4 bytes]
        // [Ordinal Table: num_funcs * 2 bytes]
        // [DLL name string]
        // [Function name strings...]

        let dir_size = 40u32;
        let addr_table_off = dir_size;
        let name_ptr_off = addr_table_off + num_funcs * 4;
        let ordinal_off = name_ptr_off + num_funcs * 4;
        let strings_off = ordinal_off + num_funcs * 2;
        // Pad to 4-byte alignment
        let strings_off = (strings_off + 3) & !3;

        let mut data = vec![0u8; strings_off as usize];
        let mut strings_data = Vec::new();

        // DLL name
        let dll_name_rva_offset = strings_off as usize + strings_data.len();
        strings_data.extend_from_slice(self.dll_name.as_bytes());
        strings_data.push(0);

        // Function names
        let mut name_offsets = Vec::new();
        for (name, _) in &sorted {
            name_offsets.push(strings_off as usize + strings_data.len());
            strings_data.extend_from_slice(name.as_bytes());
            strings_data.push(0);
        }

        // Export Directory Table
        // Characteristics
        data[0..4].copy_from_slice(&0u32.to_le_bytes());
        // TimeDateStamp
        data[4..8].copy_from_slice(&0u32.to_le_bytes());
        // MajorVersion, MinorVersion
        data[8..10].copy_from_slice(&0u16.to_le_bytes());
        data[10..12].copy_from_slice(&0u16.to_le_bytes());
        // Name RVA (will be fixed up by caller adding section RVA)
        data[12..16].copy_from_slice(&(dll_name_rva_offset as u32).to_le_bytes());
        // Ordinal Base
        data[16..20].copy_from_slice(&1u32.to_le_bytes());
        // NumberOfFunctions
        data[20..24].copy_from_slice(&num_funcs.to_le_bytes());
        // NumberOfNames
        data[24..28].copy_from_slice(&num_funcs.to_le_bytes());
        // AddressOfFunctions
        data[28..32].copy_from_slice(&addr_table_off.to_le_bytes());
        // AddressOfNames
        data[32..36].copy_from_slice(&name_ptr_off.to_le_bytes());
        // AddressOfNameOrdinals
        data[36..40].copy_from_slice(&ordinal_off.to_le_bytes());

        // Address Table (function RVAs)
        for (i, (_, rva)) in sorted.iter().enumerate() {
            let off = addr_table_off as usize + i * 4;
            data[off..off+4].copy_from_slice(&rva.to_le_bytes());
        }

        // Name Pointer Table (RVAs to name strings)
        for (i, name_off) in name_offsets.iter().enumerate() {
            let off = name_ptr_off as usize + i * 4;
            data[off..off+4].copy_from_slice(&(*name_off as u32).to_le_bytes());
        }

        // Ordinal Table
        for i in 0..num_funcs {
            let off = ordinal_off as usize + i as usize * 2;
            data[off..off+2].copy_from_slice(&(i as u16).to_le_bytes());
        }

        data.extend(strings_data);
        data
    }
}
