use super::instruction::Instruction;
use super::register::Arch;

/// Data definition types
#[derive(Debug, Clone)]
pub enum DataDef {
    Byte(Vec<u8>),
    Word(Vec<u16>),
    Dword(Vec<u32>),
    Qword(Vec<u64>),
    String(String),
    WString(String),
    ReserveBytes(usize),
    ReserveWords(usize),
    ReserveDwords(usize),
    ReserveQwords(usize),
    Float32(Vec<f32>),
    Float64(Vec<f64>),
    Struct(String, Vec<DataItem>),  // struct instance
}

/// A named data item
#[derive(Debug, Clone)]
pub struct DataItem {
    pub name: String,
    pub def: DataDef,
    pub is_pub: bool,
    pub alignment: Option<usize>,
}

impl DataItem {
    pub fn new(name: String, def: DataDef) -> Self {
        Self { name, def, is_pub: false, alignment: None }
    }

    pub fn public(name: String, def: DataDef) -> Self {
        Self { name, def, is_pub: true, alignment: None }
    }
}

/// Section types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SectionKind {
    Text,
    Data,
    Bss,
    Rodata,
    Custom(String),
}

/// Struct definition in IR (for data layout)
#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<StructField>,
    pub is_pub: bool,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub size: usize,      // in bytes
    pub offset: usize,    // byte offset from start
}

impl StructDef {
    pub fn total_size(&self) -> usize {
        self.fields.iter().map(|f| f.offset + f.size).max().unwrap_or(0)
    }
}

/// Enum definition in IR (for named constants)
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<(String, i64)>,
    pub is_pub: bool,
}

/// Constant definition
#[derive(Debug, Clone)]
pub struct ConstDef {
    pub name: String,
    pub value: i64,
    pub is_pub: bool,
}

/// External symbol declaration
#[derive(Debug, Clone)]
pub struct ExternSymbol {
    pub name: String,
    pub is_function: bool,
}

/// A function/label within a text section
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub exported: bool,
    pub naked: bool,
    pub is_inline: bool,
    pub is_extern: bool,
    pub params: Vec<FuncParam>,
    pub local_vars: Vec<LocalVar>,
    pub instructions: Vec<FunctionItem>,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct FuncParam {
    pub name: String,
    pub size: usize,  // bytes
}

/// Local variable in a function
#[derive(Debug, Clone)]
pub struct LocalVar {
    pub name: String,
    pub size: usize,
    pub stack_offset: i64,
    pub is_volatile: bool,
}

/// Items inside a function
#[derive(Debug, Clone)]
pub enum FunctionItem {
    Instruction(Instruction),
    Label(String),
    Comment(String),
}

/// A section in the program
#[derive(Debug, Clone)]
pub struct Section {
    pub kind: SectionKind,
    pub functions: Vec<Function>,
    pub data: Vec<DataItem>,
}

/// The full program IR
#[derive(Debug, Clone)]
pub struct Program {
    pub arch: Arch,
    pub format: String,
    pub org: Option<u64>,
    pub sections: Vec<Section>,
    pub structs: Vec<StructDef>,
    pub enums: Vec<EnumDef>,
    pub constants: Vec<ConstDef>,
    pub externs: Vec<ExternSymbol>,
    pub uses: Vec<Vec<String>>,  // use paths
}

impl Program {
    pub fn new(arch: Arch) -> Self {
        Self {
            arch,
            format: "elf".into(),
            org: None,
            sections: Vec::new(),
            structs: Vec::new(),
            enums: Vec::new(),
            constants: Vec::new(),
            externs: Vec::new(),
            uses: Vec::new(),
        }
    }
}
