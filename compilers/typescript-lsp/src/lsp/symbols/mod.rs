//! 符号表管理模块
//!
//! 提供符号存储、查询和作用域管理功能。

mod collector;

pub use collector::SymbolCollector;

use core::range::Range;
use std::collections::HashMap;

/// 符号种类
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    /// 变量
    Variable,
    /// 函数
    Function,
    /// 类
    Class,
    /// 接口
    Interface,
    /// 类型别名
    TypeAlias,
    /// 枚举
    Enum,
    /// 枚举成员
    EnumMember,
    /// 参数
    Parameter,
    /// 属性
    Property,
    /// 方法
    Method,
    /// 模块
    Module,
    /// 命名空间
    Namespace,
}

/// 符号信息
#[derive(Clone, Debug)]
pub struct Symbol {
    /// 符号名称
    pub name: String,
    /// 符号类型（类型注解或推断类型）
    pub type_annotation: Option<String>,
    /// 符号种类
    pub kind: SymbolKind,
    /// 符号在文档中的位置范围
    pub range: Range<usize>,
    /// 符号选择范围（名称所在范围）
    pub selection_range: Range<usize>,
    /// 作用域深度
    pub scope_depth: usize,
    /// 是否已导出
    pub is_exported: bool,
    /// 函数签名（仅函数和方法有效）
    pub signature: Option<String>,
    /// 文档 URI
    pub uri: Option<String>,
}

impl Symbol {
    /// 创建新的符号
    pub fn new(name: String, kind: SymbolKind, range: Range<usize>) -> Self {
        Self {
            name,
            type_annotation: None,
            kind,
            range,
            selection_range: range,
            scope_depth: 0,
            is_exported: false,
            signature: None,
            uri: None,
        }
    }

    /// 设置类型注解
    pub fn with_type(mut self, type_annotation: String) -> Self {
        self.type_annotation = Some(type_annotation);
        self
    }

    /// 设置作用域深度
    pub fn with_scope_depth(mut self, depth: usize) -> Self {
        self.scope_depth = depth;
        self
    }

    /// 设置导出状态
    pub fn with_exported(mut self, is_exported: bool) -> Self {
        self.is_exported = is_exported;
        self
    }

    /// 设置函数签名
    pub fn with_signature(mut self, signature: String) -> Self {
        self.signature = Some(signature);
        self
    }

    /// 设置选择范围
    pub fn with_selection_range(mut self, range: Range<usize>) -> Self {
        self.selection_range = range;
        self
    }

    /// 设置文档 URI
    pub fn with_uri(mut self, uri: String) -> Self {
        self.uri = Some(uri);
        self
    }
}

/// 作用域信息
#[derive(Clone, Debug)]
pub struct Scope {
    /// 作用域深度
    pub depth: usize,
    /// 作用域起始位置
    pub start_offset: usize,
    /// 作用域结束位置（None 表示尚未结束）
    pub end_offset: Option<usize>,
    /// 作用域类型
    pub scope_type: ScopeType,
    /// 父作用域索引
    pub parent_index: Option<usize>,
}

/// 作用域类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeType {
    /// 全局作用域
    Global,
    /// 函数作用域
    Function,
    /// 块级作用域
    Block,
    /// 类作用域
    Class,
    /// 接口作用域
    Interface,
    /// 模块作用域
    Module,
}

impl Scope {
    /// 创建新作用域
    pub fn new(depth: usize, start_offset: usize, scope_type: ScopeType) -> Self {
        Self {
            depth,
            start_offset,
            end_offset: None,
            scope_type,
            parent_index: None,
        }
    }

    /// 设置父作用域
    pub fn with_parent(mut self, parent_index: usize) -> Self {
        self.parent_index = Some(parent_index);
        self
    }

    /// 结束作用域
    pub fn end(&mut self, end_offset: usize) {
        self.end_offset = Some(end_offset);
    }

    /// 检查位置是否在作用域内
    pub fn contains(&self, offset: usize) -> bool {
        offset >= self.start_offset
            && self.end_offset.map_or(true, |end| offset <= end)
    }
}

/// 符号表
#[derive(Clone, Debug, Default)]
pub struct SymbolTable {
    /// 所有符号
    symbols: Vec<Symbol>,
    /// 按名称索引的符号
    name_index: HashMap<String, Vec<usize>>,
    /// 作用域栈
    scopes: Vec<Scope>,
    /// 当前作用域深度
    current_depth: usize,
}

impl SymbolTable {
    /// 创建新的符号表
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
            name_index: HashMap::new(),
            scopes: vec![Scope::new(0, 0, ScopeType::Global)],
            current_depth: 0,
        }
    }

    /// 进入新作用域
    pub fn enter_scope(&mut self, start_offset: usize, scope_type: ScopeType) {
        self.current_depth += 1;
        let parent_index = self.scopes.len() - 1;
        let scope = Scope::new(self.current_depth, start_offset, scope_type)
            .with_parent(parent_index);
        self.scopes.push(scope);
    }

    /// 退出当前作用域
    pub fn exit_scope(&mut self, end_offset: usize) {
        if self.current_depth > 0 {
            if let Some(scope) = self.scopes.last_mut() {
                scope.end(end_offset);
            }
            self.current_depth -= 1;
        }
    }

    /// 添加符号
    pub fn add_symbol(&mut self, mut symbol: Symbol) {
        symbol.scope_depth = self.current_depth;
        let index = self.symbols.len();
        self.symbols.push(symbol.clone());
        self.name_index
            .entry(symbol.name.clone())
            .or_default()
            .push(index);
    }

    /// 根据名称查找符号（当前作用域及父作用域）
    pub fn find_by_name(&self, name: &str) -> Option<&Symbol> {
        self.name_index
            .get(name)
            .and_then(|indices| {
                // 首先查找导出的符号
                for &idx in indices.iter().rev() {
                    if let Some(symbol) = self.symbols.get(idx) {
                        if symbol.is_exported && symbol.scope_depth <= self.current_depth {
                            return Some(symbol);
                        }
                    }
                }
                
                // 如果没有找到导出的符号，查找非导出的符号
                for &idx in indices.iter().rev() {
                    if let Some(symbol) = self.symbols.get(idx) {
                        if symbol.scope_depth <= self.current_depth {
                            return Some(symbol);
                        }
                    }
                }
                None
            })
    }

    /// 查找所有匹配名称的符号
    pub fn find_all_by_name(&self, name: &str) -> Vec<&Symbol> {
        self.name_index
            .get(name)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&idx| self.symbols.get(idx))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 获取当前作用域的所有符号
    pub fn current_scope_symbols(&self) -> Vec<&Symbol> {
        self.symbols
            .iter()
            .filter(|s| s.scope_depth <= self.current_depth)
            .collect()
    }

    /// 获取指定作用域深度的所有符号
    pub fn symbols_at_depth(&self, depth: usize) -> Vec<&Symbol> {
        self.symbols
            .iter()
            .filter(|s| s.scope_depth == depth)
            .collect()
    }

    /// 获取所有符号
    pub fn all_symbols(&self) -> &[Symbol] {
        &self.symbols
    }

    /// 获取当前作用域深度
    pub fn current_depth(&self) -> usize {
        self.current_depth
    }

    /// 清空符号表
    pub fn clear(&mut self) {
        self.symbols.clear();
        self.name_index.clear();
        self.scopes = vec![Scope::new(0, 0, ScopeType::Global)];
        self.current_depth = 0;
    }

    /// 根据位置查找所在作用域
    pub fn find_scope_at(&self, offset: usize) -> Option<&Scope> {
        self.scopes
            .iter()
            .filter(|s| s.contains(offset))
            .max_by_key(|s| s.depth)
    }

    /// 获取指定位置的可见符号
    pub fn visible_symbols_at(&self, offset: usize) -> Vec<&Symbol> {
        let max_depth = self
            .find_scope_at(offset)
            .map_or(0, |s| s.depth);
        
        self.symbols
            .iter()
            .filter(|s| s.scope_depth <= max_depth)
            .collect()
    }

    /// 查找类或接口的成员
    pub fn find_members(&self, type_name: &str) -> Vec<&Symbol> {
        self.symbols
            .iter()
            .filter(|s| {
                matches!(s.kind, SymbolKind::Property | SymbolKind::Method)
                    && s.type_annotation.as_deref() == Some(type_name)
            })
            .collect()
    }
}


