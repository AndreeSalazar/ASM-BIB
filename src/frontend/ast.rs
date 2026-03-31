/// AST nodes for Python-like ASM syntax (.pasm files)

#[derive(Debug, Clone)]
pub enum AstNode {
    /// @arch('x86_64')
    ArchDirective(String),
    /// @format('pe')
    FormatDirective(String),
    /// @org(0x7C00)
    OrgDirective(u64),
    /// @section('.text')
    SectionDirective(String),
    /// @export
    ExportAttribute,
    /// @naked
    NakedAttribute,
    /// @label('name')
    Label(String),
    /// @macro
    MacroAttribute,
    /// def name():
    FunctionDef {
        name: String,
        exported: bool,
        naked: bool,
        is_macro: bool,
        body: Vec<AstNode>,
    },
    /// mov(rax, rbx) — instruction call
    InstructionCall {
        name: String,
        args: Vec<Expr>,
    },
    /// msg = string("Hello\n")
    DataAssign {
        name: String,
        value: DataValue,
    },
    /// Comment line
    Comment(String),
}

/// Expression in an argument
#[derive(Debug, Clone)]
pub enum Expr {
    Register(String),
    Immediate(i64),
    Label(String),
    StringLit(String),
    Memory(Box<MemExpr>),
}

/// Memory expression: [base + index*scale + disp]
#[derive(Debug, Clone)]
pub struct MemExpr {
    pub base: Option<String>,
    pub index: Option<String>,
    pub scale: u8,
    pub disp: i64,
}

/// Data value types
#[derive(Debug, Clone)]
pub enum DataValue {
    Byte(Vec<i64>),
    Word(Vec<i64>),
    Dword(Vec<i64>),
    Qword(Vec<i64>),
    String(String),
    WString(String),
    ResBytes(usize),
    ResWords(usize),
    ResDwords(usize),
    ResQwords(usize),
}
