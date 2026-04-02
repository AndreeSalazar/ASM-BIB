use crate::ir::*;
use crate::emitters::{Emitter, OutputFormat};

pub struct NasmEmitter;

// ─── Operand formatting ────────────────────────────────────────────────

fn emit_operand(op: &Operand) -> String {
    match op {
        Operand::Reg(r) => r.name(),
        Operand::Imm(v) => format_imm(*v),
        Operand::Label(name) => name.clone(),
        Operand::Memory { base, index, scale, disp } => {
            emit_memory(base.as_ref(), index.as_ref(), *scale, *disp)
        }
        Operand::StringLit(s) => emit_string_operand(s),
    }
}

fn format_imm(v: i64) -> String {
    if v < 0 {
        format!("-0x{:X}", v.unsigned_abs())
    } else if v > 9 {
        format!("0x{:X}", v)
    } else {
        v.to_string()
    }
}

fn emit_memory(base: Option<&Register>, index: Option<&Register>, scale: u8, disp: i64) -> String {
    let mut parts = Vec::new();

    if let Some(b) = base {
        parts.push(b.name());
    }

    if let Some(idx) = index {
        if scale > 1 {
            parts.push(format!("{}*{}", idx.name(), scale));
        } else {
            parts.push(idx.name());
        }
    }

    if disp != 0 || parts.is_empty() {
        if disp < 0 && !parts.is_empty() {
            return format!("[{} - 0x{:X}]", parts.join(" + "), disp.unsigned_abs());
        }
        parts.push(format_imm(disp));
    }

    format!("[{}]", parts.join(" + "))
}

fn emit_string_operand(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\")
        .replace('\n', "\", 10, \"")
        .replace('\r', "\", 13, \"")
        .replace('\t', "\", 9, \"")
        .replace('\0', "\", 0, \"");
    let result = format!("\"{}\"", escaped);
    result.replace("\"\", ", "").replace(", \"\"", "")
}

// ─── Section names ─────────────────────────────────────────────────────

fn emit_section_name(kind: &SectionKind) -> &'static str {
    match kind {
        SectionKind::Text => "section .text",
        SectionKind::Data => "section .data",
        SectionKind::Bss  => "section .bss",
        SectionKind::Rodata => "section .rodata",
        SectionKind::Custom(_) => "section .text",
    }
}

// ─── Data definitions ──────────────────────────────────────────────────

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
            let mut units: Vec<String> = s.encode_utf16()
                .map(|u| format!("0x{:04X}", u))
                .collect();
            units.push("0x0000".into());
            format!("{}: dw {}", name, units.join(", "))
        }
        DataDef::ReserveBytes(n)  => format!("{}: resb {}", name, n),
        DataDef::ReserveWords(n)  => format!("{}: resw {}", name, n),
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
            let mut lines = vec![format!("; {} (struct {})", name, struct_name)];
            for field in fields {
                lines.push(emit_data_def(field));
            }
            lines.join("\n")
        }
    }
}

// ─── Extern declarations ───────────────────────────────────────────────

fn collect_extern_symbols(program: &Program) -> Vec<String> {
    let defined: Vec<&str> = program.sections.iter()
        .flat_map(|s| s.functions.iter())
        .map(|f| f.name.as_str())
        .collect();

    let mut externs = Vec::new();

    for ext in &program.externs {
        if !externs.contains(&ext.name) {
            externs.push(ext.name.clone());
        }
    }

    for sec in &program.sections {
        for func in &sec.functions {
            for item in &func.instructions {
                if let FunctionItem::Instruction(instr) = item {
                    if matches!(instr.opcode, Opcode::Call) {
                        for op in &instr.operands {
                            if let Operand::Label(name) = op {
                                if !defined.contains(&name.as_str()) && !externs.contains(name) {
                                    externs.push(name.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    externs
}

// ─── Main emitter ──────────────────────────────────────────────────────

impl Emitter for NasmEmitter {
    fn format(&self) -> OutputFormat {
        OutputFormat::Nasm
    }

    fn emit(&self, program: &Program) -> String {
        let mut out = String::with_capacity(4096);

        out.push_str("; Generated by ASM-BIB\n");

        match program.arch {
            Arch::X86_16 => out.push_str("bits 16\n"),
            Arch::X86_32 => out.push_str("bits 32\n"),
            Arch::X86_64 => out.push_str("bits 64\n"),
        }

        if let Some(org) = program.org {
            out.push_str(&format!("org 0x{:X}\n", org));
        }

        out.push('\n');

        // extern declarations
        let externs = collect_extern_symbols(program);
        for ext in &externs {
            out.push_str(&format!("extern {}\n", ext));
        }
        if !externs.is_empty() {
            out.push('\n');
        }

        // global declarations
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

        // sections
        for section in &program.sections {
            out.push_str(emit_section_name(&section.kind));
            out.push('\n');

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
                                    .map(emit_operand)
                                    .collect();
                                out.push_str(&format!("    {} {}\n", mnemonic, ops.join(", ")));
                            }
                        }
                    }
                }
                out.push('\n');
            }

            for item in &section.data {
                out.push_str(&format!("    {}\n", emit_data_def(item)));
            }

            if !section.data.is_empty() {
                out.push('\n');
            }
        }

        out
    }
}
