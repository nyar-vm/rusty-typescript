/// 二元操作符
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    StrictEq,
    StrictNeq,
    Gt,
    Gte,
    Lt,
    Lte,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    UShr,
    Pow,
}

/// 一元操作符
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Neg,
    Pos,
    BitNot,
    Inc,
    Dec,
    TypeOf,
    Void,
    Delete,
}

/// VM 指令
#[derive(Debug, Clone)]
pub enum Instruction {
    PushUndefined,
    PushNull,
    PushBoolean(bool),
    PushNumber(f64),
    PushString(String),

    LoadVariable(String),
    StoreVariable(String),

    CreateObject,
    GetProperty,
    SetProperty,

    CreateArray,
    GetElement,
    SetElement,

    CreateFunction(String, u32),
    SetFunctionBody(Vec<Instruction>),
    Call(u32),
    Return,

    CreateClass(String),
    AddMethod(String),
    SetClassBody(Vec<Instruction>),

    CreateTypeAlias(String),
    CreateInterface(String),

    BinaryOp(BinaryOp),

    UnaryOp(UnaryOp),

    Jump(u32),
    JumpIfFalse(u32),
    JumpLoop(u32),

    Pop,
    Dup,
    Swap,

    LoadLocal(usize),
    StoreLocal(usize),

    TryStart { handler_ip: usize, finally_ip: Option<usize>, exception_var: Option<String> },
    TryEnd,
    Throw,

    ImportModule { name: String, alias: Option<String> },
    Export { name: String },
}
