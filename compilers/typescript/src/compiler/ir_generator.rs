//! 中间表示生成器模块
//!
//! 负责将抽象语法树转换为中间表示（IR）。

use crate::compiler::{CompilationResult, CompilationStage};
use std::{collections::HashMap, sync::Arc};
use typescript_ir::Program;

use typescript_types::TsError;

/// 中间表示生成器
#[derive(Debug, Clone)]
pub struct IRGenerator {
    /// 当前编译阶段
    current_stage: CompilationStage,
    /// IR生成缓存，使用AST哈希作为键
    ir_cache: HashMap<u64, Arc<Program>>,
    /// 缓存大小限制
    cache_size_limit: usize,
}

impl IRGenerator {
    /// 创建新的中间表示生成器
    pub fn new() -> Self {
        Self { current_stage: CompilationStage::IRGeneration, ir_cache: HashMap::new(), cache_size_limit: 500 }
    }

    /// 计算哈希值
    fn compute_hash(source: &str) -> u64 {
        use std::{
            collections::hash_map::DefaultHasher,
            hash::{Hash, Hasher},
        };
        let mut hasher = DefaultHasher::new();
        source.hash(&mut hasher);
        hasher.finish()
    }

    /// 生成中间表示
    pub fn generate(&mut self, source: &str) -> CompilationResult<Program> {
        self.current_stage = CompilationStage::IRGeneration;

        // 计算哈希值
        let hash = Self::compute_hash(source);

        // 检查缓存
        if let Some(ir) = self.ir_cache.get(&hash) {
            return CompilationResult::Success(ir.as_ref().clone());
        }

        // 生成基本的 IR 结构
        let statements = self.parse_source(source);
        let ir = Program { statements };

        // 存储到缓存
        let ir_arc = Arc::new(ir.clone());
        self.ir_cache.insert(hash, ir_arc);

        // 管理缓存大小
        self.manage_cache_size();

        CompilationResult::Success(ir)
    }

    /// 解析源代码并生成语句
    fn parse_source(&self, source: &str) -> Vec<typescript_ir::Statement> {
        let mut statements = Vec::new();

        // 简单的词法分析和语法分析
        let tokens = self.tokenize(source);
        let mut token_iter = tokens.into_iter().peekable();

        while token_iter.peek().is_some() {
            if let Some(stmt) = self.parse_statement(&mut token_iter) {
                statements.push(stmt);
            }
        }

        statements
    }

    /// 词法分析，将源代码转换为 tokens
    fn tokenize(&self, source: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();

        for c in source.chars() {
            if c.is_whitespace() {
                if !current_token.is_empty() {
                    tokens.push(current_token);
                    current_token = String::new();
                }
            }
            else if "+-*/=<>!&|(){}[];:,.".contains(c) {
                if !current_token.is_empty() {
                    tokens.push(current_token);
                    current_token = String::new();
                }
                tokens.push(c.to_string());
            }
            else {
                current_token.push(c);
            }
        }

        if !current_token.is_empty() {
            tokens.push(current_token);
        }

        tokens
    }

    /// 解析语句
    fn parse_statement(
        &self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<String>>,
    ) -> Option<typescript_ir::Statement> {
        let token = tokens.peek()?;

        match token.as_str() {
            "let" | "const" | "var" => self.parse_variable_declaration(tokens),
            "function" => self.parse_function_declaration(tokens),
            "class" => self.parse_class_declaration(tokens),
            "if" => self.parse_if_statement(tokens),
            "while" => self.parse_while_statement(tokens),
            "for" => self.parse_for_statement(tokens),
            "return" => self.parse_return_statement(tokens),
            "break" => {
                tokens.next();
                Some(typescript_ir::Statement::Break)
            }
            "continue" => {
                tokens.next();
                Some(typescript_ir::Statement::Continue)
            }
            _ => self.parse_expression_statement(tokens),
        }
    }

    /// 解析变量声明
    fn parse_variable_declaration(
        &self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<String>>,
    ) -> Option<typescript_ir::Statement> {
        // 跳过 let/const/var
        tokens.next();

        // 获取变量名
        let name = tokens.next()?;

        // 检查是否有类型注解
        let mut ty = None;
        if tokens.peek() == Some(&":".to_string()) {
            tokens.next(); // 跳过 :
            let type_name = tokens.next()?;
            ty = Some(typescript_ir::types::TypeAnnotation::Any);
        }

        // 检查是否有初始化值
        let mut initializer = None;
        if tokens.peek() == Some(&"=".to_string()) {
            tokens.next(); // 跳过 =
            if let Some(expr) = self.parse_expression(tokens) {
                initializer = Some(Box::new(expr));
            }
        }

        // 跳过分号
        if tokens.peek() == Some(&";".to_string()) {
            tokens.next();
        }

        Some(typescript_ir::Statement::VariableDeclaration { name, ty, initializer })
    }

    /// 解析函数声明
    fn parse_function_declaration(
        &self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<String>>,
    ) -> Option<typescript_ir::Statement> {
        // 跳过 function
        tokens.next();

        // 获取函数名
        let name = tokens.next()?;

        // 跳过 (
        if tokens.next() != Some("(".to_string()) {
            return None;
        }

        // 解析参数
        let mut params = Vec::new();
        while tokens.peek() != Some(&")".to_string()) {
            if let Some(param) = tokens.next() {
                params.push(param);
                if tokens.peek() == Some(&",".to_string()) {
                    tokens.next();
                }
            }
        }

        // 跳过 )
        tokens.next();

        // 检查是否有返回类型
        let mut return_type = None;
        if tokens.peek() == Some(&":".to_string()) {
            tokens.next(); // 跳过 :
            let type_name = tokens.next()?;
            return_type = Some(typescript_ir::types::TypeAnnotation::Any);
        }

        // 跳过 {
        if tokens.next() != Some("{".to_string()) {
            return None;
        }

        // 解析函数体
        let mut body = Vec::new();
        while tokens.peek() != Some(&"}".to_string()) {
            if let Some(stmt) = self.parse_statement(tokens) {
                body.push(stmt);
            }
        }

        // 跳过 }
        tokens.next();

        Some(typescript_ir::Statement::FunctionDeclaration {
            name,
            type_params: Vec::new(),
            params,
            return_type,
            body,
            decorators: Vec::new(),
        })
    }

    /// 解析类声明
    fn parse_class_declaration(
        &self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<String>>,
    ) -> Option<typescript_ir::Statement> {
        // 跳过 class
        tokens.next();

        // 获取类名
        let name = tokens.next()?;

        // 检查是否有父类
        let mut super_class = None;
        if tokens.peek() == Some(&"extends".to_string()) {
            tokens.next(); // 跳过 extends
            super_class = tokens.next();
        }

        // 跳过 {
        if tokens.next() != Some("{".to_string()) {
            return None;
        }

        // 解析方法
        let mut methods = Vec::new();
        while tokens.peek() != Some(&"}".to_string()) {
            if let Some(method) = self.parse_method(tokens) {
                methods.push(method);
            }
        }

        // 跳过 }
        tokens.next();

        Some(typescript_ir::Statement::ClassDeclaration { name, super_class, methods, decorators: Vec::new() })
    }

    /// 解析方法
    fn parse_method(
        &self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<String>>,
    ) -> Option<typescript_ir::program::Method> {
        // 获取方法名
        let name = tokens.next()?;

        // 跳过 (
        if tokens.next() != Some("(".to_string()) {
            return None;
        }

        // 解析参数
        let mut params = Vec::new();
        while tokens.peek() != Some(&")".to_string()) {
            if let Some(param) = tokens.next() {
                params.push(param);
                if tokens.peek() == Some(&",".to_string()) {
                    tokens.next();
                }
            }
        }

        // 跳过 )
        tokens.next();

        // 检查是否有返回类型
        let mut return_type = None;
        if tokens.peek() == Some(&":".to_string()) {
            tokens.next(); // 跳过 :
            let type_name = tokens.next()?;
            return_type = Some(typescript_ir::types::TypeAnnotation::Any);
        }

        // 跳过 {
        if tokens.next() != Some("{".to_string()) {
            return None;
        }

        // 解析方法体
        let mut body = Vec::new();
        while tokens.peek() != Some(&"}".to_string()) {
            if let Some(stmt) = self.parse_statement(tokens) {
                body.push(stmt);
            }
        }

        // 跳过 }
        tokens.next();

        Some(typescript_ir::program::Method {
            name,
            type_params: Vec::new(),
            params,
            return_type,
            body,
            decorators: Vec::new(),
        })
    }

    /// 解析 if 语句
    fn parse_if_statement(
        &self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<String>>,
    ) -> Option<typescript_ir::Statement> {
        // 跳过 if
        tokens.next();

        // 跳过 (
        if tokens.next() != Some("(".to_string()) {
            return None;
        }

        // 解析条件表达式
        let test = Box::new(self.parse_expression(tokens)?);

        // 跳过 )
        if tokens.next() != Some(")".to_string()) {
            return None;
        }

        // 解析 then 分支
        let consequent = Box::new(self.parse_statement(tokens)?);

        // 解析 else 分支
        let mut alternate = None;
        if tokens.peek() == Some(&"else".to_string()) {
            tokens.next(); // 跳过 else
            alternate = Some(Box::new(self.parse_statement(tokens)?));
        }

        Some(typescript_ir::Statement::If { test, consequent, alternate })
    }

    /// 解析 while 语句
    fn parse_while_statement(
        &self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<String>>,
    ) -> Option<typescript_ir::Statement> {
        // 跳过 while
        tokens.next();

        // 跳过 (
        if tokens.next() != Some("(".to_string()) {
            return None;
        }

        // 解析条件表达式
        let test = Box::new(self.parse_expression(tokens)?);

        // 跳过 )
        if tokens.next() != Some(")".to_string()) {
            return None;
        }

        // 解析循环体
        let body = Box::new(self.parse_statement(tokens)?);

        Some(typescript_ir::Statement::While { test, body })
    }

    /// 解析 for 语句
    fn parse_for_statement(
        &self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<String>>,
    ) -> Option<typescript_ir::Statement> {
        // 跳过 for
        tokens.next();

        // 跳过 (
        if tokens.next() != Some("(".to_string()) {
            return None;
        }

        // 解析初始化语句
        let init = if tokens.peek() != Some(&";".to_string()) { Some(Box::new(self.parse_statement(tokens)?)) } else { None };

        // 跳过 ;
        if tokens.peek() == Some(&";".to_string()) {
            tokens.next();
        }

        // 解析条件表达式
        let test = if tokens.peek() != Some(&";".to_string()) { Some(Box::new(self.parse_expression(tokens)?)) } else { None };

        // 跳过 ;
        if tokens.peek() == Some(&";".to_string()) {
            tokens.next();
        }

        // 解析更新表达式
        let update =
            if tokens.peek() != Some(&")".to_string()) { Some(Box::new(self.parse_expression(tokens)?)) } else { None };

        // 跳过 )
        if tokens.next() != Some(")".to_string()) {
            return None;
        }

        // 解析循环体
        let body = Box::new(self.parse_statement(tokens)?);

        Some(typescript_ir::Statement::For { init, test, update, body })
    }

    /// 解析 return 语句
    fn parse_return_statement(
        &self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<String>>,
    ) -> Option<typescript_ir::Statement> {
        // 跳过 return
        tokens.next();

        // 解析返回表达式
        let expr = if tokens.peek() != Some(&";".to_string()) { Some(Box::new(self.parse_expression(tokens)?)) } else { None };

        // 跳过 ;
        if tokens.peek() == Some(&";".to_string()) {
            tokens.next();
        }

        Some(typescript_ir::Statement::Return(expr))
    }

    /// 解析表达式语句
    fn parse_expression_statement(
        &self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<String>>,
    ) -> Option<typescript_ir::Statement> {
        if let Some(expr) = self.parse_expression(tokens) {
            // 跳过 ;
            if tokens.peek() == Some(&";".to_string()) {
                tokens.next();
            }
            Some(typescript_ir::Statement::Expression(Box::new(expr)))
        }
        else {
            None
        }
    }

    /// 解析表达式
    fn parse_expression(
        &self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<String>>,
    ) -> Option<typescript_ir::Expression> {
        self.parse_primary_expression(tokens)
    }

    /// 解析基本表达式
    fn parse_primary_expression(
        &self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<String>>,
    ) -> Option<typescript_ir::Expression> {
        let token = tokens.next()?;

        match token.as_str() {
            "true" => Some(typescript_ir::Expression::Literal(typescript_types::TsValue::Boolean(true))),
            "false" => Some(typescript_ir::Expression::Literal(typescript_types::TsValue::Boolean(false))),
            "null" => Some(typescript_ir::Expression::Literal(typescript_types::TsValue::Null)),
            "undefined" => Some(typescript_ir::Expression::Literal(typescript_types::TsValue::Undefined)),
            _ if token.parse::<f64>().is_ok() => {
                Some(typescript_ir::Expression::Literal(typescript_types::TsValue::Number(token.parse().unwrap())))
            }
            _ if token.starts_with('"') || token.starts_with('\'') => {
                let value = token.trim_matches('"').trim_matches('\'');
                Some(typescript_ir::Expression::Literal(typescript_types::TsValue::String(value.to_string())))
            }
            "(" => {
                let expr = self.parse_expression(tokens)?;
                if tokens.next() != Some(")".to_string()) { None } else { Some(expr) }
            }
            _ => {
                // 标识符
                Some(typescript_ir::Expression::Identifier(token))
            }
        }
    }

    /// 并行生成多个中间表示
    #[cfg(feature = "parallel")]
    pub fn generate_parallel(&mut self, sources: &[&str]) -> Vec<CompilationResult<Program>> {
        use rayon::prelude::*;
        sources.par_iter().map(|source| self.generate(source)).collect()
    }

    /// 管理缓存大小
    fn manage_cache_size(&mut self) {
        if self.ir_cache.len() > self.cache_size_limit {
            // 简单的 LRU 策略：移除一半的缓存项
            let remove_count = self.ir_cache.len() / 2;
            let keys_to_remove: Vec<u64> = self.ir_cache.keys().take(remove_count).cloned().collect();
            for key in keys_to_remove {
                self.ir_cache.remove(&key);
            }
        }
    }

    /// 获取当前编译阶段
    pub fn current_stage(&self) -> CompilationStage {
        self.current_stage
    }

    /// 重置中间表示生成器
    pub fn reset(&mut self) {
        self.current_stage = CompilationStage::IRGeneration;
        // 清空缓存
        self.ir_cache.clear();
    }

    /// 清除缓存
    pub fn clear_cache(&mut self) {
        self.ir_cache.clear();
    }

    /// 设置缓存大小限制
    pub fn set_cache_size_limit(&mut self, limit: usize) {
        self.cache_size_limit = limit;
        // 立即调整缓存大小
        self.manage_cache_size();
    }

    /// 获取缓存大小
    pub fn cache_size(&self) -> usize {
        self.ir_cache.len()
    }
}

impl Default for IRGenerator {
    fn default() -> Self {
        Self::new()
    }
}
