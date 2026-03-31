use crate::ir::*;
use crate::emitters::{Emitter, OutputFormat};

pub struct NasmEmitter;

impl NasmEmitter {
    fn emit_operand(op: &Operand) -> String {
        match op {
            Operand::Reg(r) => r.name(),
            Operand::Imm(v) => {
                if *v < 0 {
                    format!("-0x{:X}", v.unsigned_abs())
                } else {
                    format!("0x{:X}", v)
                }
            }
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

                if *disp != 0 || parts.is_empty() {
                    if *disp < 0 {
                        // Negative displacement: remove last "+" and use "-"
                        let abs = disp.unsigned_abs();
                        if parts.is_empty() {
                            parts.push(format!("0x{:X}", abs));
                        } else {
                            return format!("[{} - 0x{:X}]", parts.join(" + "), abs);
                        }
                    } else {
                        parts.push(format!("0x{:X}", disp));
                    }
                }

                format!("[{}]", parts.join(" + "))
            }
            Operand::StringLit(s) => {
                let escaped = s.replace('\\', "\\\\")
                    .replace('\n', "\", 10, \"")
                    .replace('\r', "\", 13, \"")
                    .replace('\t', "\", 9, \"")
                    .replace('\0', "\", 0, \"");
                // Clean up empty string fragments from replacements
                let result = format!("\"{}\"", escaped);
                result.replace("\"\", ", "").replace(", \"\"", "")
            }
        }
    }

    fn emit_section_name(kind: &SectionKind) -> String {
        match kind {
            SectionKind::Text => "section .text".into(),
            SectionKind::Data => "section .data".into(),
            SectionKind::Bss => "section .bss".into(),
            SectionKind::Rodata => "section .rodata".into(),
            SectionKind::Custom(name) => format!("section {}", name),
        }
    }

    fn emit_data_def(item: &DataItem) -> String {
        let name = &item.name;
        match &item.def {
            DataDef::Byte(vals) => {
                let vs: Vec<String> = vals.iter().map(|v| format!("0x{:02X}", v)).collect();
                format!("{}: db {}", name, vs.join(", "))
            }
            DataDef::Word(vals) => {
                let vs: Vec<String> = vals.iter().map(|v| format!("0x{:04X}", v)).collect();
                format!("{}: dw {}", name, vs.join(", "))
            }
            DataDef::Dword(vals) => {
                let vs: Vec<String> = vals.iter().map(|v| format!("0x{:08X}", v)).collect();
                format!("{}: dd {}", name, vs.join(", "))
            }
            DataDef::Qword(vals) => {
                let vs: Vec<String> = vals.iter().map(|v| format!("0x{:016X}", v)).collect();
                format!("{}: dq {}", name, vs.join(", "))
            }
            DataDef::String(s) => {
                let escaped = s.replace('\n', "\", 10, \"")
                    .replace('\r', "\", 13, \"")
                    .replace('\t', "\", 9, \"");
                let lit = format!("\"{}\"", escaped)
                    .replace("\"\", ", "")
                    .replace(", \"\"", "");
                format!("{}: db {}, 0", name, lit)
            }
            DataDef::WString(s) => {
                let units: Vec<String> = s.encode_utf16()
                    .map(|u| format!("0x{:04X}", u))
                    .collect();
                let mut all = units;
                all.push("0x0000".into());
                format!("{}: dw {}", name, all.join(", "))
            }
            DataDef::ReserveBytes(n) => format!("{}: resb {}", name, n),
            DataDef::ReserveWords(n) => format!("{}: resw {}", name, n),
            DataDef::ReserveDwords(n) => format!("{}: resd {}", name, n),
            DataDef::ReserveQwords(n) => format!("{}: resq {}", name, n),
            DataDef::Float32(vals) => {
                let vs: Vec<String> = vals.iter().map(|v| format!("{}", v)).collect();
                format!("{}: dd {}", name, vs.join(", "))
            }
            DataDef::Float64(vals) => {
                let vs: Vec<String> = vals.iter().map(|v| format!("{}", v)).collect();
                format!("{}: dq {}", name, vs.join(", "))
            }
            DataDef::Struct(struct_name, fields) => {
                let mut lines = vec![format!("; {} {} (struct {})", name, struct_name, struct_name)];
                for field in fields {
                    lines.push(Self::emit_data_def(field));
                }
                lines.join("\n")
            }
        }
    }
}

impl Emitter for NasmEmitter {
    fn format(&self) -> OutputFormat {
        OutputFormat::Nasm
    }

    fn emit(&self, program: &Program) -> String {
        let mut out = String::new();

        // Header comment
        out.push_str("; Generated by NASM-BIB\n");

        // Architecture hint
        match program.arch {
            Arch::X86_16 => out.push_str("bits 16\n"),
            Arch::X86_32 => out.push_str("bits 32\n"),
            Arch::X86_64 => out.push_str("bits 64\n"),
            _ => {}
        }

        // Origin
        if let Some(org) = program.org {
            out.push_str(&format!("org 0x{:X}\n", org));
        }

        out.push('\n');

        // Collect all exported function names for `global` directives
        let globals: Vec<&str> = program.sections.iter()
            .flat_map(|s| s.functions.iter())
            .filter(|f| f.exported)
            .map(|f| f.name.as_str())
            .collect();

        if !globals.is_empty() {
            for g in &globals {
                out.push_str(&format!("global {}\n", g));
            }
            out.push('\n');
        }

        // Emit each section
        for section in &program.sections {
            out.push_str(&Self::emit_section_name(&section.kind));
            out.push('\n');

            // Functions / code
            for func in &section.functions {
                out.push_str(&format!("{}:\n", func.name));
                for item in &func.instructions {
                    match item {
                        FunctionItem::Label(lbl) => {
                            out.push_str(&format!(".{}:\n", lbl));
                        }
                        FunctionItem::Comment(text) => {
                            out.push_str(&format!("    ; {}\n", text));
                        }
                        FunctionItem::Instruction(instr) => {
                            let mnemonic = instr.opcode.name();
                            if instr.operands.is_empty() {
                                out.push_str(&format!("    {}\n", mnemonic));
                            } else {
                                let ops: Vec<String> = instr.operands.iter()
                                    .map(Self::emit_operand)
                                    .collect();
                                out.push_str(&format!("    {} {}\n", mnemonic, ops.join(", ")));
                            }
                        }
                    }
                }
                out.push('\n');
            }

            // Data definitions
            for item in &section.data {
                out.push_str(&format!("    {}\n", Self::emit_data_def(item)));
            }

            if !section.data.is_empty() {
                out.push('\n');
            }
        }

        out
    }
}
