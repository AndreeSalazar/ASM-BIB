use super::ast::*;
use super::lexer::Token;
use crate::ir::{
    Arch, DataDef, DataItem, Function, FunctionItem, Instruction, Opcode, Operand,
    Program, Register, Section, SectionKind,
};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    auto_counter: usize,
    pending_data: Vec<DataItem>,
    format: String,
    if_stack: Vec<(String, String)>,
    loop_stack: Vec<(String, String)>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens, pos: 0, auto_counter: 0,
            pending_data: Vec::new(), format: "elf".to_string(),
            if_stack: Vec::new(), loop_stack: Vec::new(),
        }
    }

    fn next_label(&mut self, prefix: &str) -> String {
        let l = format!("__{}{}", prefix, self.auto_counter);
        self.auto_counter += 1;
        l
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
        self.pos += 1;
        tok
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match self.advance() {
            Token::Ident(s) => Ok(s),
            other => Err(format!("expected identifier, got {:?}", other)),
        }
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek(), Token::Newline | Token::Comment(_)) {
            self.advance();
        }
    }

    fn skip_to_newline(&mut self) {
        while !matches!(self.peek(), Token::Newline | Token::Eof) {
            self.advance();
        }
    }

    /// Parse the full .pasm source into a Program IR
    pub fn parse(&mut self) -> Result<Program, String> {
        let mut arch = Arch::X86_64;
        let mut format = "elf".to_string();
        let mut org: Option<u64> = None;
        let mut sections: Vec<Section> = Vec::new();
        let mut current_section: Option<Section> = None;
        let mut pending_export = false;
        let mut pending_naked = false;
        let mut pending_macro = false;

        self.skip_newlines();

        while !matches!(self.peek(), Token::Eof) {
            match self.peek().clone() {
                Token::At => {
                    self.advance(); // @
                    let directive = self.expect_ident()?;
                    match directive.as_str() {
                        "arch" => {
                            self.expect_lparen()?;
                            let arch_str = self.expect_string_or_ident()?;
                            self.expect_rparen()?;
                            arch = Arch::from_str(&arch_str)
                                .ok_or(format!("unknown arch: {}", arch_str))?;
                        }
                        "format" => {
                            self.expect_lparen()?;
                            format = self.expect_string_or_ident()?;
                            self.expect_rparen()?;
                            self.format = format.clone();
                        }
                        "org" => {
                            self.expect_lparen()?;
                            org = Some(self.expect_integer()? as u64);
                            self.expect_rparen()?;
                        }
                        "section" => {
                            self.expect_lparen()?;
                            let sec_name = self.expect_string_or_ident()?;
                            self.expect_rparen()?;
                            // save previous section
                            if let Some(sec) = current_section.take() {
                                sections.push(sec);
                            }
                            let kind = match sec_name.as_str() {
                                ".text" | "text" => SectionKind::Text,
                                ".data" | "data" => SectionKind::Data,
                                ".bss" | "bss" => SectionKind::Bss,
                                other => SectionKind::Custom(other.to_string()),
                            };
                            current_section = Some(Section {
                                kind,
                                functions: Vec::new(),
                                data: Vec::new(),
                            });
                        }
                        "export" => { pending_export = true; }
                        "naked" => { pending_naked = true; }
                        "macro" => { pending_macro = true; }
                        "label" => {
                            self.expect_lparen()?;
                            let label_name = self.expect_string_or_ident()?;
                            self.expect_rparen()?;
                            // Add label to current function
                            if let Some(ref mut sec) = current_section {
                                if let Some(func) = sec.functions.last_mut() {
                                    func.instructions.push(FunctionItem::Label(label_name));
                                }
                            }
                        }
                        _ => {
                            self.skip_to_newline();
                        }
                    }
                }
                Token::Def => {
                    self.advance(); // def
                    let name = self.expect_ident()?;
                    // skip params: (...)
                    if matches!(self.peek(), Token::LParen) {
                        self.advance();
                        let mut depth = 1;
                        while depth > 0 {
                            match self.advance() {
                                Token::LParen => depth += 1,
                                Token::RParen => depth -= 1,
                                Token::Eof => return Err("unexpected EOF in function params".into()),
                                _ => {}
                            }
                        }
                    }
                    // expect colon
                    if matches!(self.peek(), Token::Colon) {
                        self.advance();
                    }

                    let exported = pending_export;
                    let naked = pending_naked;
                    let _is_macro = pending_macro;
                    pending_export = false;
                    pending_naked = false;
                    pending_macro = false;

                    // Parse body (indented lines)
                    let instructions = self.parse_function_body()?;

                    let func = Function {
                        name,
                        exported,
                        naked,
                        is_inline: false,
                        is_extern: false,
                        params: Vec::new(),
                        local_vars: Vec::new(),
                        instructions,
                    };

                    if let Some(ref mut sec) = current_section {
                        sec.functions.push(func);
                    } else {
                        // auto-create .text section
                        let mut sec = Section {
                            kind: SectionKind::Text,
                            functions: Vec::new(),
                            data: Vec::new(),
                        };
                        sec.functions.push(func);
                        current_section = Some(sec);
                    }
                }
                Token::Class => {
                    self.advance(); // class
                    let class_name = self.expect_ident()?;
                    if matches!(self.peek(), Token::Colon) {
                        self.advance();
                    }
                    self.skip_newlines();

                    // Parse class body: methods defined with def
                    while matches!(self.peek(), Token::Indent(_)) {
                        self.advance(); // indent

                        match self.peek().clone() {
                            Token::Def => {
                                self.advance(); // def
                                let method_name = self.expect_ident()?;
                                // skip params
                                if matches!(self.peek(), Token::LParen) {
                                    self.advance();
                                    let mut depth = 1;
                                    while depth > 0 {
                                        match self.advance() {
                                            Token::LParen => depth += 1,
                                            Token::RParen => depth -= 1,
                                            Token::Eof => return Err("unexpected EOF in method params".into()),
                                            _ => {}
                                        }
                                    }
                                }
                                if matches!(self.peek(), Token::Colon) {
                                    self.advance();
                                }

                                let instructions = self.parse_function_body()?;
                                let func_name = format!("{}_{}", class_name, method_name);

                                let func = Function {
                                    name: func_name,
                                    exported: pending_export,
                                    naked: false,
                                    is_inline: false,
                                    is_extern: false,
                                    params: Vec::new(),
                                    local_vars: Vec::new(),
                                    instructions,
                                };
                                pending_export = false;

                                if let Some(ref mut sec) = current_section {
                                    sec.functions.push(func);
                                } else {
                                    let mut sec = Section {
                                        kind: SectionKind::Text,
                                        functions: Vec::new(),
                                        data: Vec::new(),
                                    };
                                    sec.functions.push(func);
                                    current_section = Some(sec);
                                }
                            }
                            _ => { self.skip_to_newline(); }
                        }

                        if matches!(self.peek(), Token::Newline) {
                            self.advance();
                        }
                    }
                }
                Token::Ident(_) => {
                    // Could be: name = data_type(value) OR instruction call at top level
                    let ident = if let Token::Ident(s) = self.advance() { s } else { unreachable!() };

                    if matches!(self.peek(), Token::Equals) {
                        // Data assignment: name = type(value)
                        self.advance(); // =
                        let data_item = self.parse_data_value(&ident)?;
                        if let Some(ref mut sec) = current_section {
                            sec.data.push(data_item);
                        }
                    } else if matches!(self.peek(), Token::LParen) {
                        // Top-level instruction call (rare, but possible)
                        let args = self.parse_call_args()?;
                        let inst = self.build_instruction(&ident, &args)?;
                        if let Some(ref mut sec) = current_section {
                            if let Some(func) = sec.functions.last_mut() {
                                func.instructions.push(FunctionItem::Instruction(inst));
                            }
                        }
                    } else {
                        self.skip_to_newline();
                    }
                }
                Token::Indent(_) => { self.advance(); }
                Token::Newline => { self.advance(); }
                Token::Comment(_) => { self.advance(); }
                _ => { self.advance(); }
            }
        }

        if let Some(sec) = current_section {
            sections.push(sec);
        }

        // Inject auto-generated string data from print() expansions
        if !self.pending_data.is_empty() {
            let data_sec = sections.iter_mut().find(|s| s.kind == SectionKind::Data);
            if let Some(sec) = data_sec {
                sec.data.extend(self.pending_data.drain(..));
            } else {
                let sec = Section {
                    kind: SectionKind::Data,
                    functions: Vec::new(),
                    data: self.pending_data.drain(..).collect(),
                };
                sections.insert(0, sec);
            }
        }

        let mut program = Program::new(arch);
        program.format = format;
        program.org = org;
        program.sections = sections;
        Ok(program)
    }

    fn parse_function_body(&mut self) -> Result<Vec<FunctionItem>, String> {
        let mut items = Vec::new();
        self.skip_newlines();

        while matches!(self.peek(), Token::Indent(_)) {
            self.advance(); // consume indent

            match self.peek().clone() {
                Token::At => {
                    self.advance(); // @
                    let dir = self.expect_ident()?;
                    match dir.as_str() {
                        "label" => {
                            self.expect_lparen()?;
                            let name = self.expect_string_or_ident()?;
                            self.expect_rparen()?;
                            items.push(FunctionItem::Label(name));
                        }
                        "if" => {
                            // @if(reg, op, val) → cmp + conditional jump
                            self.expect_lparen()?;
                            let lhs = self.expect_string_or_ident()?;
                            self.expect_comma()?;
                            let op = self.expect_operator()?;
                            self.expect_comma()?;
                            let rhs = self.parse_expr()?;
                            self.expect_rparen()?;

                            let else_lbl = self.next_label("else_");
                            let endif_lbl = self.next_label("endif_");
                            self.if_stack.push((else_lbl.clone(), endif_lbl));

                            let lhs_op = self.ident_to_operand(&lhs);
                            let rhs_op = self.expr_to_operand(&rhs);
                            items.push(FunctionItem::Instruction(Instruction::two(Opcode::Cmp, lhs_op, rhs_op)));

                            let jcc = match op.as_str() {
                                "==" | "eq" => Opcode::Jne,
                                "!=" | "ne" => Opcode::Je,
                                "<" | "lt"  => Opcode::Jge,
                                "<=" | "le" => Opcode::Jg,
                                ">" | "gt"  => Opcode::Jge,
                                ">=" | "ge" => Opcode::Jl,
                                _ => Opcode::Jne,
                            };
                            // Jump to else/endif if condition is FALSE
                            items.push(FunctionItem::Instruction(Instruction::one(jcc, Operand::Label(else_lbl))));
                        }
                        "else" => {
                            if let Some((else_lbl, endif_lbl)) = self.if_stack.last() {
                                let endif = endif_lbl.clone();
                                let else_l = else_lbl.clone();
                                items.push(FunctionItem::Instruction(Instruction::one(Opcode::Jmp, Operand::Label(endif))));
                                items.push(FunctionItem::Label(else_l));
                            }
                        }
                        "endif" => {
                            if let Some((else_lbl, endif_lbl)) = self.if_stack.pop() {
                                // If no @else was used, the else label still needs to land here
                                items.push(FunctionItem::Label(else_lbl));
                                items.push(FunctionItem::Label(endif_lbl));
                            }
                        }
                        "loop" => {
                            // @loop(reg, count) → mov reg, count; label:
                            self.expect_lparen()?;
                            let reg_name = self.expect_string_or_ident()?;
                            self.expect_comma()?;
                            let count = self.parse_expr()?;
                            self.expect_rparen()?;

                            let start_lbl = self.next_label("loop_");
                            let end_lbl = self.next_label("loopend_");
                            self.loop_stack.push((start_lbl.clone(), end_lbl));

                            let reg_op = self.ident_to_operand(&reg_name);
                            let count_op = self.expr_to_operand(&count);
                            items.push(FunctionItem::Instruction(Instruction::two(Opcode::Mov, reg_op, count_op)));
                            items.push(FunctionItem::Label(start_lbl));
                        }
                        "endloop" => {
                            if let Some((start_lbl, end_lbl)) = self.loop_stack.pop() {
                                items.push(FunctionItem::Instruction(Instruction::two(Opcode::Sub, Operand::Reg(Register::Rcx), Operand::Imm(1))));
                                items.push(FunctionItem::Instruction(Instruction::one(Opcode::Jne, Operand::Label(start_lbl))));
                                items.push(FunctionItem::Label(end_lbl));
                            }
                        }
                        "while" => {
                            // @while(reg, op, val) → label: cmp; jcc end
                            self.expect_lparen()?;
                            let lhs = self.expect_string_or_ident()?;
                            self.expect_comma()?;
                            let op_str = self.expect_operator()?;
                            self.expect_comma()?;
                            let rhs = self.parse_expr()?;
                            self.expect_rparen()?;

                            let start_lbl = self.next_label("while_");
                            let end_lbl = self.next_label("wend_");
                            self.loop_stack.push((start_lbl.clone(), end_lbl.clone()));

                            items.push(FunctionItem::Label(start_lbl));
                            let lhs_op = self.ident_to_operand(&lhs);
                            let rhs_op = self.expr_to_operand(&rhs);
                            items.push(FunctionItem::Instruction(Instruction::two(Opcode::Cmp, lhs_op, rhs_op)));

                            let jcc = match op_str.as_str() {
                                "==" | "eq" => Opcode::Jne,
                                "!=" | "ne" => Opcode::Je,
                                "<" | "lt"  => Opcode::Jge,
                                ">" | "gt"  => Opcode::Jle,
                                _ => Opcode::Jne,
                            };
                            items.push(FunctionItem::Instruction(Instruction::one(jcc, Operand::Label(end_lbl))));
                        }
                        "endwhile" => {
                            if let Some((start_lbl, end_lbl)) = self.loop_stack.pop() {
                                items.push(FunctionItem::Instruction(Instruction::one(Opcode::Jmp, Operand::Label(start_lbl))));
                                items.push(FunctionItem::Label(end_lbl));
                            }
                        }
                        "break" => {
                            if let Some((_, end_lbl)) = self.loop_stack.last() {
                                let end = end_lbl.clone();
                                items.push(FunctionItem::Instruction(Instruction::one(Opcode::Jmp, Operand::Label(end))));
                            }
                        }
                        "continue" => {
                            if let Some((start_lbl, _)) = self.loop_stack.last() {
                                let start = start_lbl.clone();
                                items.push(FunctionItem::Instruction(Instruction::one(Opcode::Jmp, Operand::Label(start))));
                            }
                        }
                        _ => { self.skip_to_newline(); }
                    }
                }
                Token::Ident(_) => {
                    let name = if let Token::Ident(s) = self.advance() { s } else { unreachable!() };

                    // Handle compound instructions like "rep movsb", "rep stosb", "repe cmpsb"
                    let final_name = if name == "rep" || name == "repe" || name == "repne" {
                        if let Token::Ident(ref suffix) = *self.peek() {
                            let compound = format!("{} {}", name, suffix);
                            self.advance();
                            compound
                        } else {
                            name
                        }
                    } else {
                        name
                    };

                    if matches!(self.peek(), Token::LParen) {
                        let args = self.parse_call_args()?;
                        if let Some(expanded) = self.try_expand_builtin(&final_name, &args)? {
                            for item in expanded {
                                items.push(item);
                            }
                        } else {
                            let inst = self.build_instruction(&final_name, &args)?;
                            items.push(FunctionItem::Instruction(inst));
                        }
                    } else if matches!(self.peek(), Token::Equals) {
                        self.skip_to_newline();
                    } else {
                        if let Some(opcode) = Opcode::from_str(&final_name) {
                            items.push(FunctionItem::Instruction(Instruction::zero(opcode)));
                        }
                    }
                }
                Token::Comment(_) => { self.advance(); }
                _ => { self.skip_to_newline(); }
            }

            // consume trailing newline
            if matches!(self.peek(), Token::Newline) {
                self.advance();
            }
        }

        Ok(items)
    }

    fn parse_call_args(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        if !matches!(self.peek(), Token::LParen) {
            return Ok(args);
        }
        self.advance(); // (

        if matches!(self.peek(), Token::RParen) {
            self.advance();
            return Ok(args);
        }

        loop {
            let arg = self.parse_expr()?;
            args.push(arg);
            match self.peek() {
                Token::Comma => { self.advance(); }
                Token::RParen => { self.advance(); break; }
                _ => { self.advance(); break; }
            }
        }

        Ok(args)
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        match self.peek().clone() {
            Token::Ident(s) => {
                self.advance();
                // Check if it's a register
                if Register::from_str(&s).is_some() {
                    Ok(Expr::Register(s))
                } else {
                    Ok(Expr::Label(s))
                }
            }
            Token::Integer(v) | Token::HexInteger(v) => {
                self.advance();
                Ok(Expr::Immediate(v))
            }
            Token::FloatLiteral(v) => {
                self.advance();
                Ok(Expr::Label(format!("{}", v)))
            }
            Token::StringLiteral(s) => {
                self.advance();
                Ok(Expr::StringLit(s))
            }
            Token::LBracket => {
                self.advance();
                // Parse memory expression [base + index*scale + disp]
                let mut base = None;
                let mut index = None;
                let mut scale = 1u8;
                let mut disp = 0i64;

                loop {
                    match self.peek().clone() {
                        Token::RBracket => { self.advance(); break; }
                        Token::Ident(r) => {
                            self.advance();
                            if base.is_none() {
                                base = Some(r);
                            } else {
                                index = Some(r);
                            }
                        }
                        Token::Integer(v) | Token::HexInteger(v) => {
                            self.advance();
                            // Check if this is a scale or displacement
                            if matches!(self.peek(), Token::Star) {
                                self.advance(); // *
                                scale = v as u8;
                            } else {
                                disp = v;
                            }
                        }
                        Token::Plus | Token::Minus => { self.advance(); }
                        Token::Star => {
                            self.advance();
                            if let Token::Integer(v) | Token::HexInteger(v) = self.peek().clone() {
                                self.advance();
                                scale = v as u8;
                            }
                        }
                        Token::Eof => return Err("unexpected EOF in memory expr".into()),
                        _ => { self.advance(); }
                    }
                }

                Ok(Expr::Memory(Box::new(MemExpr { base, index, scale, disp })))
            }
            other => Err(format!("unexpected token in expression: {:?}", other)),
        }
    }

    fn build_instruction(&self, name: &str, args: &[Expr]) -> Result<Instruction, String> {
        let opcode = Opcode::from_str(name)
            .ok_or_else(|| format!("unknown instruction: {}", name))?;

        let operands: Vec<Operand> = args.iter().map(|a| self.expr_to_operand(a)).collect();

        Ok(Instruction::new(opcode, operands))
    }

    fn expr_to_operand(&self, expr: &Expr) -> Operand {
        match expr {
            Expr::Register(name) => {
                Register::from_str(name)
                    .map(Operand::Reg)
                    .unwrap_or_else(|| Operand::Label(name.clone()))
            }
            Expr::Immediate(v) => Operand::Imm(*v),
            Expr::Label(s) => Operand::Label(s.clone()),
            Expr::StringLit(s) => Operand::StringLit(s.clone()),
            Expr::Memory(mem) => {
                let base_reg = mem.base.as_ref().and_then(|r| Register::from_str(r));
                let index_reg = mem.index.as_ref().and_then(|r| Register::from_str(r));

                // If base is a non-register name (label/data symbol) with no register parts
                if let Some(ref base_name) = mem.base {
                    if base_reg.is_none() && index_reg.is_none() {
                        if mem.disp != 0 {
                            return Operand::Label(format!("{} + {}", base_name, mem.disp));
                        }
                        return Operand::Label(base_name.clone());
                    }
                }

                Operand::Memory {
                    base: base_reg,
                    index: index_reg,
                    scale: mem.scale,
                    disp: mem.disp,
                }
            }
            Expr::Bool(true) => Operand::Imm(1),
            Expr::Bool(false) | Expr::Null => Operand::Imm(0),
            Expr::Call { name, .. } => Operand::Label(name.clone()),
            Expr::NamespaceAccess { path } => Operand::Label(path.join("::")),
            Expr::FieldAccess { object, field } => {
                if let Expr::Register(r) = object.as_ref() {
                    Operand::Label(format!("{}.{}", r, field))
                } else {
                    Operand::Label(field.clone())
                }
            }
            _ => Operand::Imm(0),
        }
    }

    fn parse_data_value(&mut self, name: &str) -> Result<DataItem, String> {
        let type_name = self.expect_ident()?;
        let def = match type_name.as_str() {
            "byte" => {
                let args = self.parse_call_args()?;
                let vals: Vec<u8> = args.iter().filter_map(|a| {
                    if let Expr::Immediate(v) = a { Some(*v as u8) } else { None }
                }).collect();
                DataDef::Byte(vals)
            }
            "word" => {
                let args = self.parse_call_args()?;
                let vals: Vec<u16> = args.iter().filter_map(|a| {
                    if let Expr::Immediate(v) = a { Some(*v as u16) } else { None }
                }).collect();
                DataDef::Word(vals)
            }
            "dword" => {
                let args = self.parse_call_args()?;
                let vals: Vec<u32> = args.iter().filter_map(|a| {
                    if let Expr::Immediate(v) = a { Some(*v as u32) } else { None }
                }).collect();
                DataDef::Dword(vals)
            }
            "qword" => {
                let args = self.parse_call_args()?;
                let vals: Vec<u64> = args.iter().filter_map(|a| {
                    if let Expr::Immediate(v) = a { Some(*v as u64) } else { None }
                }).collect();
                DataDef::Qword(vals)
            }
            "string" => {
                let args = self.parse_call_args()?;
                let s = args.iter().find_map(|a| {
                    if let Expr::StringLit(s) = a { Some(s.clone()) } else { None }
                }).unwrap_or_default();
                DataDef::String(s)
            }
            "wstring" => {
                let args = self.parse_call_args()?;
                let s = args.iter().find_map(|a| {
                    if let Expr::StringLit(s) = a { Some(s.clone()) } else { None }
                }).unwrap_or_default();
                DataDef::WString(s)
            }
            "resb" => {
                let args = self.parse_call_args()?;
                let n = args.first().and_then(|a| if let Expr::Immediate(v) = a { Some(*v as usize) } else { None }).unwrap_or(1);
                DataDef::ReserveBytes(n)
            }
            "resd" => {
                let args = self.parse_call_args()?;
                let n = args.first().and_then(|a| if let Expr::Immediate(v) = a { Some(*v as usize) } else { None }).unwrap_or(1);
                DataDef::ReserveDwords(n)
            }
            "float32" | "real4" => {
                let args = self.parse_call_args()?;
                let vals: Vec<f32> = args.iter().filter_map(|a| {
                    match a {
                        Expr::Immediate(v) => Some(*v as f32),
                        Expr::Label(s) => s.parse::<f32>().ok(),
                        _ => None,
                    }
                }).collect();
                DataDef::Float32(vals)
            }
            "float64" | "real8" => {
                let args = self.parse_call_args()?;
                let vals: Vec<f64> = args.iter().filter_map(|a| {
                    match a {
                        Expr::Immediate(v) => Some(*v as f64),
                        Expr::Label(s) => s.parse::<f64>().ok(),
                        _ => None,
                    }
                }).collect();
                DataDef::Float64(vals)
            }
            "resw" => {
                let args = self.parse_call_args()?;
                let n = args.first().and_then(|a| if let Expr::Immediate(v) = a { Some(*v as usize) } else { None }).unwrap_or(1);
                DataDef::ReserveWords(n)
            }
            "resq" => {
                let args = self.parse_call_args()?;
                let n = args.first().and_then(|a| if let Expr::Immediate(v) = a { Some(*v as usize) } else { None }).unwrap_or(1);
                DataDef::ReserveQwords(n)
            }
            // Signed type aliases (same storage, semantic sugar)
            "sbyte" | "int8" => {
                let args = self.parse_call_args()?;
                let vals: Vec<u8> = args.iter().filter_map(|a| {
                    if let Expr::Immediate(v) = a { Some(*v as u8) } else { None }
                }).collect();
                DataDef::Byte(vals)
            }
            "sword" | "int16" => {
                let args = self.parse_call_args()?;
                let vals: Vec<u16> = args.iter().filter_map(|a| {
                    if let Expr::Immediate(v) = a { Some(*v as u16) } else { None }
                }).collect();
                DataDef::Word(vals)
            }
            "sdword" | "int32" => {
                let args = self.parse_call_args()?;
                let vals: Vec<u32> = args.iter().filter_map(|a| {
                    if let Expr::Immediate(v) = a { Some(*v as u32) } else { None }
                }).collect();
                DataDef::Dword(vals)
            }
            "sqword" | "int64" => {
                let args = self.parse_call_args()?;
                let vals: Vec<u64> = args.iter().filter_map(|a| {
                    if let Expr::Immediate(v) = a { Some(*v as u64) } else { None }
                }).collect();
                DataDef::Qword(vals)
            }
            "array" => {
                // array(type, count) → generates N DUP(0)
                let args = self.parse_call_args()?;
                let type_str = match args.first() {
                    Some(Expr::Label(s)) => s.as_str(),
                    _ => "byte",
                };
                let count = match args.get(1) {
                    Some(Expr::Immediate(v)) => *v as usize,
                    _ => 1,
                };
                match type_str {
                    "byte" | "sbyte" => DataDef::ReserveBytes(count),
                    "word" | "sword" => DataDef::ReserveWords(count),
                    "dword" | "sdword" => DataDef::ReserveDwords(count),
                    "qword" | "sqword" => DataDef::ReserveQwords(count),
                    _ => DataDef::ReserveBytes(count),
                }
            }
            "buffer" => {
                // buffer(size) → BYTE size DUP(?)
                let args = self.parse_call_args()?;
                let n = args.first().and_then(|a| if let Expr::Immediate(v) = a { Some(*v as usize) } else { None }).unwrap_or(256);
                DataDef::ReserveBytes(n)
            }
            _ => return Err(format!("unknown data type: {}", type_name)),
        };

        Ok(DataItem::new(name.to_string(), def))
    }

    // ── High-level builtins ────────────────────────────────────────────

    fn ident_to_operand(&self, name: &str) -> Operand {
        Register::from_str(name)
            .map(Operand::Reg)
            .unwrap_or_else(|| Operand::Label(name.to_string()))
    }

    fn win64_call(&self, func: &str, args: &[Operand]) -> Vec<Instruction> {
        let regs = [Register::Rcx, Register::Rdx, Register::R8, Register::R9];
        let mut v = vec![Instruction::two(Opcode::Sub, Operand::Reg(Register::Rsp), Operand::Imm(40))];
        for (i, arg) in args.iter().enumerate().take(4) {
            match arg {
                Operand::Label(_) => v.push(Instruction::two(Opcode::Lea, Operand::Reg(regs[i].clone()), arg.clone())),
                _ => v.push(Instruction::two(Opcode::Mov, Operand::Reg(regs[i].clone()), arg.clone())),
            }
        }
        v.push(Instruction::one(Opcode::Call, Operand::Label(func.to_string())));
        v.push(Instruction::two(Opcode::Add, Operand::Reg(Register::Rsp), Operand::Imm(40)));
        v
    }

    fn try_expand_builtin(&mut self, name: &str, args: &[Expr]) -> Result<Option<Vec<FunctionItem>>, String> {
        let insts = match name {
            "print"   => Some(self.expand_print(args)?),
            "exit"    => Some(self.expand_exit(args)?),
            "printf"  => Some(self.expand_printf(args)?),
            "input"   => Some(self.expand_input(args)?),
            "alloc"   => Some(self.expand_alloc(args)?),
            "free"    => Some(self.expand_free(args)?),
            "memcpy"  => Some(self.expand_memcpy(args)?),
            "memset"  => Some(self.expand_memset(args)?),
            "memcmp"  => Some(self.expand_crt3("memcmp", args)?),
            "strlen"  => Some(self.expand_crt1("strlen", args)?),
            "strcpy"  => Some(self.expand_crt2("strcpy", args)?),
            "strcmp"   => Some(self.expand_crt2("strcmp", args)?),
            "strcat"  => Some(self.expand_crt2("strcat", args)?),
            "abs"     => Some(self.expand_abs(args)?),
            "min"     => Some(self.expand_minmax(args, true)?),
            "max"     => Some(self.expand_minmax(args, false)?),
            "sqrt"    => Some(self.expand_sqrt(args)?),
            "vec_add" => Some(self.expand_simd(Opcode::Vaddps, args)?),
            "vec_mul" => Some(self.expand_simd(Opcode::Vmulps, args)?),
            "vec_sub" => Some(self.expand_simd(Opcode::Vsubps, args)?),
            "vec_div" => Some(self.expand_simd(Opcode::Vdivps, args)?),
            _ => None,
        };
        Ok(insts.map(|v| v.into_iter().map(FunctionItem::Instruction).collect()))
    }

    // ── I/O ──

    fn expand_print(&mut self, args: &[Expr]) -> Result<Vec<Instruction>, String> {
        if let Some(Expr::StringLit(s)) = args.first() {
            let label = self.next_label("str_");
            self.pending_data.push(DataItem::new(label.clone(), DataDef::String(s.clone())));
            if self.format.contains("win") {
                Ok(self.win64_call("printf", &[Operand::Label(label)]))
            } else {
                let len = s.len() as i64;
                Ok(vec![
                    Instruction::two(Opcode::Mov, Operand::Reg(Register::Rax), Operand::Imm(1)),
                    Instruction::two(Opcode::Mov, Operand::Reg(Register::Rdi), Operand::Imm(1)),
                    Instruction::two(Opcode::Lea, Operand::Reg(Register::Rsi), Operand::Label(label)),
                    Instruction::two(Opcode::Mov, Operand::Reg(Register::Rdx), Operand::Imm(len)),
                    Instruction::zero(Opcode::Syscall),
                ])
            }
        } else if let Some(expr) = args.first() {
            let op = self.expr_to_operand(expr);
            if self.format.contains("win") {
                Ok(self.win64_call("printf", &[op]))
            } else {
                Ok(vec![
                    Instruction::two(Opcode::Mov, Operand::Reg(Register::Rax), Operand::Imm(1)),
                    Instruction::two(Opcode::Mov, Operand::Reg(Register::Rdi), Operand::Imm(1)),
                    Instruction::two(Opcode::Lea, Operand::Reg(Register::Rsi), op),
                    Instruction::two(Opcode::Mov, Operand::Reg(Register::Rdx), Operand::Imm(256)),
                    Instruction::zero(Opcode::Syscall),
                ])
            }
        } else {
            Ok(vec![])
        }
    }

    fn expand_exit(&mut self, args: &[Expr]) -> Result<Vec<Instruction>, String> {
        let code = match args.first() { Some(Expr::Immediate(v)) => *v, _ => 0 };
        if self.format.contains("win") {
            let mut v = Vec::new();
            if code == 0 {
                v.push(Instruction::two(Opcode::Xor, Operand::Reg(Register::Ecx), Operand::Reg(Register::Ecx)));
            } else {
                v.push(Instruction::two(Opcode::Mov, Operand::Reg(Register::Ecx), Operand::Imm(code)));
            }
            v.push(Instruction::one(Opcode::Call, Operand::Label("ExitProcess".to_string())));
            Ok(v)
        } else {
            Ok(vec![
                Instruction::two(Opcode::Mov, Operand::Reg(Register::Rax), Operand::Imm(60)),
                Instruction::two(Opcode::Mov, Operand::Reg(Register::Rdi), Operand::Imm(code)),
                Instruction::zero(Opcode::Syscall),
            ])
        }
    }

    fn expand_printf(&mut self, args: &[Expr]) -> Result<Vec<Instruction>, String> {
        // printf(fmt_string_or_label, arg1, arg2, ...)
        let ops: Vec<Operand> = args.iter().map(|a| {
            if let Expr::StringLit(s) = a {
                let label = self.next_label("fmt_");
                self.pending_data.push(DataItem::new(label.clone(), DataDef::String(s.clone())));
                Operand::Label(label)
            } else {
                self.expr_to_operand(a)
            }
        }).collect();
        if self.format.contains("win") {
            Ok(self.win64_call("printf", &ops))
        } else {
            // Linux: just call printf (linked with libc)
            Ok(self.win64_call("printf", &ops)) // same ABI for simplicity
        }
    }

    fn expand_input(&mut self, args: &[Expr]) -> Result<Vec<Instruction>, String> {
        // input(buffer, max_size) → ReadConsoleA or scanf
        let fmt_label = self.next_label("fmt_");
        self.pending_data.push(DataItem::new(fmt_label.clone(), DataDef::String("%s".to_string())));
        let buf_op = args.first().map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Imm(0));
        if self.format.contains("win") {
            Ok(self.win64_call("scanf", &[Operand::Label(fmt_label), buf_op]))
        } else {
            Ok(self.win64_call("scanf", &[Operand::Label(fmt_label), buf_op]))
        }
    }

    // ── Memory ──

    fn expand_alloc(&mut self, args: &[Expr]) -> Result<Vec<Instruction>, String> {
        // alloc(size) → GetProcessHeap + HeapAlloc, result in rax
        let size = args.first().map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Imm(0));
        Ok(vec![
            Instruction::two(Opcode::Sub, Operand::Reg(Register::Rsp), Operand::Imm(40)),
            Instruction::one(Opcode::Call, Operand::Label("GetProcessHeap".to_string())),
            Instruction::two(Opcode::Mov, Operand::Reg(Register::Rcx), Operand::Reg(Register::Rax)),
            Instruction::two(Opcode::Xor, Operand::Reg(Register::Edx), Operand::Reg(Register::Edx)),
            Instruction::two(Opcode::Mov, Operand::Reg(Register::R8), size),
            Instruction::one(Opcode::Call, Operand::Label("HeapAlloc".to_string())),
            Instruction::two(Opcode::Add, Operand::Reg(Register::Rsp), Operand::Imm(40)),
        ])
    }

    fn expand_free(&mut self, args: &[Expr]) -> Result<Vec<Instruction>, String> {
        // free(ptr_reg) → GetProcessHeap + HeapFree
        let ptr = args.first().map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Imm(0));
        Ok(vec![
            Instruction::two(Opcode::Sub, Operand::Reg(Register::Rsp), Operand::Imm(40)),
            Instruction::one(Opcode::Push, ptr),
            Instruction::one(Opcode::Call, Operand::Label("GetProcessHeap".to_string())),
            Instruction::two(Opcode::Mov, Operand::Reg(Register::Rcx), Operand::Reg(Register::Rax)),
            Instruction::two(Opcode::Xor, Operand::Reg(Register::Edx), Operand::Reg(Register::Edx)),
            Instruction::one(Opcode::Pop, Operand::Reg(Register::R8)),
            Instruction::one(Opcode::Call, Operand::Label("HeapFree".to_string())),
            Instruction::two(Opcode::Add, Operand::Reg(Register::Rsp), Operand::Imm(40)),
        ])
    }

    fn expand_memcpy(&mut self, args: &[Expr]) -> Result<Vec<Instruction>, String> {
        // memcpy(dst, src, n) → inline rep movsb
        let dst = args.first().map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Imm(0));
        let src = args.get(1).map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Imm(0));
        let n   = args.get(2).map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Imm(0));
        Ok(vec![
            Instruction::two(Opcode::Lea, Operand::Reg(Register::Rdi), dst),
            Instruction::two(Opcode::Lea, Operand::Reg(Register::Rsi), src),
            Instruction::two(Opcode::Mov, Operand::Reg(Register::Rcx), n),
            Instruction::zero(Opcode::RepMovsb),
        ])
    }

    fn expand_memset(&mut self, args: &[Expr]) -> Result<Vec<Instruction>, String> {
        // memset(ptr, val, n) → inline rep stosb
        let ptr = args.first().map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Imm(0));
        let val = args.get(1).map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Imm(0));
        let n   = args.get(2).map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Imm(0));
        Ok(vec![
            Instruction::two(Opcode::Lea, Operand::Reg(Register::Rdi), ptr),
            Instruction::two(Opcode::Mov, Operand::Reg(Register::Al), val),
            Instruction::two(Opcode::Mov, Operand::Reg(Register::Rcx), n),
            Instruction::zero(Opcode::RepStosb),
        ])
    }

    // ── CRT wrappers (1/2/3-arg C functions via Win64 ABI) ──

    fn expand_crt1(&mut self, func: &str, args: &[Expr]) -> Result<Vec<Instruction>, String> {
        let a1 = args.first().map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Imm(0));
        Ok(self.win64_call(func, &[a1]))
    }

    fn expand_crt2(&mut self, func: &str, args: &[Expr]) -> Result<Vec<Instruction>, String> {
        let a1 = args.first().map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Imm(0));
        let a2 = args.get(1).map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Imm(0));
        Ok(self.win64_call(func, &[a1, a2]))
    }

    fn expand_crt3(&mut self, func: &str, args: &[Expr]) -> Result<Vec<Instruction>, String> {
        let a1 = args.first().map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Imm(0));
        let a2 = args.get(1).map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Imm(0));
        let a3 = args.get(2).map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Imm(0));
        Ok(self.win64_call(func, &[a1, a2, a3]))
    }

    // ── Math ──

    fn expand_abs(&mut self, args: &[Expr]) -> Result<Vec<Instruction>, String> {
        // abs(reg) → neg + cmovl (in-place absolute value)
        let op = args.first().map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Reg(Register::Rax));
        if let Operand::Reg(ref reg) = op {
            let r = reg.clone();
            Ok(vec![
                Instruction::two(Opcode::Mov, Operand::Reg(Register::Rdx), Operand::Reg(r.clone())),
                Instruction::one(Opcode::Neg, Operand::Reg(Register::Rdx)),
                Instruction::two(Opcode::Cmovl, Operand::Reg(r), Operand::Reg(Register::Rdx)),
            ])
        } else {
            Ok(vec![])
        }
    }

    fn expand_minmax(&mut self, args: &[Expr], is_min: bool) -> Result<Vec<Instruction>, String> {
        // min(a, b) / max(a, b) → cmp + cmov, result in first reg
        let a = args.first().map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Reg(Register::Rax));
        let b = args.get(1).map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Imm(0));
        if let Operand::Reg(ref reg) = a {
            let cmov = if is_min { Opcode::Cmovg } else { Opcode::Cmovl };
            Ok(vec![
                Instruction::two(Opcode::Cmp, a.clone(), b.clone()),
                Instruction::two(cmov, Operand::Reg(reg.clone()), b),
            ])
        } else {
            Ok(vec![])
        }
    }

    fn expand_sqrt(&mut self, args: &[Expr]) -> Result<Vec<Instruction>, String> {
        // sqrt(xmm_dst, xmm_src) → sqrtss
        let dst = args.first().map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Reg(Register::Xmm(0)));
        let src = args.get(1).map(|a| self.expr_to_operand(a)).unwrap_or(dst.clone());
        Ok(vec![Instruction::two(Opcode::Sqrtss, dst, src)])
    }

    // ── SIMD/AVX ──

    fn expand_simd(&mut self, op: Opcode, args: &[Expr]) -> Result<Vec<Instruction>, String> {
        // vec_add(dst, src) or vec_add(dst, src1, src2)
        let ops: Vec<Operand> = args.iter().map(|a| self.expr_to_operand(a)).collect();
        if ops.len() >= 3 {
            Ok(vec![Instruction::new(op, ops)])
        } else if ops.len() == 2 {
            Ok(vec![Instruction::new(op, vec![ops[0].clone(), ops[0].clone(), ops[1].clone()])])
        } else {
            Ok(vec![])
        }
    }

    fn expect_lparen(&mut self) -> Result<(), String> {
        match self.advance() {
            Token::LParen => Ok(()),
            other => Err(format!("expected '(', got {:?}", other)),
        }
    }

    fn expect_operator(&mut self) -> Result<String, String> {
        match self.advance() {
            Token::EqEq => Ok("==".to_string()),
            Token::BangEq => Ok("!=".to_string()),
            Token::Lt => Ok("<".to_string()),
            Token::LtEq => Ok("<=".to_string()),
            Token::Gt => Ok(">".to_string()),
            Token::GtEq => Ok(">=".to_string()),
            Token::StringLiteral(s) => Ok(s),
            Token::Ident(s) => Ok(s),
            other => Err(format!("expected operator, got {:?}", other)),
        }
    }

    fn expect_comma(&mut self) -> Result<(), String> {
        match self.advance() {
            Token::Comma => Ok(()),
            other => Err(format!("expected ',', got {:?}", other)),
        }
    }

    fn expect_rparen(&mut self) -> Result<(), String> {
        match self.advance() {
            Token::RParen => Ok(()),
            other => Err(format!("expected ')', got {:?}", other)),
        }
    }

    fn expect_string_or_ident(&mut self) -> Result<String, String> {
        match self.advance() {
            Token::StringLiteral(s) => Ok(s),
            Token::Ident(s) => Ok(s),
            other => Err(format!("expected string or ident, got {:?}", other)),
        }
    }

    fn expect_integer(&mut self) -> Result<i64, String> {
        match self.advance() {
            Token::Integer(v) | Token::HexInteger(v) => Ok(v),
            other => Err(format!("expected integer, got {:?}", other)),
        }
    }
}
