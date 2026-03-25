//! TypeScript IR 语句模块

use crate::{
    expression::Expression,
    program::{Decorator, EnumMember, InterfaceMember, Method},
    types::{TypeAnnotation, TypeParameter},
};

/// TypeScript 中间表示 - 语句
#[derive(Debug, Clone)]
pub enum Statement {
    /// 表达式语句
    Expression(Box<Expression>),
    /// 变量声明
    VariableDeclaration { name: String, ty: Option<TypeAnnotation>, initializer: Option<Box<Expression>> },
    /// 块语句
    Block(Vec<Statement>),
    /// if 语句
    If { test: Box<Expression>, consequent: Box<Statement>, alternate: Option<Box<Statement>> },
    /// while 语句
    While { test: Box<Expression>, body: Box<Statement> },
    /// for 语句
    For { init: Option<Box<Statement>>, test: Option<Box<Expression>>, update: Option<Box<Expression>>, body: Box<Statement> },
    /// return 语句
    Return(Option<Box<Expression>>),
    /// break 语句
    Break,
    /// continue 语句
    Continue,
    /// 函数声明
    FunctionDeclaration {
        /// 函数名称
        name: String,
        /// 类型参数
        type_params: Vec<TypeParameter>,
        /// 参数
        params: Vec<String>,
        /// 返回类型
        return_type: Option<TypeAnnotation>,
        /// 函数体
        body: Vec<Statement>,
        /// 装饰器列表
        decorators: Vec<Decorator>,
    },
    /// 类声明
    ClassDeclaration {
        /// 类名称
        name: String,
        /// 父类
        super_class: Option<String>,
        /// 方法列表
        methods: Vec<Method>,
        /// 装饰器列表
        decorators: Vec<Decorator>,
    },
    /// 接口声明
    InterfaceDeclaration {
        /// 接口名称
        name: String,
        /// 类型参数
        type_params: Vec<TypeParameter>,
        /// 继承的接口
        extends: Vec<String>,
        /// 接口成员
        members: Vec<InterfaceMember>,
    },
    /// 类型别名声明
    TypeAlias {
        /// 类型别名名称
        name: String,
        /// 类型参数
        type_params: Vec<TypeParameter>,
        /// 类型
        ty: TypeAnnotation,
    },
    /// 命名空间声明
    NamespaceDeclaration {
        /// 命名空间名称
        name: String,
        /// 命名空间体
        body: Vec<Statement>,
        /// 是否导出
        is_exported: bool,
    },
    /// 模块声明
    ModuleDeclaration {
        /// 模块名称
        name: String,
        /// 模块体
        body: Vec<Statement>,
    },
    /// 导入声明
    ImportDeclaration {
        /// 导入说明符列表
        specifiers: Vec<ImportSpecifier>,
        /// 导入源
        source: String,
        /// 是否仅类型导入
        is_type_only: bool,
    },
    /// 导出声明
    ExportDeclaration {
        /// 导出的声明
        declaration: Box<Statement>,
        /// 是否为默认导出
        is_default: bool,
    },
    /// 枚举声明
    EnumDeclaration {
        /// 枚举名称
        name: String,
        /// 枚举成员列表
        members: Vec<EnumMember>,
        /// 是否为 const 枚举
        is_const: bool,
        /// 是否为 declare 枚举
        is_declare: bool,
    },
}

/// 导入说明符
#[derive(Debug, Clone)]
pub struct ImportSpecifier {
    /// 本地名称
    pub local: String,
    /// 导入的名称
    pub imported: String,
    /// 是否仅类型导入
    pub is_type_only: bool,
}

/// TypeScript 中间表示 - 语句（带类型信息）
#[derive(Debug, Clone)]
pub struct TypedStatement {
    /// 语句
    pub stmt: Statement,
    /// 类型注解（如果适用）
    pub ty: Option<TypeAnnotation>,
    /// 是否为死代码
    pub is_dead: bool,
}

impl TypedStatement {
    /// 创建一个新的带类型信息的语句
    pub fn new(stmt: Statement, ty: Option<TypeAnnotation>) -> Self {
        let is_dead = stmt.is_dead_code();

        Self { stmt, ty, is_dead }
    }
}

impl Statement {
    /// 检查语句是否为死代码（不会执行的代码）
    pub fn is_dead_code(&self) -> bool {
        match self {
            Statement::Return(_) => false, // return 语句不是死代码
            Statement::Break => false,     // break 语句不是死代码
            Statement::Continue => false,  // continue 语句不是死代码
            _ => false,                    // 其他语句可能不是死代码
        }
    }

    /// 检查语句是否为变量声明
    pub fn is_variable_declaration(&self) -> bool {
        matches!(self, Statement::VariableDeclaration { .. })
    }

    /// 检查语句是否为函数声明
    pub fn is_function_declaration(&self) -> bool {
        matches!(self, Statement::FunctionDeclaration { .. })
    }

    /// 检查语句是否为类声明
    pub fn is_class_declaration(&self) -> bool {
        matches!(self, Statement::ClassDeclaration { .. })
    }
}
