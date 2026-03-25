//! TypeScript IR 类型定义模块

/// TypeScript 中间表示 - 类型注解
#[derive(Debug, Clone)]
pub enum TypeAnnotation {
    /// 基本类型
    Primitive(PrimitiveType),
    /// 数组类型
    Array(Box<TypeAnnotation>),
    /// 对象类型
    Object(Vec<(String, TypeAnnotation)>),
    /// 联合类型
    Union(Vec<TypeAnnotation>),
    /// 交叉类型
    Intersection(Vec<TypeAnnotation>),
    /// 泛型类型
    Generic { name: String, args: Vec<TypeAnnotation> },
    /// 函数类型
    Function { params: Vec<TypeAnnotation>, return_type: Box<TypeAnnotation> },
    /// 类型引用
    TypeReference(String),
    /// 任何类型
    Any,
    /// 未知类型
    Unknown,
    /// 空类型
    Void,
    /// never 类型
    Never,
    /// 元组类型
    Tuple(Vec<TypeAnnotation>),
}

/// 类型参数
#[derive(Debug, Clone)]
pub struct TypeParameter {
    /// 类型参数名称
    pub name: String,
    /// 约束类型
    pub constraint: Option<TypeAnnotation>,
    /// 默认类型
    pub default: Option<TypeAnnotation>,
}

/// 基本类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveType {
    /// 布尔类型
    Boolean,
    /// 数字类型
    Number,
    /// 字符串类型
    String,
    /// 符号类型
    Symbol,
    /// 大整数类型
    BigInt,
    /// null 类型
    Null,
    /// undefined 类型
    Undefined,
}

/// 二元操作符
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    /// 加法
    Add,
    /// 减法
    Sub,
    /// 乘法
    Mul,
    /// 除法
    Div,
    /// 取模
    Mod,
    /// 等于
    Eq,
    /// 不等于
    Neq,
    /// 严格等于
    StrictEq,
    /// 严格不等于
    StrictNeq,
    /// 大于
    Gt,
    /// 大于等于
    Gte,
    /// 小于
    Lt,
    /// 小于等于
    Lte,
    /// 逻辑与
    And,
    /// 逻辑或
    Or,
    /// 位与
    BitAnd,
    /// 位或
    BitOr,
    /// 位异或
    BitXor,
    /// 左移
    Shl,
    /// 右移
    Shr,
    /// 无符号右移
    UShr,
    /// 幂运算
    Pow,
}

/// 一元操作符
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    /// 逻辑非
    Not,
    /// 负号
    Neg,
    /// 正号
    Pos,
    /// 位非
    BitNot,
    /// 递增
    Inc,
    /// 递减
    Dec,
    /// typeof
    TypeOf,
    /// void
    Void,
    /// delete
    Delete,
}

/// 赋值操作符
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssignmentOp {
    /// 简单赋值
    Assign,
    /// 加法赋值
    AddAssign,
    /// 减法赋值
    SubAssign,
    /// 乘法赋值
    MulAssign,
    /// 除法赋值
    DivAssign,
    /// 取模赋值
    ModAssign,
    /// 位与赋值
    BitAndAssign,
    /// 位或赋值
    BitOrAssign,
    /// 位异或赋值
    BitXorAssign,
    /// 左移赋值
    ShlAssign,
    /// 右移赋值
    ShrAssign,
    /// 无符号右移赋值
    UShrAssign,
    /// 幂运算赋值
    PowAssign,
}
