use crate::ir::*;
use crate::emitters::{Emitter, OutputFormat};

pub struct GasEmitter;

impl Emitter for GasEmitter {
    fn format(&self) -> OutputFormat {
        OutputFormat::Gas
    }

    fn emit(&self, program: &Program) -> String {
        let mut out = String::new();

        for section in &program.sections {
            match &section.kind {
                SectionKind::Text => out.push_str(".text\n"),
                SectionKind::Data => out.push_str(".data\n"),
                SectionKind::Bss => out.push_str(".bss\n"),
                SectionKind::Rodata => out.push_str(".section .rodata\n"),
                SectionKind::Custom(name) => {
                    out.push_str(&format!(".section {}\n", name));
                }
            }

            for data_item in &section.data {
                out.push_str(&format!("{}:\n", data_item.name));
                out.push_str(&emit_data_def(&data_item.def));
            }

            for func in &section.functions {
                if func.exported {
                    out.push_str(&format!(".globl {}\n", func.name));
                }
                out.push_str(&format!("{}:\n", func.name));
                for item in &func.instructions {
                    match item {
                        FunctionItem::Label(label) => {
                            out.push_str(&format!(".{}:\n", label));
                        }
                        FunctionItem::Comment(text) => {
                            out.push_str(&format!("    # {}\n", text));
                        }
                        FunctionItem::Instruction(instr) => {
                            out.push_str(&emit_instruction(instr));
                        }
                    }
                }
            }

            out.push('\n');
        }

        out
    }
}

fn emit_data_def(def: &DataDef) -> String {
    match def {
        DataDef::Byte(vals) => {
            vals.iter()
                .map(|v| format!("    .byte {}\n", v))
                .collect()
        }
        DataDef::Word(vals) => {
            vals.iter()
                .map(|v| format!("    .word {}\n", v))
                .collect()
        }
        DataDef::Dword(vals) => {
            vals.iter()
                .map(|v| format!("    .long {}\n", v))
                .collect()
        }
        DataDef::Qword(vals) => {
            vals.iter()
                .map(|v| format!("    .quad {}\n", v))
                .collect()
        }
        DataDef::String(s) => {
            format!("    .asciz \"{}\"\n", escape_string(s))
        }
        DataDef::WString(s) => {
            let mut out = String::new();
            for ch in s.encode_utf16() {
                out.push_str(&format!("    .word {}\n", ch));
            }
            out.push_str("    .word 0\n");
            out
        }
        DataDef::ReserveBytes(n) => format!("    .space {}\n", n),
        DataDef::ReserveWords(n) => format!("    .space {}\n", n * 2),
        DataDef::ReserveDwords(n) => format!("    .space {}\n", n * 4),
        DataDef::ReserveQwords(n) => format!("    .space {}\n", n * 8),
        DataDef::Float32(vals) => {
            let vs: Vec<String> = vals.iter().map(|v| format!("{}", v)).collect();
            format!("    .float {}\n", vs.join(", "))
        }
        DataDef::Float64(vals) => {
            let vs: Vec<String> = vals.iter().map(|v| format!("{}", v)).collect();
            format!("    .double {}\n", vs.join(", "))
        }
        DataDef::Struct(_struct_name, fields) => {
            let mut out = String::new();
            for field in fields {
                out.push_str(&format!("{}:\n", field.name));
                out.push_str(&emit_data_def(&field.def));
            }
            out
        }
    }
}

fn escape_string(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '\0' => out.push_str("\\0"),
            c => out.push(c),
        }
    }
    out
}

fn emit_instruction(instr: &Instruction) -> String {
    let opcode_base = instr.opcode.name();
    let operands = &instr.operands;

    if operands.is_empty() {
        return format!("    {}\n", opcode_base);
    }

    let suffix = infer_suffix(operands);
    let needs_suffix = opcode_needs_suffix(&instr.opcode);

    let opcode_str = if needs_suffix {
        format!("{}{}", opcode_base, suffix)
    } else {
        opcode_base.to_string()
    };

    // AT&T reverses operand order: for 2 operands, emit operands[1] (src) then operands[0] (dst)
    let operand_strs: Vec<String> = if operands.len() == 2 {
        vec![
            emit_operand(&operands[1]),
            emit_operand(&operands[0]),
        ]
    } else if operands.len() == 3 {
        // 3-operand: reverse all
        vec![
            emit_operand(&operands[2]),
            emit_operand(&operands[1]),
            emit_operand(&operands[0]),
        ]
    } else {
        // 1 operand: no reversal
        operands.iter().map(emit_operand).collect()
    };

    format!("    {} {}\n", opcode_str, operand_strs.join(", "))
}

fn emit_operand(op: &Operand) -> String {
    match op {
        Operand::Reg(reg) => format!("%{}", reg.name()),
        Operand::Imm(val) => format!("${}", val),
        Operand::Label(name) => name.clone(),
        Operand::Memory { base, index, scale, disp } => {
            emit_memory(*disp, base.as_ref(), index.as_ref(), *scale)
        }
        Operand::StringLit(s) => format!("$\"{}\"", escape_string(s)),
    }
}

fn emit_memory(disp: i64, base: Option<&Register>, index: Option<&Register>, scale: u8) -> String {
    let disp_str = if disp != 0 {
        format!("{}", disp)
    } else {
        String::new()
    };

    match (base, index) {
        (Some(b), Some(i)) => {
            if scale > 1 {
                format!("{}(%{}, %{}, {})", disp_str, b.name(), i.name(), scale)
            } else {
                format!("{}(%{}, %{})", disp_str, b.name(), i.name())
            }
        }
        (Some(b), None) => {
            format!("{}(%{})", disp_str, b.name())
        }
        (None, Some(i)) => {
            if scale > 1 {
                format!("{}(, %{}, {})", disp_str, i.name(), scale)
            } else {
                format!("{}(, %{})", disp_str, i.name())
            }
        }
        (None, None) => {
            format!("{}", disp)
        }
    }
}

fn infer_suffix(operands: &[Operand]) -> &'static str {
    for op in operands {
        if let Operand::Reg(reg) = op {
            return size_to_suffix(&reg.size());
        }
        if let Operand::Memory { base, index, .. } = op {
            if let Some(b) = base {
                return size_to_suffix(&b.size());
            }
            if let Some(i) = index {
                return size_to_suffix(&i.size());
            }
        }
    }
    "q"
}

fn size_to_suffix(size: &Size) -> &'static str {
    match size {
        Size::Byte => "b",
        Size::Word => "w",
        Size::Dword => "l",
        Size::Qword => "q",
        Size::Xmmword | Size::Ymmword | Size::Zmmword => "",
    }
}

fn opcode_needs_suffix(opcode: &Opcode) -> bool {
    matches!(
        opcode,
        Opcode::Mov | Opcode::Movzx | Opcode::Movsx |
        Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Imul |
        Opcode::Div | Opcode::Idiv | Opcode::Inc | Opcode::Dec | Opcode::Neg |
        Opcode::And | Opcode::Or | Opcode::Xor | Opcode::Not |
        Opcode::Shl | Opcode::Shr | Opcode::Sar | Opcode::Rol | Opcode::Ror |
        Opcode::Cmp | Opcode::Test |
        Opcode::Lea | Opcode::Xchg |
        Opcode::Push | Opcode::Pop
    )
}
