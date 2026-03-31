use super::ast::*;
use super::lexer::Token;
use crate::ir::{
    Arch, DataDef, DataItem, Function, FunctionItem, Instruction, Opcode, Operand,
    Program, Register, Section, SectionKind,
};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
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
                    if dir == "label" {
                        self.expect_lparen()?;
                        let name = self.expect_string_or_ident()?;
                        self.expect_rparen()?;
                        items.push(FunctionItem::Label(name));
                    } else {
                        self.skip_to_newline();
                    }
                }
                Token::Ident(_) => {
                    let name = if let Token::Ident(s) = self.advance() { s } else { unreachable!() };

                    if matches!(self.peek(), Token::LParen) {
                        let args = self.parse_call_args()?;
                        let inst = self.build_instruction(&name, &args)?;
                        items.push(FunctionItem::Instruction(inst));
                    } else if matches!(self.peek(), Token::Equals) {
                        // skip in-function data for now
                        self.skip_to_newline();
                    } else {
                        // bare instruction with no args (like ret, nop, etc.)
                        if let Some(opcode) = Opcode::from_str(&name) {
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
            Expr::Memory(mem) => Operand::Memory {
                base: mem.base.as_ref().and_then(|r| Register::from_str(r)),
                index: mem.index.as_ref().and_then(|r| Register::from_str(r)),
                scale: mem.scale,
                disp: mem.disp,
            },
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
            _ => return Err(format!("unknown data type: {}", type_name)),
        };

        Ok(DataItem { name: name.to_string(), def })
    }

    fn expect_lparen(&mut self) -> Result<(), String> {
        match self.advance() {
            Token::LParen => Ok(()),
            other => Err(format!("expected '(', got {:?}", other)),
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
