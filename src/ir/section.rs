use super::instruction::Instruction;
use super::register::Arch;

/// Data definition types
#[derive(Debug, Clone)]
pub enum DataDef {
    Byte(Vec<u8>),
    Word(Vec<u16>),
    Dword(Vec<u32>),
    Qword(Vec<u64>),
    String(String),       // null-terminated
    WString(String),      // UTF-16 wide string
    ReserveBytes(usize),  // resb
    ReserveWords(usize),  // resw
    ReserveDwords(usize), // resd
    ReserveQwords(usize), // resq
}

/// A named data item
#[derive(Debug, Clone)]
pub struct DataItem {
    pub name: String,
    pub def: DataDef,
}

/// Section types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SectionKind {
    Text,
    Data,
    Bss,
    Custom(String),
}

/// A function/label within a text section
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub exported: bool,
    pub naked: bool,
    pub instructions: Vec<FunctionItem>,
}

/// Items inside a function
#[derive(Debug, Clone)]
pub enum FunctionItem {
    Instruction(Instruction),
    Label(String),
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
}

impl Program {
    pub fn new(arch: Arch) -> Self {
        Self {
            arch,
            format: "elf".into(),
            org: None,
            sections: Vec::new(),
        }
    }
}
