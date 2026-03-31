/// Token types for the .pasm Python-like ASM language

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Decorators
    At,              // @
    // Keywords
    Def,             // def
    // Symbols
    LParen,          // (
    RParen,          // )
    LBracket,        // [
    RBracket,        // ]
    Comma,           // ,
    Colon,           // :
    Equals,          // =
    Plus,            // +
    Minus,           // -
    Star,            // *
    Dot,             // .
    // Literals
    Ident(String),
    Integer(i64),
    HexInteger(i64),
    StringLiteral(String),
    // Structure
    Newline,
    Indent(usize),
    Dedent,
    // Special
    Comment(String),
    Eof,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    pub line: usize,
    pub col: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            input: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.input.get(self.pos).copied()?;
        self.pos += 1;
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    fn skip_whitespace_on_line(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_string(&mut self, quote: char) -> String {
        let mut s = String::new();
        loop {
            match self.advance() {
                Some('\\') => {
                    match self.advance() {
                        Some('n') => s.push('\n'),
                        Some('t') => s.push('\t'),
                        Some('r') => s.push('\r'),
                        Some('0') => s.push('\0'),
                        Some('\\') => s.push('\\'),
                        Some(c) if c == quote => s.push(c),
                        Some(c) => { s.push('\\'); s.push(c); }
                        None => break,
                    }
                }
                Some(c) if c == quote => break,
                Some(c) => s.push(c),
                None => break,
            }
        }
        s
    }

    fn read_number(&mut self, first: char) -> Token {
        let mut num_str = String::new();
        num_str.push(first);

        if first == '0' {
            if let Some('x') | Some('X') = self.peek() {
                self.advance();
                let mut hex = String::new();
                while let Some(ch) = self.peek() {
                    if ch.is_ascii_hexdigit() || ch == '_' {
                        if ch != '_' { hex.push(ch); }
                        self.advance();
                    } else {
                        break;
                    }
                }
                let val = i64::from_str_radix(&hex, 16).unwrap_or(0);
                return Token::HexInteger(val);
            }
            if let Some('b') | Some('B') = self.peek() {
                self.advance();
                let mut bin = String::new();
                while let Some(ch) = self.peek() {
                    if ch == '0' || ch == '1' || ch == '_' {
                        if ch != '_' { bin.push(ch); }
                        self.advance();
                    } else {
                        break;
                    }
                }
                let val = i64::from_str_radix(&bin, 2).unwrap_or(0);
                return Token::HexInteger(val);
            }
        }

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '_' {
                if ch != '_' { num_str.push(ch); }
                self.advance();
            } else {
                break;
            }
        }
        let val: i64 = num_str.parse().unwrap_or(0);
        Token::Integer(val)
    }

    fn read_ident(&mut self, first: char) -> String {
        let mut s = String::new();
        s.push(first);
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' || ch == '.' {
                s.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        s
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        loop {
            // Count indentation at start of line
            if self.col == 1 {
                let mut indent = 0;
                while let Some(ch) = self.peek() {
                    match ch {
                        ' ' => { indent += 1; self.advance(); }
                        '\t' => { indent += 4; self.advance(); }
                        _ => break,
                    }
                }
                if let Some('\n') | Some('\r') = self.peek() {
                    // blank line
                    self.advance();
                    if let Some('\n') = self.peek() { self.advance(); }
                    continue;
                }
                if self.peek().is_none() { break; }
                if indent > 0 {
                    tokens.push(Token::Indent(indent));
                }
            }

            self.skip_whitespace_on_line();

            let ch = match self.peek() {
                Some(c) => c,
                None => break,
            };

            match ch {
                '\n' => {
                    self.advance();
                    tokens.push(Token::Newline);
                }
                '\r' => {
                    self.advance();
                    if let Some('\n') = self.peek() { self.advance(); }
                    tokens.push(Token::Newline);
                }
                '#' => {
                    self.advance();
                    let mut comment = String::new();
                    while let Some(c) = self.peek() {
                        if c == '\n' || c == '\r' { break; }
                        comment.push(c);
                        self.advance();
                    }
                    tokens.push(Token::Comment(comment.trim().to_string()));
                }
                '@' => { self.advance(); tokens.push(Token::At); }
                '(' => { self.advance(); tokens.push(Token::LParen); }
                ')' => { self.advance(); tokens.push(Token::RParen); }
                '[' => { self.advance(); tokens.push(Token::LBracket); }
                ']' => { self.advance(); tokens.push(Token::RBracket); }
                ',' => { self.advance(); tokens.push(Token::Comma); }
                ':' => { self.advance(); tokens.push(Token::Colon); }
                '=' => { self.advance(); tokens.push(Token::Equals); }
                '+' => { self.advance(); tokens.push(Token::Plus); }
                '-' => {
                    self.advance();
                    if let Some(c) = self.peek() {
                        if c.is_ascii_digit() {
                            let c = self.advance().unwrap();
                            match self.read_number(c) {
                                Token::Integer(v) => tokens.push(Token::Integer(-v)),
                                Token::HexInteger(v) => tokens.push(Token::HexInteger(-v)),
                                _ => {}
                            }
                        } else {
                            tokens.push(Token::Minus);
                        }
                    } else {
                        tokens.push(Token::Minus);
                    }
                }
                '*' => { self.advance(); tokens.push(Token::Star); }
                '.' => { self.advance(); tokens.push(Token::Dot); }
                '"' | '\'' => {
                    let q = self.advance().unwrap();
                    let s = self.read_string(q);
                    tokens.push(Token::StringLiteral(s));
                }
                c if c.is_ascii_digit() => {
                    self.advance();
                    let tok = self.read_number(c);
                    tokens.push(tok);
                }
                c if c.is_alphabetic() || c == '_' => {
                    self.advance();
                    let ident = self.read_ident(c);
                    match ident.as_str() {
                        "def" => tokens.push(Token::Def),
                        _ => tokens.push(Token::Ident(ident)),
                    }
                }
                _ => { self.advance(); }
            }
        }

        tokens.push(Token::Eof);
        tokens
    }
}
