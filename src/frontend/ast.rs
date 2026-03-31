/// AST nodes for Python+C hybrid ASM syntax (.pasm files)
/// Combines Python ease with C power for ASM construction

/// Top-level type representation
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    U8, U16, U32, U64,
    I8, I16, I32, I64,
    F32, F64,
    Bool,
    Void,
    Ptr(Box<Type>),          // *u8, *u32, etc.
    Array(Box<Type>, usize), // [u8; 256]
    Named(String),           // struct name, register type, etc.
}

/// Expression nodes
#[derive(Debug, Clone)]
pub enum Expr {
    // Literals
    Register(String),
    Immediate(i64),
    Label(String),
    StringLit(String),
    Bool(bool),
    Null,

    // Memory access
    Memory(Box<MemExpr>),
    Deref(Box<Expr>),            // *ptr
    AddrOf(Box<Expr>),           // &var

    // Operations (high-level → compile to ASM)
    BinOp {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },

    // Field access
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },
    NamespaceAccess {
        path: Vec<String>,       // x86_64::regs::rax
    },

    // Function/instruction call
    Call {
        name: String,
        args: Vec<Expr>,
    },

    // Type operations
    Cast {
        expr: Box<Expr>,
        ty: Type,
    },
    SizeOf(Type),
    AlignOf(Type),

    // Array index
    Index {
        array: Box<Expr>,
        index: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    BitAnd, BitOr, BitXor,
    Shl, Shr,
    Eq, Ne, Lt, Gt, Le, Ge,
    LogicAnd, LogicOr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,       // -x
    BitNot,    // ~x
    LogicNot,  // !x
}

/// Memory expression: [base + index*scale + disp]
#[derive(Debug, Clone)]
pub struct MemExpr {
    pub base: Option<String>,
    pub index: Option<String>,
    pub scale: u8,
    pub disp: i64,
}

/// Struct field definition
#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub ty: Type,
    pub offset: Option<usize>,
}

/// Enum variant
#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<i64>,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Option<Type>,
}

/// Statement nodes (inside function bodies)
#[derive(Debug, Clone)]
pub enum Stmt {
    // ASM instruction call: mov(rax, rbx) or push(rbp)
    InstructionCall {
        name: String,
        args: Vec<Expr>,
    },

    // Variable declaration: let x: u64 = 42
    Let {
        name: String,
        ty: Option<Type>,
        value: Option<Expr>,
        is_volatile: bool,
    },

    // Constant: const SIZE: u64 = 4096
    Const {
        name: String,
        ty: Option<Type>,
        value: Expr,
    },

    // Assignment: x = expr  or  x += expr
    Assign {
        target: Expr,
        op: Option<BinOp>,  // None = plain assign, Some = compound (+=, -=, etc.)
        value: Expr,
    },

    // High-level if → compiles to cmp + jcc
    If {
        condition: Expr,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
    },

    // While loop → cmp + jcc loop
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },

    // For loop → counter pattern
    For {
        init: Box<Stmt>,
        condition: Expr,
        update: Box<Stmt>,
        body: Vec<Stmt>,
    },

    // Loop (infinite, use break)
    Loop {
        body: Vec<Stmt>,
    },

    Break,
    Continue,
    Return(Option<Expr>),

    // Label
    Label(String),

    // Raw ASM block: asm { ... }
    AsmBlock {
        instructions: Vec<Stmt>,
    },

    // Expression statement (function call, etc.)
    Expr(Expr),
}

/// Top-level AST items
#[derive(Debug, Clone)]
pub enum AstNode {
    // Directives
    ArchDirective(String),
    FormatDirective(String),
    OrgDirective(u64),
    SectionDirective(String),

    // Use/Import: use x86_64::sse
    Use {
        path: Vec<String>,
    },

    // Struct definition: struct GDTEntry { limit: u16, base: u32, ... }
    StructDef {
        name: String,
        is_pub: bool,
        fields: Vec<StructField>,
    },

    // Enum definition: enum Syscall { Read = 0, Write = 1, ... }
    EnumDef {
        name: String,
        is_pub: bool,
        variants: Vec<EnumVariant>,
    },

    // Function (def or fn): supports both Python and C style
    FunctionDef {
        name: String,
        params: Vec<Param>,
        return_type: Option<Type>,
        is_pub: bool,
        is_naked: bool,
        is_inline: bool,
        is_extern: bool,
        is_unsafe: bool,
        body: Vec<Stmt>,
    },

    // Extern declaration: extern fn printf(fmt: *u8) -> i32
    ExternDecl {
        name: String,
        params: Vec<Param>,
        return_type: Option<Type>,
    },

    // Static/Global data: static msg: [u8] = "Hello\n"
    StaticData {
        name: String,
        ty: Option<Type>,
        value: DataValue,
        is_pub: bool,
    },

    // Constant: const STACK_SIZE: u64 = 0x1000
    ConstData {
        name: String,
        ty: Option<Type>,
        value: Expr,
        is_pub: bool,
    },

    // Data assignment (legacy): msg = string("Hello")
    DataAssign {
        name: String,
        value: DataValue,
    },

    // Comment
    Comment(String),
}

/// Data value types (for section .data / .bss)
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
    Expr(Expr),
}
