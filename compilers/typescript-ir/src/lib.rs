#![warn(missing_docs)]

use typescript_types::TsValue;

mod optimization;

// Reexport oak-typescript AST
pub use oak_typescript::ast;

pub use optimization::*;

/// TypeScript 中间表示 - 表达式
#[derive(Debug, Clone)]
pub enum Expression {
    /// 字面量
    Literal(TsValue),
    /// 标识符
    Identifier(String),
    /// 二元表达式
    Binary { left: Box<Expression>, op: BinaryOp, right: Box<Expression> },
    /// 一元表达式
    Unary { op: UnaryOp, expr: Box<Expression> },
    /// 函数调用
    Call { callee: Box<Expression>, args: Vec<Expression> },
    /// 成员访问
    Member { object: Box<Expression>, property: Box<Expression> },
    /// 数组访问
    Index { object: Box<Expression>, index: Box<Expression> },
    /// 对象字面量
    Object(Vec<(String, Expression)>),
    /// 数组字面量
    Array(Vec<Expression>),
    /// 赋值表达式
    Assignment { left: Box<Expression>, op: AssignmentOp, right: Box<Expression> },
    /// 条件表达式
    Conditional { test: Box<Expression>, consequent: Box<Expression>, alternate: Box<Expression> },
    /// 函数表达式
    Function { params: Vec<String>, body: Vec<Statement> },
    /// 箭头函数
    ArrowFunction { params: Vec<String>, body: Box<Expression> },
}

/// TypeScript 中间表示 - 表达式（带类型信息）
#[derive(Debug, Clone)]
pub struct TypedExpression {
    /// 表达式
    pub expr: Expression,
    /// 类型注解
    pub ty: TypeAnnotation,
    /// 是否为常量
    pub is_constant: bool,
    /// 常量值（如果是常量）
    pub constant_value: Option<TsValue>,
}

impl TypedExpression {
    /// 创建一个新的带类型信息的表达式
    pub fn new(expr: Expression, ty: TypeAnnotation) -> Self {
        let constant_value = expr.eval();
        let is_constant = constant_value.is_some();

        Self { expr, ty, is_constant, constant_value }
    }

    /// 优化表达式
    pub fn optimize(&self) -> Self {
        let optimized_expr = match &self.expr {
            Expression::Binary { left, op, right } => {
                let optimized_left = TypedExpression::new(*left.clone(), self.ty.clone()).optimize();
                let optimized_right = TypedExpression::new(*right.clone(), self.ty.clone()).optimize();

                if optimized_left.is_constant && optimized_right.is_constant {
                    if let (Some(left_val), Some(right_val)) = (optimized_left.constant_value, optimized_right.constant_value) {
                        let result = Expression::eval_binary_op(left_val, op.clone(), right_val);
                        Expression::Literal(result)
                    }
                    else {
                        Expression::Binary {
                            left: Box::new(optimized_left.expr),
                            op: op.clone(),
                            right: Box::new(optimized_right.expr),
                        }
                    }
                }
                else {
                    Expression::Binary {
                        left: Box::new(optimized_left.expr),
                        op: op.clone(),
                        right: Box::new(optimized_right.expr),
                    }
                }
            }
            Expression::Unary { op, expr: inner_expr } => {
                let optimized_inner = TypedExpression::new(*inner_expr.clone(), self.ty.clone()).optimize();

                if optimized_inner.is_constant {
                    if let Some(inner_val) = optimized_inner.constant_value {
                        let result = Expression::eval_unary_op(op.clone(), inner_val);
                        Expression::Literal(result)
                    }
                    else {
                        Expression::Unary { op: op.clone(), expr: Box::new(optimized_inner.expr) }
                    }
                }
                else {
                    Expression::Unary { op: op.clone(), expr: Box::new(optimized_inner.expr) }
                }
            }
            _ => self.expr.clone(),
        };

        Self::new(optimized_expr, self.ty.clone())
    }
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

/// TypeScript 中间表示 - 程序（带类型信息）
#[derive(Debug, Clone)]
pub struct TypedProgram {
    /// 语句
    pub statements: Vec<TypedStatement>,
    /// 类型环境
    pub type_env: std::collections::HashMap<String, TypeAnnotation>,
}

impl TypedProgram {
    /// 创建一个新的带类型信息的程序
    pub fn new(statements: Vec<TypedStatement>) -> Self {
        Self { statements, type_env: std::collections::HashMap::new() }
    }
}

/// TypeScript 中间表示 - 控制流图节点
#[derive(Debug, Clone)]
pub struct CFGNode {
    /// 节点 ID
    pub id: usize,
    /// 语句
    pub statements: Vec<Statement>,
    /// 后继节点
    pub successors: Vec<usize>,
    /// 前驱节点
    pub predecessors: Vec<usize>,
    /// 是否为入口节点
    pub is_entry: bool,
    /// 是否为出口节点
    pub is_exit: bool,
}

impl CFGNode {
    /// 创建一个新的控制流图节点
    pub fn new(id: usize) -> Self {
        Self { id, statements: Vec::new(), successors: Vec::new(), predecessors: Vec::new(), is_entry: false, is_exit: false }
    }
}

/// TypeScript 中间表示 - 控制流图
#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    /// 节点
    pub nodes: Vec<CFGNode>,
    /// 入口节点 ID
    pub entry_id: usize,
    /// 出口节点 ID
    pub exit_id: usize,
}

impl ControlFlowGraph {
    /// 创建一个新的控制流图
    pub fn new() -> Self {
        let mut entry_node = CFGNode::new(0);
        let mut exit_node = CFGNode::new(1);
        entry_node.is_entry = true;
        exit_node.is_exit = true;

        Self { nodes: vec![entry_node, exit_node], entry_id: 0, exit_id: 1 }
    }

    /// 添加节点
    pub fn add_node(&mut self) -> usize {
        let id = self.nodes.len();
        self.nodes.push(CFGNode::new(id));
        id
    }

    /// 添加边
    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.nodes[from].successors.push(to);
        self.nodes[to].predecessors.push(from);
    }
}

impl Expression {
    /// 尝试计算表达式的值（用于常量折叠）
    pub fn eval(&self) -> Option<TsValue> {
        match self {
            Expression::Literal(value) => Some(value.clone()),
            Expression::Binary { left, op, right } => {
                if let (Some(left_val), Some(right_val)) = (left.eval(), right.eval()) {
                    Some(Self::eval_binary_op(left_val, op.clone(), right_val))
                }
                else {
                    None
                }
            }
            Expression::Unary { op, expr } => {
                if let Some(expr_val) = expr.eval() {
                    Some(Self::eval_unary_op(op.clone(), expr_val))
                }
                else {
                    None
                }
            }
            Expression::Conditional { test, consequent, alternate } => {
                if let Some(test_val) = test.eval() {
                    if test_val.to_boolean() { consequent.eval() } else { alternate.eval() }
                }
                else {
                    None
                }
            }
            Expression::Array(elements) => {
                let mut evaluated = Vec::new();
                for elem in elements {
                    if let Some(val) = elem.eval() {
                        evaluated.push(val);
                    }
                    else {
                        return None;
                    }
                }
                Some(TsValue::Array(evaluated))
            }
            Expression::Object(properties) => {
                let mut evaluated = Vec::new();
                for (key, value) in properties {
                    if let Some(val) = value.eval() {
                        evaluated.push((key.clone(), val));
                    }
                    else {
                        return None;
                    }
                }
                Some(TsValue::Object(evaluated))
            }
            _ => None, // 其他表达式类型无法在编译时求值
        }
    }

    /// 计算二元操作
    fn eval_binary_op(left: TsValue, op: BinaryOp, right: TsValue) -> TsValue {
        match op {
            BinaryOp::Add => {
                if left.is_string() || right.is_string() {
                    TsValue::String(format!("{}{}", left.to_string(), right.to_string()))
                }
                else {
                    TsValue::Number(left.to_number() + right.to_number())
                }
            }
            BinaryOp::Sub => TsValue::Number(left.to_number() - right.to_number()),
            BinaryOp::Mul => TsValue::Number(left.to_number() * right.to_number()),
            BinaryOp::Div => TsValue::Number(left.to_number() / right.to_number()),
            BinaryOp::Mod => TsValue::Number(left.to_number() % right.to_number()),
            BinaryOp::Eq => TsValue::Boolean(left.to_string() == right.to_string()),
            BinaryOp::Neq => TsValue::Boolean(left.to_string() != right.to_string()),
            BinaryOp::StrictEq => TsValue::Boolean(
                left.to_string() == right.to_string()
                    && left.is_number() == right.is_number()
                    && left.is_string() == right.is_string()
                    && left.is_boolean() == right.is_boolean(),
            ),
            BinaryOp::StrictNeq => TsValue::Boolean(
                left.to_string() != right.to_string()
                    || left.is_number() != right.is_number()
                    || left.is_string() != right.is_string()
                    || left.is_boolean() != right.is_boolean(),
            ),
            BinaryOp::Gt => TsValue::Boolean(left.to_number() > right.to_number()),
            BinaryOp::Gte => TsValue::Boolean(left.to_number() >= right.to_number()),
            BinaryOp::Lt => TsValue::Boolean(left.to_number() < right.to_number()),
            BinaryOp::Lte => TsValue::Boolean(left.to_number() <= right.to_number()),
            BinaryOp::And => TsValue::Boolean(left.to_boolean() && right.to_boolean()),
            BinaryOp::Or => TsValue::Boolean(left.to_boolean() || right.to_boolean()),
            BinaryOp::BitAnd => TsValue::Number((left.to_number() as i64 & right.to_number() as i64) as f64),
            BinaryOp::BitOr => TsValue::Number((left.to_number() as i64 | right.to_number() as i64) as f64),
            BinaryOp::BitXor => TsValue::Number((left.to_number() as i64 ^ right.to_number() as i64) as f64),
            BinaryOp::Shl => TsValue::Number(((left.to_number() as i64) << (right.to_number() as u32)) as f64),
            BinaryOp::Shr => TsValue::Number(((left.to_number() as i64) >> (right.to_number() as u32)) as f64),
            BinaryOp::UShr => TsValue::Number(((left.to_number() as u64) >> (right.to_number() as u32)) as f64),
            BinaryOp::Pow => TsValue::Number(left.to_number().powf(right.to_number())),
        }
    }

    /// 计算一元操作
    fn eval_unary_op(op: UnaryOp, expr: TsValue) -> TsValue {
        match op {
            UnaryOp::Not => TsValue::Boolean(!expr.to_boolean()),
            UnaryOp::Neg => TsValue::Number(-expr.to_number()),
            UnaryOp::Pos => TsValue::Number(expr.to_number()),
            UnaryOp::BitNot => TsValue::Number(!(expr.to_number() as i64) as f64),
            UnaryOp::Inc => TsValue::Number(expr.to_number() + 1.0),
            UnaryOp::Dec => TsValue::Number(expr.to_number() - 1.0),
            UnaryOp::TypeOf => TsValue::String(if expr.is_undefined() {
                "undefined".to_string()
            }
            else if expr.is_null() {
                "object".to_string()
            }
            else if expr.is_boolean() {
                "boolean".to_string()
            }
            else if expr.is_number() {
                "number".to_string()
            }
            else if expr.is_string() {
                "string".to_string()
            }
            else if expr.is_function() {
                "function".to_string()
            }
            else if expr.is_array() {
                "object".to_string()
            }
            else {
                "object".to_string()
            }),
            UnaryOp::Void => TsValue::Undefined,
            UnaryOp::Delete => TsValue::Boolean(true), // 简化处理
        }
    }
}

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
    FunctionDeclaration { name: String, params: Vec<String>, return_type: Option<TypeAnnotation>, body: Vec<Statement> },
    /// 类声明
    ClassDeclaration { name: String, super_class: Option<String>, methods: Vec<Method> },
    /// 接口声明
    InterfaceDeclaration { name: String, extends: Vec<String>, members: Vec<InterfaceMember> },
    /// 类型别名声明
    TypeAlias { name: String, ty: TypeAnnotation },
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
    Generic(String, Vec<TypeAnnotation>),
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

/// 方法
#[derive(Debug, Clone)]
pub struct Method {
    /// 方法名
    pub name: String,
    /// 参数
    pub params: Vec<String>,
    /// 返回类型
    pub return_type: Option<TypeAnnotation>,
    /// 方法体
    pub body: Vec<Statement>,
}

/// 接口成员
#[derive(Debug, Clone)]
pub enum InterfaceMember {
    /// 属性
    Property { name: String, ty: TypeAnnotation, optional: bool },
    /// 方法
    Method { name: String, params: Vec<(String, TypeAnnotation)>, return_type: TypeAnnotation, optional: bool },
}

/// 程序
#[derive(Debug, Clone)]
pub struct Program {
    /// 语句
    pub statements: Vec<Statement>,
}

/// 函数
#[derive(Debug, Clone)]
pub struct Function {
    /// 函数名
    pub name: String,
    /// 参数
    pub params: Vec<String>,
    /// 返回类型
    pub return_type: Option<TypeAnnotation>,
    /// 函数体
    pub body: Vec<Statement>,
}

/// 类
#[derive(Debug, Clone)]
pub struct Class {
    /// 类名
    pub name: String,
    /// 父类
    pub super_class: Option<String>,
    /// 方法
    pub methods: Vec<Method>,
}

/// 模块
#[derive(Debug, Clone)]
pub struct Module {
    /// 模块名
    pub name: String,
    /// 语句
    pub statements: Vec<Statement>,
}

/// IR 访问者 trait
pub trait Visitor<T> {
    /// 访问表达式
    fn visit_expression(&mut self, expr: &Expression) -> T;
    /// 访问语句
    fn visit_statement(&mut self, stmt: &Statement) -> T;
    /// 访问类型注解
    fn visit_type_annotation(&mut self, ty: &TypeAnnotation) -> T;
    /// 访问程序
    fn visit_program(&mut self, program: &Program) -> T;
    /// 访问模块
    fn visit_module(&mut self, module: &Module) -> T;
}

/// 简单的 IR 遍历实现
pub struct SimpleVisitor {
    pub result: Vec<String>,
}

impl SimpleVisitor {
    pub fn new() -> Self {
        Self { result: Vec::new() }
    }
}

impl Visitor<()> for SimpleVisitor {
    fn visit_expression(&mut self, expr: &Expression) -> () {
        match expr {
            Expression::Literal(_) => self.result.push("Literal".to_string()),
            Expression::Identifier(name) => self.result.push(format!("Identifier: {}", name)),
            Expression::Binary { left, op, right } => {
                self.result.push("Binary".to_string());
                self.visit_expression(left);
                self.visit_expression(right);
            }
            Expression::Unary { op, expr } => {
                self.result.push("Unary".to_string());
                self.visit_expression(expr);
            }
            Expression::Call { callee, args } => {
                self.result.push("Call".to_string());
                self.visit_expression(callee);
                for arg in args {
                    self.visit_expression(arg);
                }
            }
            Expression::Member { object, property } => {
                self.result.push("Member".to_string());
                self.visit_expression(object);
                self.visit_expression(property);
            }
            Expression::Index { object, index } => {
                self.result.push("Index".to_string());
                self.visit_expression(object);
                self.visit_expression(index);
            }
            Expression::Object(properties) => {
                self.result.push("Object".to_string());
                for (_, value) in properties {
                    self.visit_expression(value);
                }
            }
            Expression::Array(elements) => {
                self.result.push("Array".to_string());
                for elem in elements {
                    self.visit_expression(elem);
                }
            }
            Expression::Assignment { left, op, right } => {
                self.result.push("Assignment".to_string());
                self.visit_expression(left);
                self.visit_expression(right);
            }
            Expression::Conditional { test, consequent, alternate } => {
                self.result.push("Conditional".to_string());
                self.visit_expression(test);
                self.visit_expression(consequent);
                self.visit_expression(alternate);
            }
            Expression::Function { params, body } => {
                self.result.push("Function".to_string());
                for stmt in body {
                    self.visit_statement(stmt);
                }
            }
            Expression::ArrowFunction { params, body } => {
                self.result.push("ArrowFunction".to_string());
                self.visit_expression(body);
            }
        }
    }

    fn visit_statement(&mut self, stmt: &Statement) -> () {
        match stmt {
            Statement::Expression(expr) => {
                self.result.push("ExpressionStatement".to_string());
                self.visit_expression(expr);
            }
            Statement::VariableDeclaration { name, ty, initializer } => {
                self.result.push(format!("VariableDeclaration: {}", name));
                if let Some(init) = initializer {
                    self.visit_expression(init);
                }
            }
            Statement::Block(statements) => {
                self.result.push("Block".to_string());
                for stmt in statements {
                    self.visit_statement(stmt);
                }
            }
            Statement::If { test, consequent, alternate } => {
                self.result.push("If".to_string());
                self.visit_expression(test);
                self.visit_statement(consequent);
                if let Some(alt) = alternate {
                    self.visit_statement(alt);
                }
            }
            Statement::While { test, body } => {
                self.result.push("While".to_string());
                self.visit_expression(test);
                self.visit_statement(body);
            }
            Statement::For { init, test, update, body } => {
                self.result.push("For".to_string());
                if let Some(init_stmt) = init {
                    self.visit_statement(init_stmt);
                }
                if let Some(test_expr) = test {
                    self.visit_expression(test_expr);
                }
                if let Some(update_expr) = update {
                    self.visit_expression(update_expr);
                }
                self.visit_statement(body);
            }
            Statement::Return(expr) => {
                self.result.push("Return".to_string());
                if let Some(expr) = expr {
                    self.visit_expression(expr);
                }
            }
            Statement::Break => self.result.push("Break".to_string()),
            Statement::Continue => self.result.push("Continue".to_string()),
            Statement::FunctionDeclaration { name, params, return_type, body } => {
                self.result.push(format!("FunctionDeclaration: {}", name));
                for stmt in body {
                    self.visit_statement(stmt);
                }
            }
            Statement::ClassDeclaration { name, super_class, methods } => {
                self.result.push(format!("ClassDeclaration: {}", name));
                for method in methods {
                    for stmt in &method.body {
                        self.visit_statement(stmt);
                    }
                }
            }
            Statement::InterfaceDeclaration { name, extends, members } => {
                self.result.push(format!("InterfaceDeclaration: {}", name));
            }
            Statement::TypeAlias { name, ty } => {
                self.result.push(format!("TypeAlias: {}", name));
                self.visit_type_annotation(ty);
            }
        }
    }

    fn visit_type_annotation(&mut self, ty: &TypeAnnotation) -> () {
        match ty {
            TypeAnnotation::Primitive(_) => self.result.push("PrimitiveType".to_string()),
            TypeAnnotation::Array(elem_ty) => {
                self.result.push("ArrayType".to_string());
                self.visit_type_annotation(elem_ty);
            }
            TypeAnnotation::Object(properties) => {
                self.result.push("ObjectType".to_string());
                for (_, ty) in properties {
                    self.visit_type_annotation(ty);
                }
            }
            TypeAnnotation::Union(types) => {
                self.result.push("UnionType".to_string());
                for ty in types {
                    self.visit_type_annotation(ty);
                }
            }
            TypeAnnotation::Intersection(types) => {
                self.result.push("IntersectionType".to_string());
                for ty in types {
                    self.visit_type_annotation(ty);
                }
            }
            TypeAnnotation::Generic(name, args) => {
                self.result.push(format!("GenericType: {}", name));
                for arg in args {
                    self.visit_type_annotation(arg);
                }
            }
            TypeAnnotation::Function { params, return_type } => {
                self.result.push("FunctionType".to_string());
                for param in params {
                    self.visit_type_annotation(param);
                }
                self.visit_type_annotation(return_type);
            }
            TypeAnnotation::TypeReference(name) => {
                self.result.push(format!("TypeReference: {}", name));
            }
            TypeAnnotation::Any => self.result.push("AnyType".to_string()),
            TypeAnnotation::Unknown => self.result.push("UnknownType".to_string()),
            TypeAnnotation::Void => self.result.push("VoidType".to_string()),
            TypeAnnotation::Never => self.result.push("NeverType".to_string()),
            TypeAnnotation::Tuple(types) => {
                self.result.push("TupleType".to_string());
                for ty in types {
                    self.visit_type_annotation(ty);
                }
            }
        }
    }

    fn visit_program(&mut self, program: &Program) -> () {
        self.result.push("Program".to_string());
        for stmt in &program.statements {
            self.visit_statement(stmt);
        }
    }

    fn visit_module(&mut self, module: &Module) -> () {
        self.result.push(format!("Module: {}", module.name));
        for stmt in &module.statements {
            self.visit_statement(stmt);
        }
    }
}

/// 从 oak-typescript AST 转换为 IR
pub mod ast_to_ir {
    use super::*;
    use oak_typescript::ast;

    /// 将 oak-typescript Expression 转换为 IR Expression
    pub fn expression(ast_expr: &ast::Expression) -> Expression {
        match &*ast_expr.kind {
            ast::ExpressionKind::Identifier(name) => Expression::Identifier(name.clone()),
            ast::ExpressionKind::NumericLiteral(value) => Expression::Literal(TsValue::Number(*value)),
            ast::ExpressionKind::StringLiteral(value) => Expression::Literal(TsValue::String(value.clone())),
            ast::ExpressionKind::BooleanLiteral(value) => Expression::Literal(TsValue::Boolean(*value)),
            ast::ExpressionKind::NullLiteral => Expression::Literal(TsValue::Null),
            ast::ExpressionKind::BinaryExpression { left, operator, right } => {
                let binary_op = match operator.as_str() {
                    "+" => BinaryOp::Add,
                    "-" => BinaryOp::Sub,
                    "*" => BinaryOp::Mul,
                    "/" => BinaryOp::Div,
                    "%" => BinaryOp::Mod,
                    "==" => BinaryOp::Eq,
                    "!=" => BinaryOp::Neq,
                    "===" => BinaryOp::StrictEq,
                    "!==" => BinaryOp::StrictNeq,
                    ">" => BinaryOp::Gt,
                    ">=" => BinaryOp::Gte,
                    "<" => BinaryOp::Lt,
                    "<=" => BinaryOp::Lte,
                    "&&" => BinaryOp::And,
                    "||" => BinaryOp::Or,
                    "&" => BinaryOp::BitAnd,
                    "|" => BinaryOp::BitOr,
                    "^" => BinaryOp::BitXor,
                    "<<" => BinaryOp::Shl,
                    ">>" => BinaryOp::Shr,
                    ">>>" => BinaryOp::UShr,
                    "**" => BinaryOp::Pow,
                    _ => BinaryOp::Add, // 默认为加法操作
                };
                Expression::Binary { left: Box::new(expression(left)), op: binary_op, right: Box::new(expression(right)) }
            }
            ast::ExpressionKind::UnaryExpression { operator, argument } => {
                let unary_op = match operator.as_str() {
                    "!" => UnaryOp::Not,
                    "-" => UnaryOp::Neg,
                    "+" => UnaryOp::Pos,
                    "~" => UnaryOp::BitNot,
                    "typeof" => UnaryOp::TypeOf,
                    "void" => UnaryOp::Void,
                    "delete" => UnaryOp::Delete,
                    _ => UnaryOp::Not, // 默认为逻辑非
                };
                Expression::Unary { op: unary_op, expr: Box::new(expression(argument)) }
            }
            ast::ExpressionKind::MemberExpression { object, property, computed, optional } => {
                Expression::Member { object: Box::new(expression(object)), property: Box::new(expression(property)) }
            }
            ast::ExpressionKind::CallExpression { func, args } => {
                Expression::Call { callee: Box::new(expression(func)), args: args.iter().map(expression).collect() }
            }
            ast::ExpressionKind::AssignmentExpression { left, operator, right } => {
                let assignment_op = match operator.as_str() {
                    "=" => AssignmentOp::Assign,
                    "+=" => AssignmentOp::AddAssign,
                    "-=" => AssignmentOp::SubAssign,
                    "*=" => AssignmentOp::MulAssign,
                    "/=" => AssignmentOp::DivAssign,
                    "%=" => AssignmentOp::ModAssign,
                    "&=" => AssignmentOp::BitAndAssign,
                    "|=" => AssignmentOp::BitOrAssign,
                    "^=" => AssignmentOp::BitXorAssign,
                    "<<=" => AssignmentOp::ShlAssign,
                    ">>=" => AssignmentOp::ShrAssign,
                    ">>>=" => AssignmentOp::UShrAssign,
                    "**=" => AssignmentOp::PowAssign,
                    _ => AssignmentOp::Assign, // 默认为简单赋值
                };
                Expression::Assignment {
                    left: Box::new(expression(left)),
                    op: assignment_op,
                    right: Box::new(expression(right)),
                }
            }
            ast::ExpressionKind::ConditionalExpression { test, consequent, alternate } => Expression::Conditional {
                test: Box::new(expression(test)),
                consequent: Box::new(expression(consequent)),
                alternate: Box::new(expression(alternate)),
            },
            ast::ExpressionKind::ObjectLiteral { properties } => {
                let mut ir_properties = vec![];
                for prop in properties {
                    if let ast::ObjectProperty::Property { name, value, .. } = prop {
                        ir_properties.push((name.clone(), expression(value)));
                    }
                }
                Expression::Object(ir_properties)
            }
            ast::ExpressionKind::ArrayLiteral { elements } => Expression::Array(elements.iter().map(expression).collect()),
            ast::ExpressionKind::FunctionExpression { name, params, body, .. } => {
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                let ir_body: Vec<Statement> = body.iter().map(statement).collect();
                Expression::Function { params: param_names, body: ir_body }
            }
            ast::ExpressionKind::ArrowFunction { params, body, .. } => {
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                // 箭头函数的 body 是 Statement，需要特殊处理
                match &**body {
                    ast::Statement::ExpressionStatement(expr_stmt) => {
                        Expression::ArrowFunction { params: param_names, body: Box::new(expression(&expr_stmt.expression)) }
                    }
                    _ => {
                        // 对于其他类型的 body，创建一个块语句
                        let ir_body = statement(body);
                        Expression::Function { params: param_names, body: vec![ir_body] }
                    }
                }
            }
            _ => Expression::Literal(TsValue::Undefined), // 其他表达式类型暂时不处理
        }
    }

    /// 将 oak-typescript Statement 转换为 IR Statement
    pub fn statement(ast_stmt: &ast::Statement) -> Statement {
        match ast_stmt {
            ast::Statement::ExpressionStatement(expr_stmt) => {
                Statement::Expression(Box::new(expression(&expr_stmt.expression)))
            }
            ast::Statement::VariableDeclaration(var_decl) => {
                Statement::VariableDeclaration {
                    name: var_decl.name.clone(),
                    ty: None, // 暂时不处理类型注解
                    initializer: var_decl.value.as_ref().map(|expr| Box::new(expression(expr))),
                }
            }
            ast::Statement::BlockStatement(block_stmt) => {
                Statement::Block(block_stmt.statements.iter().map(statement).collect())
            }
            ast::Statement::IfStatement(if_stmt) => Statement::If {
                test: Box::new(expression(&if_stmt.test)),
                consequent: Box::new(statement(&if_stmt.consequent)),
                alternate: if_stmt.alternate.as_ref().map(|stmt| Box::new(statement(stmt))),
            },
            ast::Statement::WhileStatement(while_stmt) => {
                Statement::While { test: Box::new(expression(&while_stmt.test)), body: Box::new(statement(&while_stmt.body)) }
            }
            ast::Statement::ForStatement(for_stmt) => Statement::For {
                init: for_stmt.initializer.as_ref().map(|stmt| Box::new(statement(stmt))),
                test: for_stmt.test.as_ref().map(|expr| Box::new(expression(expr))),
                update: for_stmt.incrementor.as_ref().map(|expr| Box::new(expression(expr))),
                body: Box::new(statement(&for_stmt.body)),
            },
            ast::Statement::ReturnStatement(return_stmt) => {
                Statement::Return(return_stmt.argument.as_ref().map(|expr| Box::new(expression(expr))))
            }
            ast::Statement::BreakStatement(_) => Statement::Break,
            ast::Statement::ContinueStatement(_) => Statement::Continue,
            ast::Statement::FunctionDeclaration(func_decl) => {
                let param_names: Vec<String> = func_decl.params.iter().map(|p| p.name.clone()).collect();
                let ir_body: Vec<Statement> = func_decl.body.iter().map(statement).collect();
                Statement::FunctionDeclaration {
                    name: func_decl.name.clone(),
                    params: param_names,
                    return_type: None, // 暂时不处理返回类型
                    body: ir_body,
                }
            }
            _ => Statement::Expression(Box::new(Expression::Literal(TsValue::Undefined))), // 其他语句类型暂时不处理
        }
    }

    /// 将 oak-typescript TypeScriptRoot 转换为 IR Program
    pub fn program(ast_program: &ast::TypeScriptRoot) -> Program {
        Program { statements: ast_program.statements.iter().map(statement).collect() }
    }
}
