use crate::ir::*;
use crate::emitters::{Emitter, OutputFormat};

pub struct MasmEmitter;

impl Emitter for MasmEmitter {
    fn format(&self) -> OutputFormat {
        OutputFormat::Masm
    }

    fn emit(&self, program: &Program) -> String {
        let mut out = String::new();

        for section in &program.sections {
            match &section.kind {
                SectionKind::Text => {
                    out.push_str(".code\n");
                    for func in &section.functions {
                        emit_function(&mut out, func);
                    }
                }
                SectionKind::Data => {
                    out.push_str(".data\n");
                    for item in &section.data {
                        emit_data_item(&mut out, &item.name, &item.def);
                    }
                }
                SectionKind::Bss => {
                    out.push_str(".data?\n");
                    for item in &section.data {
                        emit_data_item(&mut out, &item.name, &item.def);
                    }
                }
                SectionKind::Custom(name) => {
                    out.push_str(&format!("{}\n", name));
                    for func in &section.functions {
                        emit_function(&mut out, func);
                    }
                    for item in &section.data {
                        emit_data_item(&mut out, &item.name, &item.def);
                    }
                }
            }
            out.push('\n');
        }

        out.push_str("END\n");
        out
    }
}

fn emit_function(out: &mut String, func: &Function) {
    if func.exported {
        out.push_str(&format!("PUBLIC {}\n", func.name));
    }
    out.push_str(&format!("{} PROC\n", func.name));
    for item in &func.instructions {
        match item {
            FunctionItem::Label(lbl) => {
                out.push_str(&format!("{}:\n", lbl));
            }
            FunctionItem::Instruction(instr) => {
                out.push_str(&format!("    {}\n", emit_instruction(instr)));
            }
        }
    }
    out.push_str(&format!("{} ENDP\n", func.name));
}

fn emit_instruction(instr: &Instruction) -> String {
    let mnemonic = instr.opcode.name();
    if instr.operands.is_empty() {
        return mnemonic.to_string();
    }
    let ops: Vec<String> = instr.operands.iter().map(emit_operand).collect();
    format!("{} {}", mnemonic, ops.join(", "))
}

fn emit_operand(op: &Operand) -> String {
    match op {
        Operand::Reg(reg) => reg.name(),
        Operand::Imm(val) => {
            if *val < 0 {
                format!("-{}", -val)
            } else {
                val.to_string()
            }
        }
        Operand::Label(name) => name.clone(),
        Operand::StringLit(s) => format!("\"{}\"", s),
        Operand::Memory { base, index, scale, disp } => emit_memory(base, index, *scale, *disp),
    }
}

fn emit_memory(
    base: &Option<Register>,
    index: &Option<Register>,
    scale: u8,
    disp: i64,
) -> String {
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
        if disp >= 0 && !parts.is_empty() {
            parts.push(format!("{}", disp));
        } else if disp < 0 {
            // Negative displacement: remove the implicit '+' by using "- abs"
            return format!("[{} - {}]", parts.join(" + "), -disp);
        } else {
            parts.push(format!("{}", disp));
        }
    }

    format!("[{}]", parts.join(" + "))
}

fn emit_data_item(out: &mut String, name: &str, def: &DataDef) {
    match def {
        DataDef::Byte(vals) => {
            let list: Vec<String> = vals.iter().map(|v| v.to_string()).collect();
            out.push_str(&format!("{} BYTE {}\n", name, list.join(", ")));
        }
        DataDef::Word(vals) => {
            let list: Vec<String> = vals.iter().map(|v| v.to_string()).collect();
            out.push_str(&format!("{} WORD {}\n", name, list.join(", ")));
        }
        DataDef::Dword(vals) => {
            let list: Vec<String> = vals.iter().map(|v| v.to_string()).collect();
            out.push_str(&format!("{} DWORD {}\n", name, list.join(", ")));
        }
        DataDef::Qword(vals) => {
            let list: Vec<String> = vals.iter().map(|v| v.to_string()).collect();
            out.push_str(&format!("{} QWORD {}\n", name, list.join(", ")));
        }
        DataDef::String(s) => {
            out.push_str(&format!("{} BYTE \"{}\", 0\n", name, s));
        }
        DataDef::WString(s) => {
            let units: Vec<String> = s.encode_utf16().map(|u| u.to_string()).collect();
            out.push_str(&format!("{} WORD {}, 0\n", name, units.join(", ")));
        }
        DataDef::ReserveBytes(n) => {
            out.push_str(&format!("{} BYTE {} DUP(?)\n", name, n));
        }
        DataDef::ReserveWords(n) => {
            out.push_str(&format!("{} WORD {} DUP(?)\n", name, n));
        }
        DataDef::ReserveDwords(n) => {
            out.push_str(&format!("{} DWORD {} DUP(?)\n", name, n));
        }
        DataDef::ReserveQwords(n) => {
            out.push_str(&format!("{} QWORD {} DUP(?)\n", name, n));
        }
    }
}
