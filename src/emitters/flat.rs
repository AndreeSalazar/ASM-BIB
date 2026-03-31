use crate::ir::*;
use crate::emitters::{Emitter, OutputFormat};

pub struct FlatEmitter;

impl Emitter for FlatEmitter {
    fn format(&self) -> OutputFormat {
        OutputFormat::Flat
    }

    fn emit(&self, program: &Program) -> String {
        let mut out = String::new();

        // BITS directive
        let bits = match program.arch {
            Arch::X86_16 => 16,
            Arch::X86_32 => 32,
            Arch::X86_64 => 64,
            _ => 64,
        };
        out.push_str(&format!("[BITS {}]\n", bits));

        // ORG directive
        if let Some(addr) = program.org {
            out.push_str(&format!("[ORG 0x{:X}]\n", addr));
        }

        out.push('\n');

        for section in &program.sections {
            // Emit functions / code
            for func in &section.functions {
                if func.exported {
                    out.push_str(&format!("global {}\n", func.name));
                }
                out.push_str(&format!("{}:\n", func.name));
                for item in &func.instructions {
                    match item {
                        FunctionItem::Label(label) => {
                            out.push_str(&format!("{}:\n", label));
                        }
                        FunctionItem::Comment(text) => {
                            out.push_str(&format!("    ; {}\n", text));
                        }
                        FunctionItem::Instruction(instr) => {
                            out.push_str(&format!("    {}\n", format_instruction(instr)));
                        }
                    }
                }
            }

            // Emit data definitions
            for item in &section.data {
                out.push_str(&format!("{}: ", item.name));
                out.push_str(&format_data(&item.def));
                out.push('\n');
            }
        }

        out
    }
}

fn format_instruction(instr: &Instruction) -> String {
    let mnemonic = instr.opcode.name();
    if instr.operands.is_empty() {
        return mnemonic.to_string();
    }
    let ops: Vec<String> = instr.operands.iter().map(format_operand).collect();
    format!("{} {}", mnemonic, ops.join(", "))
}

fn format_operand(op: &Operand) -> String {
    match op {
        Operand::Reg(reg) => reg.name(),
        Operand::Imm(val) => {
            if *val < 0 {
                format!("-0x{:X}", -val)
            } else {
                format!("0x{:X}", val)
            }
        }
        Operand::Label(name) => name.clone(),
        Operand::Memory { base, index, scale, disp } => {
            let mut inner = String::new();
            if let Some(b) = base {
                inner.push_str(&b.name());
            }
            if let Some(idx) = index {
                if !inner.is_empty() {
                    inner.push_str(" + ");
                }
                inner.push_str(&idx.name());
                if *scale > 1 {
                    inner.push_str(&format!("*{}", scale));
                }
            }
            if *disp != 0 {
                if !inner.is_empty() {
                    if *disp > 0 {
                        inner.push_str(&format!(" + 0x{:X}", disp));
                    } else {
                        inner.push_str(&format!(" - 0x{:X}", -disp));
                    }
                } else {
                    inner.push_str(&format!("0x{:X}", disp));
                }
            }
            if inner.is_empty() {
                inner.push('0');
            }
            format!("[{}]", inner)
        }
        Operand::StringLit(s) => format!("\"{}\"", s),
    }
}

fn format_data(def: &DataDef) -> String {
    match def {
        DataDef::Byte(vals) => {
            let items: Vec<String> = vals.iter().map(|v| format!("0x{:02X}", v)).collect();
            format!("db {}", items.join(", "))
        }
        DataDef::Word(vals) => {
            let items: Vec<String> = vals.iter().map(|v| format!("0x{:04X}", v)).collect();
            format!("dw {}", items.join(", "))
        }
        DataDef::Dword(vals) => {
            let items: Vec<String> = vals.iter().map(|v| format!("0x{:08X}", v)).collect();
            format!("dd {}", items.join(", "))
        }
        DataDef::Qword(vals) => {
            let items: Vec<String> = vals.iter().map(|v| format!("0x{:016X}", v)).collect();
            format!("dq {}", items.join(", "))
        }
        DataDef::String(s) => format!("db \"{}\", 0", s),
        DataDef::WString(s) => {
            let units: Vec<String> = s.encode_utf16().map(|u| format!("0x{:04X}", u)).collect();
            format!("dw {}, 0", units.join(", "))
        }
        DataDef::ReserveBytes(n) => format!("resb {}", n),
        DataDef::ReserveWords(n) => format!("resw {}", n),
        DataDef::ReserveDwords(n) => format!("resd {}", n),
        DataDef::ReserveQwords(n) => format!("resq {}", n),
        DataDef::Float32(vals) => {
            let vs: Vec<String> = vals.iter().map(|v| format!("{}", v)).collect();
            format!("dd {}", vs.join(", "))
        }
        DataDef::Float64(vals) => {
            let vs: Vec<String> = vals.iter().map(|v| format!("{}", v)).collect();
            format!("dq {}", vs.join(", "))
        }
        DataDef::Struct(struct_name, fields) => {
            let mut lines = vec![format!("; struct {}", struct_name)];
            for field in fields {
                lines.push(format!("{}: {}", field.name, format_data(&field.def)));
            }
            lines.join("\n")
        }
    }
}
