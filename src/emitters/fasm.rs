use crate::ir::*;
use crate::emitters::{Emitter, OutputFormat};

pub struct FasmEmitter;

impl FasmEmitter {
    fn emit_format_directive(&self, program: &Program) -> String {
        match program.format.to_lowercase().as_str() {
            "elf64" | "elf" => "format ELF64",
            "elf32" => "format ELF",
            "pe64" | "pe" | "win64" => "format PE64",
            "pe32" | "win32" => "format PE",
            "bin" => "format binary",
            _ => "format ELF64",
        }
        .to_string()
    }

    fn emit_section_header(&self, kind: &SectionKind) -> String {
        match kind {
            SectionKind::Text => "section '.text' code readable executable".into(),
            SectionKind::Data => "section '.data' data readable writeable".into(),
            SectionKind::Bss => "section '.bss' readable writeable".into(),
            SectionKind::Custom(name) => format!("section '{}' readable", name),
        }
    }

    fn emit_operand(&self, op: &Operand) -> String {
        match op {
            Operand::Reg(reg) => reg.name(),
            Operand::Imm(val) => val.to_string(),
            Operand::Label(name) => name.clone(),
            Operand::Memory { base, index, scale, disp } => {
                let mut parts = Vec::new();
                if let Some(b) = base {
                    parts.push(b.name());
                }
                if let Some(idx) = index {
                    if *scale > 1 {
                        parts.push(format!("{}*{}", idx.name(), scale));
                    } else {
                        parts.push(idx.name());
                    }
                }
                if *disp != 0 {
                    if *disp > 0 && !parts.is_empty() {
                        parts.push(format!("{}", disp));
                    } else if *disp < 0 {
                        return format!("[{} - {}]", parts.join(" + "), -disp);
                    } else {
                        parts.push(format!("{}", disp));
                    }
                }
                if parts.is_empty() {
                    "[0]".into()
                } else {
                    format!("[{}]", parts.join(" + "))
                }
            }
            Operand::StringLit(s) => format!("\"{}\"", s),
        }
    }

    fn emit_instruction(&self, instr: &Instruction) -> String {
        let mnemonic = instr.opcode.name();
        if instr.operands.is_empty() {
            return format!("    {}", mnemonic);
        }
        let ops: Vec<String> = instr.operands.iter().map(|o| self.emit_operand(o)).collect();
        format!("    {} {}", mnemonic, ops.join(", "))
    }

    fn emit_data_item(&self, item: &DataItem) -> String {
        let value = match &item.def {
            DataDef::Byte(bytes) => {
                let vals: Vec<String> = bytes.iter().map(|b| format!("0x{:02X}", b)).collect();
                format!("db {}", vals.join(", "))
            }
            DataDef::Word(words) => {
                let vals: Vec<String> = words.iter().map(|w| format!("0x{:04X}", w)).collect();
                format!("dw {}", vals.join(", "))
            }
            DataDef::Dword(dwords) => {
                let vals: Vec<String> = dwords.iter().map(|d| format!("0x{:08X}", d)).collect();
                format!("dd {}", vals.join(", "))
            }
            DataDef::Qword(qwords) => {
                let vals: Vec<String> = qwords.iter().map(|q| format!("0x{:016X}", q)).collect();
                format!("dq {}", vals.join(", "))
            }
            DataDef::String(s) => format!("db \"{}\", 0", s),
            DataDef::WString(s) => {
                let units: Vec<String> = s.encode_utf16().map(|u| format!("0x{:04X}", u)).collect();
                format!("dw {}, 0", units.join(", "))
            }
            DataDef::ReserveBytes(n) => format!("rb {}", n),
            DataDef::ReserveWords(n) => format!("rw {}", n),
            DataDef::ReserveDwords(n) => format!("rd {}", n),
            DataDef::ReserveQwords(n) => format!("rq {}", n),
        };
        format!("    {} {}", item.name, value)
    }
}

impl Emitter for FasmEmitter {
    fn emit(&self, program: &Program) -> String {
        let mut out = String::new();

        out.push_str(&self.emit_format_directive(program));
        out.push('\n');

        if let Some(org) = program.org {
            out.push_str(&format!("org 0x{:X}\n", org));
        }

        out.push('\n');

        for section in &program.sections {
            out.push_str(&self.emit_section_header(&section.kind));
            out.push('\n');
            out.push('\n');

            for func in &section.functions {
                if func.exported {
                    out.push_str(&format!("public {}\n", func.name));
                }
                out.push_str(&format!("{}:\n", func.name));

                for item in &func.instructions {
                    match item {
                        FunctionItem::Instruction(instr) => {
                            out.push_str(&self.emit_instruction(instr));
                            out.push('\n');
                        }
                        FunctionItem::Label(label) => {
                            out.push_str(&format!(".{}:\n", label));
                        }
                    }
                }

                out.push('\n');
            }

            for item in &section.data {
                out.push_str(&self.emit_data_item(item));
                out.push('\n');
            }

            if !section.data.is_empty() {
                out.push('\n');
            }
        }

        out
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Fasm
    }
}
