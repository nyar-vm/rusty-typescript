//! 编译上下文模块
//!
//! 提供编译过程中使用的上下文信息，包括编译选项、符号表、类型环境等。

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};
use typescript_ir::{Program, TypeAnnotation, TypedProgram};
use typescript_types::TsError;

// 导入 num_cpus 库用于获取系统 CPU 核心数
use num_cpus;

/// 编译选项
#[derive(Debug, Clone)]
pub struct CompilationOptions {
    /// 是否启用严格模式
    pub strict: bool,
    /// 是否启用增量编译
    pub incremental: bool,
    /// 是否启用并行编译
    pub parallel: bool,
    /// 并行编译的线程数
    pub parallel_threads: usize,
    /// 是否生成源映射
    pub source_map: bool,
    /// 目标 ECMAScript 版本
    pub target: String,
    /// 模块系统
    pub module: String,
    /// 输出目录
    pub out_dir: Option<PathBuf>,
}

impl Default for CompilationOptions {
    fn default() -> Self {
        Self {
            strict: false,
            incremental: true,
            parallel: true,
            parallel_threads: num_cpus::get(),
            source_map: true,
            target: "es2020".to_string(),
            module: "esnext".to_string(),
            out_dir: None,
        }
    }
}

/// 符号表项
#[derive(Debug, Clone)]
pub struct Symbol {
    /// 符号名称
    pub name: String,
    /// 符号类型
    pub ty: TypeAnnotation,
    /// 是否为常量
    pub is_const: bool,
    /// 是否为导出
    pub is_exported: bool,
    /// 定义位置
    pub definition: Option<(String, usize, usize)>,
}

/// 符号表
#[derive(Debug, Clone)]
pub struct SymbolTable {
    /// 符号映射
    symbols: HashMap<String, Symbol>,
}

impl SymbolTable {
    /// 创建新的符号表
    pub fn new() -> Self {
        Self { symbols: HashMap::new() }
    }

    /// 添加符号
    pub fn add_symbol(&mut self, symbol: Symbol) {
        self.symbols.insert(symbol.name.clone(), symbol);
    }

    /// 获取符号
    pub fn get_symbol(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    /// 检查符号是否存在
    pub fn has_symbol(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    /// 移除符号
    pub fn remove_symbol(&mut self, name: &str) {
        self.symbols.remove(name);
    }

    /// 合并符号表
    pub fn merge(&mut self, other: &SymbolTable) {
        for (name, symbol) in &other.symbols {
            self.symbols.insert(name.clone(), symbol.clone());
        }
    }
}

/// 编译上下文
#[derive(Debug, Clone)]
pub struct CompilationContext {
    /// 编译选项
    pub options: CompilationOptions,
    /// 全局符号表
    pub global_symbols: SymbolTable,
    /// 当前作用域的符号表
    pub local_symbols: Vec<SymbolTable>,
    /// 类型环境
    pub type_env: HashMap<String, TypeAnnotation>,
    /// 函数环境
    pub function_env: HashMap<String, (Vec<TypeAnnotation>, Option<TypeAnnotation>)>,
    /// 类型别名
    pub type_aliases: HashMap<String, TypeAnnotation>,
    /// 接口定义
    pub interfaces: HashMap<String, Vec<(String, TypeAnnotation, bool)>>,
    /// 依赖项
    pub dependencies: HashSet<String>,
    /// 当前编译的文件路径
    pub current_file: Option<PathBuf>,
    /// 编译阶段
    pub current_stage: crate::compiler::CompilationStage,
    /// 错误列表
    pub errors: Vec<TsError>,
}

impl CompilationContext {
    /// 创建新的编译上下文
    pub fn new(options: CompilationOptions) -> Self {
        Self {
            options,
            global_symbols: SymbolTable::new(),
            local_symbols: vec![SymbolTable::new()],
            type_env: HashMap::new(),
            function_env: HashMap::new(),
            type_aliases: HashMap::new(),
            interfaces: HashMap::new(),
            dependencies: HashSet::new(),
            current_file: None,
            current_stage: crate::compiler::CompilationStage::LexicalAnalysis,
            errors: Vec::new(),
        }
    }

    /// 进入新的作用域
    pub fn enter_scope(&mut self) {
        self.local_symbols.push(SymbolTable::new());
    }

    /// 退出当前作用域
    pub fn exit_scope(&mut self) {
        if !self.local_symbols.is_empty() {
            self.local_symbols.pop();
        }
    }

    /// 添加符号到当前作用域
    pub fn add_symbol(&mut self, symbol: Symbol) {
        if let Some(last) = self.local_symbols.last_mut() {
            last.add_symbol(symbol);
        }
    }

    /// 查找符号
    pub fn find_symbol(&self, name: &str) -> Option<&Symbol> {
        // 从局部作用域开始查找
        for symbols in self.local_symbols.iter().rev() {
            if let Some(symbol) = symbols.get_symbol(name) {
                return Some(symbol);
            }
        }
        // 查找全局作用域
        self.global_symbols.get_symbol(name)
    }

    /// 添加错误
    pub fn add_error(&mut self, error: TsError) {
        self.errors.push(error);
    }

    /// 检查是否有错误
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// 获取所有错误
    pub fn get_errors(&self) -> &[TsError] {
        &self.errors
    }

    /// 设置当前编译阶段
    pub fn set_stage(&mut self, stage: crate::compiler::CompilationStage) {
        self.current_stage = stage;
    }

    /// 添加依赖项
    pub fn add_dependency(&mut self, dependency: String) {
        self.dependencies.insert(dependency);
    }

    /// 获取所有依赖项
    pub fn get_dependencies(&self) -> &HashSet<String> {
        &self.dependencies
    }
}

impl Default for CompilationContext {
    fn default() -> Self {
        Self::new(CompilationOptions::default())
    }
}
