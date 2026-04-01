/// Token types for the .pasm Python+C hybrid ASM language

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Decorators
    At,              // @

    // Keywords — Python style
    Def,             // def

    // Keywords — C style
    Fn,              // fn
    Struct,          // struct
    Enum,            // enum
    Use,             // use
    Let,             // let
    Const,           // const
    Static,          // static
    Extern,          // extern
    Pub,             // pub
    Inline,          // inline
    Volatile,        // volatile
    Unsafe,          // unsafe
    Naked,           // naked
    Asm,             // asm

    // Control flow
    If,              // if
    Else,            // else
    While,           // while
    For,             // for
    Loop,            // loop
    Break,           // break
    Continue,        // continue
    Return,          // return

    // Type keywords
    As,              // as
    SizeOf,          // sizeof
    AlignOf,         // alignof
    TypeOf,          // typeof
    Null,            // null
    True,            // true
    False,           // false

    // Symbols
    LParen, RParen,       // ( )
    LBracket, RBracket,   // [ ]
    LBrace, RBrace,       // { }
    Comma,                // ,
    Colon,                // :
    Semi,                 // ;
    Dot,                  // .

    // Operators
    Equals,          // =
    EqEq,            // ==
    BangEq,          // !=
    Lt, Gt,          // < >
    LtEq, GtEq,     // <= >=
    Plus, Minus,     // + -
    Star, Slash,     // * /
    Percent,         // %
    Ampersand,       // &
    Pipe,            // |
    Caret,           // ^
    Bang,            // !
    Tilde,           // ~
    ShlOp, ShrOp,    // << >>
    Arrow,           // ->
    FatArrow,        // =>
    DoubleColon,     // ::

    // Compound assignment
    PlusEq, MinusEq, StarEq, SlashEq,  // += -= *= /=
    AmpEq, PipeEq, CaretEq,            // &= |= ^=
    ShlEq, ShrEq,                       // <<= >>=

    // Literals
    Ident(String),
    Integer(i64),
    HexInteger(i64),
    FloatLiteral(f64),
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

    fn peek_next(&self) -> Option<char> {
        self.input.get(self.pos + 1).copied()
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

        // Check for float: digits followed by '.' and another digit
        if self.peek() == Some('.') {
            if let Some(next) = self.input.get(self.pos + 1) {
                if next.is_ascii_digit() {
                    self.advance(); // consume '.'
                    num_str.push('.');
                    while let Some(ch) = self.peek() {
                        if ch.is_ascii_digit() || ch == '_' {
                            if ch != '_' { num_str.push(ch); }
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    let val: f64 = num_str.parse().unwrap_or(0.0);
                    return Token::FloatLiteral(val);
                }
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

    fn read_line_comment(&mut self) -> Token {
        let mut comment = String::new();
        while let Some(c) = self.peek() {
            if c == '\n' || c == '\r' { break; }
            comment.push(c);
            self.advance();
        }
        Token::Comment(comment.trim().to_string())
    }

    fn match_keyword(ident: &str) -> Option<Token> {
        match ident {
            "def"      => Some(Token::Def),
            "fn"       => Some(Token::Fn),
            "struct"   => Some(Token::Struct),
            "enum"     => Some(Token::Enum),
            "use"      => Some(Token::Use),
            "let"      => Some(Token::Let),
            "const"    => Some(Token::Const),
            "static"   => Some(Token::Static),
            "extern"   => Some(Token::Extern),
            "pub"      => Some(Token::Pub),
            "inline"   => Some(Token::Inline),
            "volatile" => Some(Token::Volatile),
            "unsafe"   => Some(Token::Unsafe),
            "naked"    => Some(Token::Naked),
            "asm"      => Some(Token::Asm),
            "if"       => Some(Token::If),
            "else"     => Some(Token::Else),
            "while"    => Some(Token::While),
            "for"      => Some(Token::For),
            "loop"     => Some(Token::Loop),
            "break"    => Some(Token::Break),
            "continue" => Some(Token::Continue),
            "return"   => Some(Token::Return),
            "as"       => Some(Token::As),
            "sizeof"   => Some(Token::SizeOf),
            "alignof"  => Some(Token::AlignOf),
            "typeof"   => Some(Token::TypeOf),
            "null"     => Some(Token::Null),
            "true"     => Some(Token::True),
            "false"    => Some(Token::False),
            _          => None,
        }
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

                // Python-style comment
                '#' => {
                    self.advance();
                    tokens.push(self.read_line_comment());
                }

                // / — could be //, /=, or plain /
                '/' => {
                    self.advance();
                    match self.peek() {
                        Some('/') => {
                            self.advance();
                            tokens.push(self.read_line_comment());
                        }
                        Some('=') => { self.advance(); tokens.push(Token::SlashEq); }
                        _ => tokens.push(Token::Slash),
                    }
                }

                '@' => { self.advance(); tokens.push(Token::At); }
                '(' => { self.advance(); tokens.push(Token::LParen); }
                ')' => { self.advance(); tokens.push(Token::RParen); }
                '[' => { self.advance(); tokens.push(Token::LBracket); }
                ']' => { self.advance(); tokens.push(Token::RBracket); }
                '{' => { self.advance(); tokens.push(Token::LBrace); }
                '}' => { self.advance(); tokens.push(Token::RBrace); }
                ',' => { self.advance(); tokens.push(Token::Comma); }
                ';' => { self.advance(); tokens.push(Token::Semi); }
                '~' => { self.advance(); tokens.push(Token::Tilde); }
                '%' => { self.advance(); tokens.push(Token::Percent); }

                // : or ::
                ':' => {
                    self.advance();
                    if self.peek() == Some(':') {
                        self.advance();
                        tokens.push(Token::DoubleColon);
                    } else {
                        tokens.push(Token::Colon);
                    }
                }

                // = or == or =>
                '=' => {
                    self.advance();
                    match self.peek() {
                        Some('=') => { self.advance(); tokens.push(Token::EqEq); }
                        Some('>') => { self.advance(); tokens.push(Token::FatArrow); }
                        _ => tokens.push(Token::Equals),
                    }
                }

                // ! or !=
                '!' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token::BangEq);
                    } else {
                        tokens.push(Token::Bang);
                    }
                }

                // < or <= or << or <<=
                '<' => {
                    self.advance();
                    match self.peek() {
                        Some('=') => { self.advance(); tokens.push(Token::LtEq); }
                        Some('<') => {
                            self.advance();
                            if self.peek() == Some('=') {
                                self.advance();
                                tokens.push(Token::ShlEq);
                            } else {
                                tokens.push(Token::ShlOp);
                            }
                        }
                        _ => tokens.push(Token::Lt),
                    }
                }

                // > or >= or >> or >>=
                '>' => {
                    self.advance();
                    match self.peek() {
                        Some('=') => { self.advance(); tokens.push(Token::GtEq); }
                        Some('>') => {
                            self.advance();
                            if self.peek() == Some('=') {
                                self.advance();
                                tokens.push(Token::ShrEq);
                            } else {
                                tokens.push(Token::ShrOp);
                            }
                        }
                        _ => tokens.push(Token::Gt),
                    }
                }

                // + or +=
                '+' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token::PlusEq);
                    } else {
                        tokens.push(Token::Plus);
                    }
                }

                // - or -= or -> or negative number
                '-' => {
                    self.advance();
                    match self.peek() {
                        Some('=') => { self.advance(); tokens.push(Token::MinusEq); }
                        Some('>') => { self.advance(); tokens.push(Token::Arrow); }
                        Some(c) if c.is_ascii_digit() => {
                            let c = self.advance().unwrap();
                            match self.read_number(c) {
                                Token::Integer(v) => tokens.push(Token::Integer(-v)),
                                Token::HexInteger(v) => tokens.push(Token::HexInteger(-v)),
                                Token::FloatLiteral(v) => tokens.push(Token::FloatLiteral(-v)),
                                _ => {}
                            }
                        }
                        _ => tokens.push(Token::Minus),
                    }
                }

                // * or *=
                '*' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token::StarEq);
                    } else {
                        tokens.push(Token::Star);
                    }
                }

                '.' => { self.advance(); tokens.push(Token::Dot); }

                // & or &=
                '&' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token::AmpEq);
                    } else {
                        tokens.push(Token::Ampersand);
                    }
                }

                // | or |=
                '|' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token::PipeEq);
                    } else {
                        tokens.push(Token::Pipe);
                    }
                }

                // ^ or ^=
                '^' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token::CaretEq);
                    } else {
                        tokens.push(Token::Caret);
                    }
                }

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
                    match Self::match_keyword(&ident) {
                        Some(kw) => tokens.push(kw),
                        None => tokens.push(Token::Ident(ident)),
                    }
                }

                _ => { self.advance(); }
            }
        }

        tokens.push(Token::Eof);
        tokens
    }
}
