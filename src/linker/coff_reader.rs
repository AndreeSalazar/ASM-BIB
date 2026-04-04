//! COFF .obj parser — Phase 10
//! Reads COFF object files and extracts sections, symbols, and relocations.

use std::collections::HashMap;

/// A parsed COFF object file
#[derive(Debug)]
pub struct CoffFile {
    pub machine: u16,
    pub sections: Vec<CoffSection>,
    pub symbols: Vec<CoffSymbol>,
    pub string_table: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct CoffSection {
    pub name: String,
    pub virtual_size: u32,
    pub characteristics: u32,
    pub data: Vec<u8>,
    pub relocations: Vec<CoffRelocation>,
}

#[derive(Debug, Clone)]
pub struct CoffRelocation {
    pub virtual_address: u32,
    pub symbol_index: u32,
    pub rel_type: u16,
}

#[derive(Debug, Clone)]
pub struct CoffSymbol {
    pub name: String,
    pub value: u32,
    pub section_number: i16,
    pub storage_class: u8,
    pub num_aux: u8,
}

// IMAGE_REL_AMD64 relocation types
pub const IMAGE_REL_AMD64_ABSOLUTE: u16 = 0x0000;
pub const IMAGE_REL_AMD64_ADDR64: u16   = 0x0001;
pub const IMAGE_REL_AMD64_ADDR32: u16   = 0x0002;
pub const IMAGE_REL_AMD64_ADDR32NB: u16 = 0x0003;
pub const IMAGE_REL_AMD64_REL32: u16    = 0x0004;
pub const IMAGE_REL_AMD64_REL32_1: u16  = 0x0005;
pub const IMAGE_REL_AMD64_REL32_2: u16  = 0x0006;
pub const IMAGE_REL_AMD64_REL32_3: u16  = 0x0007;
pub const IMAGE_REL_AMD64_REL32_4: u16  = 0x0008;
pub const IMAGE_REL_AMD64_REL32_5: u16  = 0x0009;

impl CoffFile {
    /// Parse a COFF .obj from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self, String> {
        if data.len() < 20 {
            return Err("COFF file too small".into());
        }

        let machine = u16::from_le_bytes([data[0], data[1]]);
        let num_sections = u16::from_le_bytes([data[2], data[3]]) as usize;
        let sym_table_ptr = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;
        let num_symbols = u32::from_le_bytes([data[12], data[13], data[14], data[15]]) as usize;

        // Parse string table (after symbol table)
        let str_table_offset = sym_table_ptr + num_symbols * 18;
        let string_table = if str_table_offset < data.len() {
            let str_size = if str_table_offset + 4 <= data.len() {
                u32::from_le_bytes([
                    data[str_table_offset], data[str_table_offset+1],
                    data[str_table_offset+2], data[str_table_offset+3],
                ]) as usize
            } else { 4 };
            let end = (str_table_offset + str_size).min(data.len());
            data[str_table_offset..end].to_vec()
        } else {
            vec![0, 0, 0, 4] // empty
        };

        // Parse symbols
        let mut symbols = Vec::with_capacity(num_symbols);
        let mut skip_aux = 0u8;
        for i in 0..num_symbols {
            let off = sym_table_ptr + i * 18;
            if off + 18 > data.len() { break; }

            if skip_aux > 0 {
                skip_aux -= 1;
                // Aux symbol record — push a placeholder
                symbols.push(CoffSymbol {
                    name: String::new(),
                    value: 0,
                    section_number: 0,
                    storage_class: 0,
                    num_aux: 0,
                });
                continue;
            }

            let name = read_symbol_name(&data[off..off+8], &string_table);
            let value = u32::from_le_bytes([data[off+8], data[off+9], data[off+10], data[off+11]]);
            let section_number = i16::from_le_bytes([data[off+12], data[off+13]]);
            let _sym_type = u16::from_le_bytes([data[off+14], data[off+15]]);
            let storage_class = data[off+16];
            let num_aux = data[off+17];

            symbols.push(CoffSymbol { name, value, section_number, storage_class, num_aux });
            skip_aux = num_aux;
        }

        // Parse sections
        let mut sections = Vec::with_capacity(num_sections);
        for i in 0..num_sections {
            let sh_off = 20 + i * 40;
            if sh_off + 40 > data.len() { break; }

            let sec_name = read_section_name(&data[sh_off..sh_off+8], &string_table);
            let virtual_size = u32::from_le_bytes([data[sh_off+8], data[sh_off+9], data[sh_off+10], data[sh_off+11]]);
            let raw_size = u32::from_le_bytes([data[sh_off+16], data[sh_off+17], data[sh_off+18], data[sh_off+19]]) as usize;
            let raw_ptr = u32::from_le_bytes([data[sh_off+20], data[sh_off+21], data[sh_off+22], data[sh_off+23]]) as usize;
            let reloc_ptr = u32::from_le_bytes([data[sh_off+24], data[sh_off+25], data[sh_off+26], data[sh_off+27]]) as usize;
            let num_relocs = u16::from_le_bytes([data[sh_off+32], data[sh_off+33]]) as usize;
            let characteristics = u32::from_le_bytes([data[sh_off+36], data[sh_off+37], data[sh_off+38], data[sh_off+39]]);

            // Read raw data
            let sec_data = if raw_ptr > 0 && raw_ptr + raw_size <= data.len() {
                data[raw_ptr..raw_ptr+raw_size].to_vec()
            } else {
                vec![0u8; raw_size]
            };

            // Read relocations
            let mut relocs = Vec::with_capacity(num_relocs);
            for r in 0..num_relocs {
                let roff = reloc_ptr + r * 10;
                if roff + 10 > data.len() { break; }
                relocs.push(CoffRelocation {
                    virtual_address: u32::from_le_bytes([data[roff], data[roff+1], data[roff+2], data[roff+3]]),
                    symbol_index: u32::from_le_bytes([data[roff+4], data[roff+5], data[roff+6], data[roff+7]]),
                    rel_type: u16::from_le_bytes([data[roff+8], data[roff+9]]),
                });
            }

            sections.push(CoffSection { name: sec_name, virtual_size, characteristics, data: sec_data, relocations: relocs });
        }

        Ok(CoffFile { machine, sections, symbols, string_table })
    }

    /// Get a symbol by index
    pub fn symbol_name(&self, idx: u32) -> &str {
        self.symbols.get(idx as usize).map(|s| s.name.as_str()).unwrap_or("")
    }
}

fn read_symbol_name(name_bytes: &[u8], string_table: &[u8]) -> String {
    // If first 4 bytes are zero, the next 4 are an offset into the string table
    if name_bytes[0] == 0 && name_bytes[1] == 0 && name_bytes[2] == 0 && name_bytes[3] == 0 {
        let offset = u32::from_le_bytes([name_bytes[4], name_bytes[5], name_bytes[6], name_bytes[7]]) as usize;
        if offset < string_table.len() {
            let end = string_table[offset..].iter().position(|&b| b == 0).unwrap_or(string_table.len() - offset);
            return String::from_utf8_lossy(&string_table[offset..offset+end]).to_string();
        }
        return String::new();
    }
    // Short name: up to 8 bytes
    let end = name_bytes.iter().position(|&b| b == 0).unwrap_or(8);
    String::from_utf8_lossy(&name_bytes[..end]).to_string()
}

fn read_section_name(name_bytes: &[u8], string_table: &[u8]) -> String {
    // Section names starting with '/' use string table offset
    if name_bytes[0] == b'/' {
        let offset_str = String::from_utf8_lossy(&name_bytes[1..]).trim_end_matches('\0').to_string();
        if let Ok(offset) = offset_str.trim().parse::<usize>() {
            if offset < string_table.len() {
                let end = string_table[offset..].iter().position(|&b| b == 0).unwrap_or(string_table.len() - offset);
                return String::from_utf8_lossy(&string_table[offset..offset+end]).to_string();
            }
        }
    }
    let end = name_bytes.iter().position(|&b| b == 0).unwrap_or(8);
    String::from_utf8_lossy(&name_bytes[..end]).to_string()
}
