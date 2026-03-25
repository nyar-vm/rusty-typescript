//! TypeScript IR 程序结构模块

use crate::{
    expression::Expression,
    statement::Statement,
    types::{TypeAnnotation, TypeParameter},
};

/// 装饰器目标类型
#[derive(Debug, Clone)]
pub enum DecoratorTarget {
    /// 类装饰器
    Class,
    /// 方法装饰器，包含方法名
    Method(String),
    /// 属性装饰器，包含属性名
    Property(String),
    /// 参数装饰器，包含方法名和参数索引
    Parameter(String, usize),
}

/// 装饰器
#[derive(Debug, Clone)]
pub struct Decorator {
    /// 装饰器名称
    pub name: String,
    /// 装饰器参数列表
    pub arguments: Vec<Expression>,
    /// 装饰器目标类型
    pub target: DecoratorTarget,
}

/// 方法
#[derive(Debug, Clone)]
pub struct Method {
    /// 方法名
    pub name: String,
    /// 类型参数
    pub type_params: Vec<TypeParameter>,
    /// 参数
    pub params: Vec<String>,
    /// 返回类型
    pub return_type: Option<TypeAnnotation>,
    /// 方法体
    pub body: Vec<Statement>,
    /// 装饰器列表
    pub decorators: Vec<Decorator>,
}

/// 接口成员
#[derive(Debug, Clone)]
pub enum InterfaceMember {
    /// 属性
    Property { name: String, ty: TypeAnnotation, optional: bool },
    /// 方法
    Method { name: String, params: Vec<(String, TypeAnnotation)>, return_type: TypeAnnotation, optional: bool },
}

/// 枚举成员
#[derive(Debug, Clone)]
pub struct EnumMember {
    /// 成员名称
    pub name: String,
    /// 成员值（可选）
    pub value: Option<Expression>,
    /// 是否为字符串枚举成员
    pub is_string: bool,
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

/// TypeScript 中间表示 - 程序（带类型信息）
#[derive(Debug, Clone)]
pub struct TypedProgram {
    /// 语句
    pub statements: Vec<crate::statement::TypedStatement>,
    /// 类型环境
    pub type_env: std::collections::HashMap<String, TypeAnnotation>,
    /// 函数类型环境
    pub function_env: std::collections::HashMap<String, (Vec<TypeAnnotation>, Option<TypeAnnotation>)>,
    /// 类型别名环境
    pub type_aliases: std::collections::HashMap<String, TypeAnnotation>,
    /// 接口环境
    pub interfaces: std::collections::HashMap<String, Vec<(String, TypeAnnotation, bool)>>,
}

impl TypedProgram {
    /// 创建一个新的带类型信息的程序
    pub fn new(statements: Vec<crate::statement::TypedStatement>) -> Self {
        Self {
            statements,
            type_env: std::collections::HashMap::new(),
            function_env: std::collections::HashMap::new(),
            type_aliases: std::collections::HashMap::new(),
            interfaces: std::collections::HashMap::new(),
        }
    }

    /// 从普通 Program 创建 TypedProgram
    pub fn from_program(program: &Program) -> Self {
        let typed_statements: Vec<crate::statement::TypedStatement> =
            program.statements.iter().map(|stmt| crate::statement::TypedStatement::new(stmt.clone(), None)).collect();
        Self::new(typed_statements)
    }
}

/// TypeScript 中间表示 - 模块（带类型信息）
#[derive(Debug, Clone)]
pub struct TypedModule {
    /// 模块名称
    pub name: String,
    /// 语句
    pub statements: Vec<crate::statement::TypedStatement>,
    /// 导出的符号
    pub exports: Vec<String>,
}

impl TypedModule {
    /// 创建一个新的带类型信息的模块
    pub fn new(name: String, statements: Vec<crate::statement::TypedStatement>) -> Self {
        Self { name, statements, exports: Vec::new() }
    }
}

/// TypeScript 中间表示 - 函数（带类型信息）
#[derive(Debug, Clone)]
pub struct TypedFunction {
    /// 函数名称
    pub name: String,
    /// 参数类型
    pub param_types: Vec<TypeAnnotation>,
    /// 返回类型
    pub return_type: Option<TypeAnnotation>,
    /// 函数体
    pub body: Vec<crate::statement::TypedStatement>,
    /// 局部变量类型环境
    pub local_env: std::collections::HashMap<String, TypeAnnotation>,
}

impl TypedFunction {
    /// 创建一个新的带类型信息的函数
    pub fn new(
        name: String,
        param_types: Vec<TypeAnnotation>,
        return_type: Option<TypeAnnotation>,
        body: Vec<crate::statement::TypedStatement>,
    ) -> Self {
        Self { name, param_types, return_type, body, local_env: std::collections::HashMap::new() }
    }
}

/// TypeScript 中间表示 - 控制流图节点
#[derive(Debug, Clone)]
pub struct CFGNode {
    /// 节点 ID
    pub id: usize,
    /// 节点中的语句
    pub statements: Vec<Statement>,
    /// 后继节点 ID
    pub successors: Vec<usize>,
    /// 前驱节点 ID
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
    /// 节点列表
    pub nodes: Vec<CFGNode>,
    /// 入口节点 ID
    pub entry_id: usize,
    /// 出口节点 ID
    pub exit_id: usize,
}

impl ControlFlowGraph {
    /// 创建一个新的控制流图
    pub fn new() -> Self {
        let mut entry = CFGNode::new(0);
        entry.is_entry = true;
        let mut exit = CFGNode::new(1);
        exit.is_exit = true;

        Self { nodes: vec![entry, exit], entry_id: 0, exit_id: 1 }
    }

    /// 从语句列表构建控制流图
    pub fn from_statements(statements: &[crate::statement::Statement]) -> Self {
        let mut cfg = Self::new();
        let mut current_node = cfg.entry_id;

        for stmt in statements {
            let new_node_id = cfg.nodes.len();
            let mut new_node = CFGNode::new(new_node_id);
            new_node.statements.push(stmt.clone());
            new_node.predecessors.push(current_node);
            cfg.nodes.push(new_node);

            // 更新前驱节点的后继
            if let Some(node) = cfg.nodes.get_mut(current_node) {
                node.successors.push(new_node_id);
            }

            current_node = new_node_id;
        }

        // 连接到最后一个节点到出口
        if let Some(node) = cfg.nodes.get_mut(current_node) {
            node.successors.push(cfg.exit_id);
        }
        if let Some(exit) = cfg.nodes.get_mut(cfg.exit_id) {
            exit.predecessors.push(current_node);
        }

        cfg
    }

    /// 执行到达定义分析（简化实现）
    pub fn reaching_definitions_analysis(&mut self) {
        // 简化实现，暂不执行实际分析
    }

    /// 执行活跃变量分析（简化实现）
    pub fn live_variables_analysis(&mut self) {
        // 简化实现，暂不执行实际分析
    }
}

impl Default for ControlFlowGraph {
    fn default() -> Self {
        Self::new()
    }
}
