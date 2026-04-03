use super::ast::*;
use super::lexer::Token;
use crate::ir::{
    Arch, CallingConv, DataDef, DataItem, ExternSymbol, Function, FunctionItem,
    Instruction, Opcode, Operand, Program, Register, Section, SectionKind,
    StructDef, StructField,
};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    auto_counter: usize,
    pending_data: Vec<DataItem>,
    format: String,
    if_stack: Vec<(String, String)>,
    loop_stack: Vec<(String, String)>,
    switch_stack: Vec<(String, String, Option<String>)>, // (end_lbl, reg, next_case_lbl)
    pending_structs: Vec<StructDef>,
    pending_externs: Vec<ExternSymbol>,
    pending_includes: Vec<String>,
    pending_includelibs: Vec<String>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens, pos: 0, auto_counter: 0,
            pending_data: Vec::new(), format: "elf".to_string(),
            if_stack: Vec::new(), loop_stack: Vec::new(),
            switch_stack: Vec::new(),
            pending_structs: Vec::new(),
            pending_externs: Vec::new(),
            pending_includes: Vec::new(),
            pending_includelibs: Vec::new(),
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
            Token::Class => Ok("class".into()),
            Token::Struct => Ok("struct".into()),
            Token::If => Ok("if".into()),
            Token::Else => Ok("else".into()),
            Token::While => Ok("while".into()),
            Token::Def => Ok("def".into()),
            Token::Return => Ok("return".into()),
            Token::Break => Ok("break".into()),
            Token::Continue => Ok("continue".into()),
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
        let mut pending_align: Option<usize> = None;
        let mut pending_public = false;
        let mut pending_calling_conv = CallingConv::Default;

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
                        "naked" => { pending_naked = true; pending_calling_conv = CallingConv::Naked; }
                        "macro" => { pending_macro = true; }
                        "stdcall" => { pending_calling_conv = CallingConv::Stdcall; }
                        "fastcall" => { pending_calling_conv = CallingConv::Fastcall; }
                        "cdecl" => { pending_calling_conv = CallingConv::Cdecl; }
                        "public" => { pending_public = true; }
                        "align" => {
                            self.expect_lparen()?;
                            let n = self.expect_integer()? as usize;
                            self.expect_rparen()?;
                            pending_align = Some(n);
                        }
                        "extern" => {
                            self.expect_lparen()?;
                            let ext_name = self.expect_string_or_ident()?;
                            self.expect_rparen()?;
                            self.pending_externs.push(ExternSymbol { name: ext_name, is_function: true });
                        }
                        "include" => {
                            self.expect_lparen()?;
                            let inc = self.expect_string_or_ident()?;
                            self.expect_rparen()?;
                            self.pending_includes.push(inc);
                        }
                        "includelib" => {
                            self.expect_lparen()?;
                            let lib = self.expect_string_or_ident()?;
                            self.expect_rparen()?;
                            self.pending_includelibs.push(lib);
                        }
                        "struct" => {
                            // @struct decorator: next class becomes a STRUCT
                            self.skip_newlines();
                            if matches!(self.peek(), Token::Class) {
                                self.advance(); // class
                                let struct_name = self.expect_ident()?;
                                if matches!(self.peek(), Token::Colon) { self.advance(); }
                                self.skip_newlines();
                                let mut fields = Vec::new();
                                let mut offset = 0usize;
                                while matches!(self.peek(), Token::Indent(_)) {
                                    self.advance();
                                    if let Token::Ident(fname) = self.peek().clone() {
                                        self.advance();
                                        if matches!(self.peek(), Token::Equals) {
                                            self.advance();
                                            let type_name_tok = self.expect_ident()?;
                                            let args = self.parse_call_args()?;
                                            let (size, tn, init) = match type_name_tok.as_str() {
                                                "byte" | "sbyte" | "int8" => (1, "BYTE".to_string(), args.first().map(|a| self.expr_display(a))),
                                                "word" | "sword" | "int16" => (2, "WORD".to_string(), args.first().map(|a| self.expr_display(a))),
                                                "dword" | "sdword" | "int32" => (4, "DWORD".to_string(), args.first().map(|a| self.expr_display(a))),
                                                "qword" | "sqword" | "int64" => (8, "QWORD".to_string(), args.first().map(|a| self.expr_display(a))),
                                                "real4" | "float32" => (4, "REAL4".to_string(), args.first().map(|a| self.expr_display(a))),
                                                "real8" | "float64" => (8, "REAL8".to_string(), args.first().map(|a| self.expr_display(a))),
                                                _ => (4, "DWORD".to_string(), None),
                                            };
                                            fields.push(StructField {
                                                name: fname, size, offset,
                                                type_name: tn,
                                                init_value: init.or(Some("?".to_string())),
                                            });
                                            offset += size;
                                        }
                                    }
                                    self.skip_to_newline();
                                    if matches!(self.peek(), Token::Newline) { self.advance(); }
                                }
                                self.pending_structs.push(StructDef {
                                    name: struct_name,
                                    fields,
                                    is_pub: pending_public || pending_export,
                                    alignment: pending_align.take(),
                                });
                                pending_public = false;
                            }
                        }
                        "label" => {
                            self.expect_lparen()?;
                            let label_name = self.expect_string_or_ident()?;
                            self.expect_rparen()?;
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

                    let exported = pending_export || pending_public;
                    let naked = pending_naked;
                    let _is_macro = pending_macro;
                    let cc = pending_calling_conv.clone();
                    let align = pending_align.take();
                    pending_export = false;
                    pending_naked = false;
                    pending_macro = false;
                    pending_public = false;
                    pending_calling_conv = CallingConv::Default;

                    // Parse body (indented lines)
                    let items = self.parse_function_body()?;
                    
                    let mut instructions = Vec::new();
                    let mut local_vars = Vec::new();
                    for item in items {
                        if let FunctionItem::LocalVar(v) = item {
                            local_vars.push(v);
                        } else {
                            instructions.push(item);
                        }
                    }

                    let func = Function {
                        name,
                        exported,
                        naked,
                        is_inline: false,
                        is_extern: false,
                        calling_conv: cc,
                        alignment: align,
                        params: Vec::new(),
                        local_vars,
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
                                    calling_conv: CallingConv::Default,
                                    alignment: None,
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
                        // Top-level instruction call or macro
                        let args = self.parse_call_args()?;
                        if let Some(expanded) = self.try_expand_builtin(&ident, &args)? {
                            if let Some(ref mut sec) = current_section {
                                if let Some(func) = sec.functions.last_mut() {
                                    for item in expanded {
                                        func.instructions.push(item);
                                    }
                                }
                            }
                        } else {
                            let inst = self.build_instruction(&ident, &args)?;
                            if let Some(ref mut sec) = current_section {
                                if let Some(func) = sec.functions.last_mut() {
                                    func.instructions.push(FunctionItem::Instruction(inst));
                                }
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
        program.structs = self.pending_structs.drain(..).collect();
        program.externs = self.pending_externs.drain(..).collect();
        program.includes = self.pending_includes.drain(..).collect();
        program.includelibs = self.pending_includelibs.drain(..).collect();
        Ok(program)
    }

    fn parse_function_body(&mut self) -> Result<Vec<FunctionItem>, String> {
        let mut items = Vec::new();

        loop {
            self.skip_newlines();
            if !matches!(self.peek(), Token::Indent(_)) {
                break;
            }
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
                            // Find innermost block whether it's a loop or a switch
                            // Without explicit scope tracking, this will just jump to the latest loop,
                            // OR the latest switch if we prefer. Given simple structure, let's prefer switch if we are inside one.
                            let loop_end = self.loop_stack.last().map(|(_, e)| e.clone());
                            let sw_end = self.switch_stack.last().map(|(e, _, _)| e.clone());
                            if let Some(end) = sw_end.or(loop_end) {
                                items.push(FunctionItem::Instruction(Instruction::one(Opcode::Jmp, Operand::Label(end))));
                            }
                        }
                        "continue" => {
                            if let Some((start_lbl, _)) = self.loop_stack.last() {
                                let start = start_lbl.clone();
                                items.push(FunctionItem::Instruction(Instruction::one(Opcode::Jmp, Operand::Label(start))));
                            }
                        }
                        "switch" => {
                            self.expect_lparen()?;
                            let reg_name = self.expect_string_or_ident()?;
                            self.expect_rparen()?;
                            let end_lbl = self.next_label("sw_end_");
                            self.switch_stack.push((end_lbl, reg_name, None));
                        }
                        "case" => {
                            self.expect_lparen()?;
                            let val = self.expect_integer()?;
                            self.expect_rparen()?;
                            
                            let mut reg_name = String::new();
                            let mut prev_lbl = None;
                            if let Some((_, reg, next_case)) = self.switch_stack.last_mut() {
                                reg_name = reg.clone();
                                prev_lbl = next_case.take();
                            }
                            if !reg_name.is_empty() {
                                if let Some(lbl) = prev_lbl {
                                    items.push(FunctionItem::Label(lbl));
                                }
                                let lbl = self.next_label("case_next_");
                                let reg_op = self.ident_to_operand(&reg_name);
                                items.push(FunctionItem::Instruction(Instruction::two(Opcode::Cmp, reg_op, Operand::Imm(val))));
                                items.push(FunctionItem::Instruction(Instruction::one(Opcode::Jne, Operand::Label(lbl.clone()))));
                                if let Some((_, _, next_case)) = self.switch_stack.last_mut() {
                                    *next_case = Some(lbl);
                                }
                            }
                        }
                        "local" => {
                            self.expect_lparen()?;
                            let var_name = self.expect_string_or_ident()?;
                            if matches!(self.peek(), Token::Colon) {
                                self.advance();
                            } else if matches!(self.peek(), Token::Comma) {
                                self.advance();
                            }
                            let type_name = self.expect_string_or_ident()?;
                            self.expect_rparen()?;
                            items.push(FunctionItem::LocalVar(crate::ir::LocalVar {
                                name: var_name,
                                type_name,
                            }));
                        }
                        "default" => {
                            let mut prev_lbl = None;
                            if let Some((_, _, next_case)) = self.switch_stack.last_mut() {
                                prev_lbl = next_case.take();
                            }
                            if let Some(lbl) = prev_lbl {
                                items.push(FunctionItem::Label(lbl));
                            }
                        }
                        "endswitch" => {
                            if let Some((end_lbl, _, next_case)) = self.switch_stack.pop() {
                                if let Some(lbl) = next_case {
                                    items.push(FunctionItem::Label(lbl));
                                }
                                items.push(FunctionItem::Label(end_lbl));
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
                if matches!(self.peek(), Token::LParen) {
                    let args = self.parse_call_args()?;
                    return Ok(Expr::Call { name: s, args });
                }
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
                // Use {:?} to ensure 1.0 emits as "1.0", not "1"
                Ok(Expr::Label(format!("{:?}", v)))
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
            Expr::Call { name, args } => {
                // Handle size-cast pseudo-calls: dword(x), word(x), byte(x), qword(x)
                let lower = name.to_lowercase();
                match lower.as_str() {
                    "byte" | "word" | "dword" | "qword" => {
                        if let Some(inner) = args.first() {
                            let inner_op = self.expr_to_operand(inner);
                            match inner_op {
                                Operand::Label(label_name) => {
                                    // e.g. dword(choice) → Label("[choice]") which emitter
                                    // will prepend with DWORD PTR
                                    Operand::Label(format!("[{}]", label_name))
                                }
                                Operand::Memory { base, index, scale, disp } => {
                                    Operand::Memory { base, index, scale, disp }
                                }
                                _ => inner_op,
                            }
                        } else {
                            Operand::Label(name.clone())
                        }
                    }
                    _ => Operand::Label(name.clone()),
                }
            }
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
            _ => {
                // Check if it's a known struct
                let struct_def = self.pending_structs.iter().find(|s| s.name == type_name).cloned();
                if let Some(sdef) = struct_def {
                    let mut args = Vec::new();
                    if matches!(self.peek(), Token::LParen) {
                        args = self.parse_call_args()?;
                    }
                    let fields: Vec<DataItem> = args.iter().enumerate().map(|(i, a)| {
                        // Check if there's a matching struct field definition to determine type
                        let is_float_field = sdef.fields.get(i).map(|f| {
                            matches!(f.type_name.as_str(), "REAL4" | "REAL8" | "real4" | "real8")
                        }).unwrap_or(false);
                        let is_f64 = sdef.fields.get(i).map(|f| {
                            matches!(f.type_name.as_str(), "REAL8" | "real8")
                        }).unwrap_or(false);

                        let def = if is_float_field {
                            // Float field — convert any numeric value to float
                            let fval = match a {
                                Expr::Immediate(v) => *v as f64,
                                Expr::Label(s) => s.parse::<f64>().unwrap_or(0.0),
                                _ => 0.0,
                            };
                            if is_f64 {
                                DataDef::Float64(vec![fval])
                            } else {
                                DataDef::Float32(vec![fval as f32])
                            }
                        } else {
                            match a {
                                Expr::Immediate(v) => DataDef::Dword(vec![*v as u32]),
                                _ => {
                                    let s = self.expr_display(a);
                                    if let Ok(f) = s.parse::<f32>() {
                                        DataDef::Float32(vec![f])
                                    } else {
                                        DataDef::String(s)
                                    }
                                }
                            }
                        };
                        DataItem::new(format!("field{}", i), def)
                    }).collect();
                    DataDef::Struct(type_name.to_string(), fields)
                } else {
                    return Err(format!("unknown data type: {}", type_name));
                }
            }
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
            "scanf"   => Some(self.expand_scanf(args)?),
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
            "pow"     => {
                // pow needs labels injected, handle specially
                return Ok(Some(self.expand_pow_items(args)?));
            },
            "sqrt"    => Some(self.expand_sqrt(args)?),
            "dot4"    => Some(self.expand_dot4(args)?),
            "mat4x4_mul" => Some(self.expand_mat4x4(args)?),
            "vec_add" | "vec8_add" => Some(self.expand_simd(Opcode::Vaddps, args)?),
            "vec_mul" | "vec8_mul" => Some(self.expand_simd(Opcode::Vmulps, args)?),
            "vec_sub" | "vec8_sub" => Some(self.expand_simd(Opcode::Vsubps, args)?),
            "vec_div" | "vec8_div" => Some(self.expand_simd(Opcode::Vdivps, args)?),
            "prologue" => Some(self.expand_prologue(args)?),
            "epilogue" => Some(self.expand_epilogue()?),
            "invoke" => Some(self.expand_invoke(args)?),
            _ => None,
        };
        Ok(insts.map(|v| v.into_iter().map(FunctionItem::Instruction).collect()))
    }
    
    fn expand_invoke(&self, args: &[Expr]) -> Result<Vec<Instruction>, String> {
        if args.is_empty() {
            return Err("invoke requires at least a target function".to_string());
        }
        
        let target_op = self.expr_to_operand(&args[0]);
        let mut v = Vec::new();
        
        let invoke_args = &args[1..];
        let arg_count = invoke_args.len();
        
        // C-Call ABI (Win64): 32 bytes shadow space + space for args > 4
        // Stack must be 16-byte aligned before the 'call' instruction.
        // Assuming we are 16-byte aligned here, we need to subtract an amount that parity-adjusts to 8 (so call makes it 0 again).
        // Let's allocate shadow space + extra args.
        let mut stack_alloc = 32;
        if arg_count > 4 {
            stack_alloc += (arg_count - 4) * 8;
        }
        // Ensuring the allocation maintains the 16-byte alignment rule (caller aligns to 8 for the return address push):
        // Wait, standard prologue does sub rsp, 0x28 (40 bytes = 32 + 8). That keeps it aligned.
        if stack_alloc % 16 == 0 {
            stack_alloc += 8;
        }
        
        v.push(Instruction::two(Opcode::Sub, Operand::Reg(Register::Rsp), Operand::Imm(stack_alloc as i64)));
        
        // Move extra arguments to stack > 4
        for (i, arg) in invoke_args.iter().enumerate().skip(4) {
            let offset = 32 + (i - 4) * 8;
            let arg_op = self.expr_to_operand(arg);
            let mem_op = Operand::Memory { base: Some(Register::Rsp), index: None, scale: 1, disp: offset as i64 };
            
            match arg_op {
               Operand::Label(_) => {
                   v.push(Instruction::two(Opcode::Lea, Operand::Reg(Register::Rax), arg_op));
                   v.push(Instruction::two(Opcode::Mov, mem_op, Operand::Reg(Register::Rax)));
               }
               Operand::Memory { .. } => {
                   v.push(Instruction::two(Opcode::Mov, Operand::Reg(Register::Rax), arg_op));
                   v.push(Instruction::two(Opcode::Mov, mem_op, Operand::Reg(Register::Rax)));
               }
               _ => v.push(Instruction::two(Opcode::Mov, mem_op, arg_op)),
            }
        }
        
        // Re-load first 4 arguments into RCX, RDX, R8, R9
        let regs = [Register::Rcx, Register::Rdx, Register::R8, Register::R9];
        for (i, arg) in invoke_args.iter().enumerate().take(4) {
            let arg_op = self.expr_to_operand(arg);
            match arg_op {
                Operand::Label(_) => v.push(Instruction::two(Opcode::Lea, Operand::Reg(regs[i].clone()), arg_op)),
                _ => v.push(Instruction::two(Opcode::Mov, Operand::Reg(regs[i].clone()), arg_op)),
            }
        }
        
        // Perform call
        v.push(Instruction::one(Opcode::Call, target_op));
        
        // Restore stack
        v.push(Instruction::two(Opcode::Add, Operand::Reg(Register::Rsp), Operand::Imm(stack_alloc as i64)));
        
        Ok(v)
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
        // input(buffer_label, max_size) → scanf("%s", buffer)
        let fmt_label = self.next_label("fmt_");
        self.pending_data.push(DataItem::new(fmt_label.clone(), DataDef::String("%s".to_string())));
        let buf_op = args.first().map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Imm(0));
        Ok(self.win64_call("scanf", &[Operand::Label(fmt_label), buf_op]))
    }

    fn expand_scanf(&mut self, args: &[Expr]) -> Result<Vec<Instruction>, String> {
        // scanf(fmt_string, arg1, arg2, ...) → full format support
        let ops: Vec<Operand> = args.iter().map(|a| {
            if let Expr::StringLit(s) = a {
                let label = self.next_label("fmt_");
                self.pending_data.push(DataItem::new(label.clone(), DataDef::String(s.clone())));
                Operand::Label(label)
            } else {
                self.expr_to_operand(a)
            }
        }).collect();
        Ok(self.win64_call("scanf", &ops))
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
        let dst = args.first().map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Reg(Register::Xmm(0)));
        let src = args.get(1).map(|a| self.expr_to_operand(a)).unwrap_or(dst.clone());
        Ok(vec![Instruction::two(Opcode::Sqrtss, dst, src)])
    }

    fn expand_pow_items(&mut self, args: &[Expr]) -> Result<Vec<FunctionItem>, String> {
        // pow(base, exp) → loop-based multiplication, result in rax
        let base = args.first().map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Imm(2));
        let exp = args.get(1).map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Imm(1));
        let loop_lbl = self.next_label("pow_");
        let done_lbl = self.next_label("powdone_");
        Ok(vec![
            FunctionItem::Instruction(Instruction::two(Opcode::Mov, Operand::Reg(Register::Rbx), base)),
            FunctionItem::Instruction(Instruction::two(Opcode::Mov, Operand::Reg(Register::Rcx), exp)),
            FunctionItem::Instruction(Instruction::two(Opcode::Mov, Operand::Reg(Register::Rax), Operand::Imm(1))),
            FunctionItem::Instruction(Instruction::two(Opcode::Test, Operand::Reg(Register::Rcx), Operand::Reg(Register::Rcx))),
            FunctionItem::Instruction(Instruction::one(Opcode::Je, Operand::Label(done_lbl.clone()))),
            FunctionItem::Label(loop_lbl.clone()),
            FunctionItem::Instruction(Instruction::two(Opcode::Imul, Operand::Reg(Register::Rax), Operand::Reg(Register::Rbx))),
            FunctionItem::Instruction(Instruction::one(Opcode::Dec, Operand::Reg(Register::Rcx))),
            FunctionItem::Instruction(Instruction::one(Opcode::Jne, Operand::Label(loop_lbl))),
            FunctionItem::Label(done_lbl),
        ])
    }

    fn expand_dot4(&mut self, args: &[Expr]) -> Result<Vec<Instruction>, String> {
        // dot4(dst_xmm, src_xmm) → vdpps xmm, xmm, xmm, 0FFh
        let dst = args.first().map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Reg(Register::Xmm(0)));
        let src = args.get(1).map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Reg(Register::Xmm(1)));
        Ok(vec![Instruction::new(Opcode::Vdpps, vec![dst.clone(), dst, src, Operand::Imm(0xFF)])])
    }

    fn expand_mat4x4(&mut self, args: &[Expr]) -> Result<Vec<Instruction>, String> {
        // mat4x4_mul(dst_ymm, a_ymm, b_ymm) → 4x vdpps
        let dst = args.first().map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Reg(Register::Ymm(0)));
        let a = args.get(1).map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Reg(Register::Ymm(1)));
        let b = args.get(2).map(|a| self.expr_to_operand(a)).unwrap_or(Operand::Reg(Register::Ymm(2)));
        Ok(vec![
            Instruction::new(Opcode::Vdpps, vec![dst.clone(), a.clone(), b.clone(), Operand::Imm(0xFF)]),
        ])
    }

    fn expand_prologue(&mut self, args: &[Expr]) -> Result<Vec<Instruction>, String> {
        let stack = match args.first() { Some(Expr::Immediate(v)) => *v, _ => 32 };
        Ok(vec![
            Instruction::one(Opcode::Push, Operand::Reg(Register::Rbp)),
            Instruction::two(Opcode::Mov, Operand::Reg(Register::Rbp), Operand::Reg(Register::Rsp)),
            Instruction::two(Opcode::Sub, Operand::Reg(Register::Rsp), Operand::Imm(stack)),
        ])
    }

    fn expand_epilogue(&mut self) -> Result<Vec<Instruction>, String> {
        Ok(vec![
            Instruction::zero(Opcode::Leave),
            Instruction::zero(Opcode::Ret),
        ])
    }

    // ── SIMD/AVX ──

    fn expand_simd(&mut self, op: Opcode, args: &[Expr]) -> Result<Vec<Instruction>, String> {
        let ops: Vec<Operand> = args.iter().map(|a| self.expr_to_operand(a)).collect();
        if ops.len() >= 3 {
            Ok(vec![Instruction::new(op, ops)])
        } else if ops.len() == 2 {
            Ok(vec![Instruction::new(op, vec![ops[0].clone(), ops[0].clone(), ops[1].clone()])])
        } else {
            Ok(vec![])
        }
    }

    // ── Helpers ──

    fn expr_display(&self, expr: &Expr) -> String {
        match expr {
            Expr::Immediate(v) => v.to_string(),
            Expr::Label(s) => s.clone(),
            Expr::StringLit(s) => format!("\"{}\"", s),
            Expr::Register(r) => r.clone(),
            Expr::Bool(true) => "1".to_string(),
            Expr::Bool(false) | Expr::Null => "0".to_string(),
            _ => "?".to_string(),
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
