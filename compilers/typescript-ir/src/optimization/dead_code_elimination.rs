//! 死代码消除优化模块
//!
//! 该模块实现了死代码消除优化，通过分析控制流和数据流，
//! 移除不会被执行或不会影响程序结果的代码。

use crate::{ControlFlowGraph, Expression, Statement};
use std::collections::{HashMap, HashSet};

/// 死代码消除优化器
///
/// 该优化器通过分析程序的控制流和数据流，识别并移除死代码
pub struct DeadCodeElimination;

/// 活跃变量分析器
///
/// 用于分析每个程序点哪些变量是活跃的（后续可能被使用）
#[derive(Debug, Clone)]
pub struct LiveVariableAnalysis {
    /// 每个语句位置的入口活跃变量集
    pub live_in: HashMap<usize, HashSet<String>>,
    /// 每个语句位置的出口活跃变量集
    pub live_out: HashMap<usize, HashSet<String>>,
}

impl Default for LiveVariableAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

impl LiveVariableAnalysis {
    /// 创建新的活跃变量分析器
    pub fn new() -> Self {
        Self { live_in: HashMap::new(), live_out: HashMap::new() }
    }

    /// 执行活跃变量分析
    pub fn analyze(&mut self, statements: &[Statement]) {
        let n = statements.len();
        for i in 0..n {
            self.live_in.insert(i, HashSet::new());
            self.live_out.insert(i, HashSet::new());
        }

        let mut changed = true;
        while changed {
            changed = false;
            for i in (0..n).rev() {
                let mut new_out = HashSet::new();
                if i + 1 < n {
                    if let Some(in_set) = self.live_in.get(&(i + 1)) {
                        new_out = in_set.clone();
                    }
                }

                let mut new_in = new_out.clone();
                Self::apply_statement_transfer(&statements[i], &mut new_in);

                if self.live_in.get(&i) != Some(&new_in) || self.live_out.get(&i) != Some(&new_out) {
                    changed = true;
                    self.live_in.insert(i, new_in);
                    self.live_out.insert(i, new_out);
                }
            }
        }
    }

    /// 应用语句的数据流传递函数
    fn apply_statement_transfer(stmt: &Statement, live: &mut HashSet<String>) {
        match stmt {
            Statement::VariableDeclaration { name, initializer, .. } => {
                live.remove(name);
                if let Some(init) = initializer {
                    Self::add_used_variables_in_expr(init, live);
                }
            }
            Statement::Expression(expr) => {
                Self::add_used_variables_in_expr(expr, live);
            }
            Statement::Return(expr) => {
                if let Some(e) = expr {
                    Self::add_used_variables_in_expr(e, live);
                }
            }
            Statement::If { test, consequent, alternate } => {
                Self::add_used_variables_in_expr(test, live);
                Self::apply_statement_transfer(consequent, live);
                if let Some(alt) = alternate {
                    Self::apply_statement_transfer(alt, live);
                }
            }
            Statement::While { test, body } => {
                Self::add_used_variables_in_expr(test, live);
                Self::apply_statement_transfer(body, live);
            }
            Statement::For { init, test, update, body } => {
                if let Some(t) = test {
                    Self::add_used_variables_in_expr(t, live);
                }
                if let Some(u) = update {
                    Self::add_used_variables_in_expr(u, live);
                }
                Self::apply_statement_transfer(body, live);
                if let Some(init_stmt) = init {
                    Self::apply_statement_transfer(init_stmt, live);
                }
            }
            Statement::Block(statements) => {
                for s in statements.iter().rev() {
                    Self::apply_statement_transfer(s, live);
                }
            }
            _ => {}
        }
    }

    /// 将表达式中使用的变量添加到活跃集合
    fn add_used_variables_in_expr(expr: &Expression, live: &mut HashSet<String>) {
        match expr {
            Expression::Identifier(name) => {
                live.insert(name.clone());
            }
            Expression::Binary { left, right, .. } => {
                Self::add_used_variables_in_expr(left, live);
                Self::add_used_variables_in_expr(right, live);
            }
            Expression::Unary { expr: inner_expr, .. } => {
                Self::add_used_variables_in_expr(inner_expr, live);
            }
            Expression::Call { callee, args } => {
                Self::add_used_variables_in_expr(callee, live);
                for arg in args {
                    Self::add_used_variables_in_expr(arg, live);
                }
            }
            Expression::Member { object, property } => {
                Self::add_used_variables_in_expr(object, live);
                Self::add_used_variables_in_expr(property, live);
            }
            Expression::Index { object, index } => {
                Self::add_used_variables_in_expr(object, live);
                Self::add_used_variables_in_expr(index, live);
            }
            Expression::Assignment { left, right, .. } => {
                Self::add_used_variables_in_expr(right, live);
                Self::add_used_variables_in_expr(left, live);
            }
            Expression::Conditional { test, consequent, alternate } => {
                Self::add_used_variables_in_expr(test, live);
                Self::add_used_variables_in_expr(consequent, live);
                Self::add_used_variables_in_expr(alternate, live);
            }
            Expression::Array(elements) => {
                for elem in elements {
                    Self::add_used_variables_in_expr(elem, live);
                }
            }
            Expression::Object(properties) => {
                for (_, value) in properties {
                    Self::add_used_variables_in_expr(value, live);
                }
            }
            Expression::Function { body, .. } => {
                for stmt in body {
                    Self::apply_statement_transfer(stmt, live);
                }
            }
            Expression::ArrowFunction { body, .. } => {
                Self::add_used_variables_in_expr(body, live);
            }
            Expression::Literal(_) => {}
        }
    }
}

/// 控制流分析器
///
/// 用于分析程序的控制流，识别不可达代码等
#[derive(Debug, Clone)]
pub struct ControlFlowAnalyzer {
    /// 可达的语句索引集合
    pub reachable: HashSet<usize>,
    /// 是否包含无限循环
    pub has_infinite_loop: bool,
}

impl Default for ControlFlowAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlFlowAnalyzer {
    /// 创建新的控制流分析器
    pub fn new() -> Self {
        Self { reachable: HashSet::new(), has_infinite_loop: false }
    }

    /// 分析控制流，标记可达代码
    pub fn analyze(&mut self, statements: &[Statement]) {
        let n = statements.len();
        if n == 0 {
            return;
        }

        let mut worklist = vec![0];
        let mut visited = HashSet::new();

        while let Some(idx) = worklist.pop() {
            if idx >= n || visited.contains(&idx) {
                continue;
            }
            visited.insert(idx);
            self.reachable.insert(idx);

            let stmt = &statements[idx];

            match stmt {
                Statement::Return(_) | Statement::Break | Statement::Continue => {
                    // 这些语句终止当前控制流，不继续到下一条语句
                }
                Statement::If { test, consequent, alternate } => {
                    // 如果条件是常量，只有一边可达
                    if let Some(test_val) = test.eval() {
                        if test_val.to_boolean() {
                            self.add_successor_from_statement(consequent, &mut worklist, statements, idx);
                        }
                        else if let Some(alt) = alternate {
                            self.add_successor_from_statement(alt, &mut worklist, statements, idx);
                        }
                    }
                    else {
                        // 条件不是常量，两边都可能可达
                        self.add_successor_from_statement(consequent, &mut worklist, statements, idx);
                        if let Some(alt) = alternate {
                            self.add_successor_from_statement(alt, &mut worklist, statements, idx);
                        }
                        // 下一条语句也可能可达
                        worklist.push(idx + 1);
                    }
                }
                Statement::While { test, body } => {
                    if let Some(test_val) = test.eval() {
                        if test_val.to_boolean() {
                            // 无限循环
                            self.has_infinite_loop = true;
                            self.add_successor_from_statement(body, &mut worklist, statements, idx);
                        }
                        // 否则跳过循环
                    }
                    else {
                        self.add_successor_from_statement(body, &mut worklist, statements, idx);
                        worklist.push(idx + 1);
                    }
                }
                Statement::For { test, body, .. } => {
                    if let Some(test_expr) = test {
                        if let Some(test_val) = test_expr.eval() {
                            if test_val.to_boolean() {
                                self.has_infinite_loop = true;
                                self.add_successor_from_statement(body, &mut worklist, statements, idx);
                            }
                        }
                        else {
                            self.add_successor_from_statement(body, &mut worklist, statements, idx);
                            worklist.push(idx + 1);
                        }
                    }
                    else {
                        // 无条件循环
                        self.has_infinite_loop = true;
                        self.add_successor_from_statement(body, &mut worklist, statements, idx);
                    }
                }
                Statement::Block(inner_statements) => {
                    // 块内的语句顺序执行
                    for inner_stmt in inner_statements {
                        if matches!(inner_stmt, Statement::Return(_) | Statement::Break | Statement::Continue) {
                            break;
                        }
                    }
                    worklist.push(idx + 1);
                }
                _ => {
                    // 普通语句，继续到下一条
                    worklist.push(idx + 1);
                }
            }
        }
    }

    /// 从语句添加后继到工作列表
    fn add_successor_from_statement(
        &self,
        stmt: &Statement,
        worklist: &mut Vec<usize>,
        statements: &[Statement],
        current_idx: usize,
    ) {
        if let Statement::Block(inner) = stmt {
            if !inner.is_empty() {
                // 对于块语句，我们需要递归处理
                // 这里简化处理，假设块内的语句会被单独分析
            }
        }
        worklist.push(current_idx + 1);
    }
}

impl DeadCodeElimination {
    /// 执行死代码消除
    ///
    /// 分析语句列表，移除不会被执行或不会影响程序结果的代码
    pub fn eliminate(statements: &mut Vec<Statement>, _cfg: &ControlFlowGraph) {
        let mut live_analysis = LiveVariableAnalysis::new();
        live_analysis.analyze(statements);

        let mut cf_analysis = ControlFlowAnalyzer::new();
        cf_analysis.analyze(statements);

        let mut i = 0;
        while i < statements.len() {
            if !cf_analysis.reachable.contains(&i) {
                statements.remove(i);
                continue;
            }

            match &statements[i] {
                Statement::VariableDeclaration { name, .. } => {
                    let is_used = Self::is_variable_used_after_position(statements, name, i);
                    if !is_used {
                        statements.remove(i);
                        continue;
                    }
                }
                Statement::Block(inner_statements) => {
                    let mut optimized_inner = inner_statements.clone();
                    Self::eliminate(&mut optimized_inner, _cfg);
                    if optimized_inner.is_empty() {
                        statements.remove(i);
                        continue;
                    }
                    else {
                        statements[i] = Statement::Block(optimized_inner);
                    }
                }
                Statement::Expression(expr) => {
                    if Self::is_pure_expression(expr) && !Self::has_side_effects(expr) {
                        statements.remove(i);
                        continue;
                    }
                }
                Statement::If { test, consequent, alternate } => {
                    if let Some(test_val) = test.eval() {
                        if test_val.to_boolean() {
                            let mut consequent_stmt = (**consequent).clone();
                            Self::eliminate_statement(&mut consequent_stmt, _cfg);
                            statements[i] = consequent_stmt;
                        }
                        else if let Some(alt) = alternate {
                            let mut alt_stmt = (**alt).clone();
                            Self::eliminate_statement(&mut alt_stmt, _cfg);
                            statements[i] = alt_stmt;
                        }
                        else {
                            statements.remove(i);
                            continue;
                        }
                    }
                    else {
                        let mut consequent_stmt = (**consequent).clone();
                        Self::eliminate_statement(&mut consequent_stmt, _cfg);
                        let mut alternate_stmt = alternate.as_ref().map(|alt| {
                            let mut s = (**alt).clone();
                            Self::eliminate_statement(&mut s, _cfg);
                            Box::new(s)
                        });

                        if Self::is_empty_statement(&consequent_stmt) && alternate_stmt.is_none() {
                            statements.remove(i);
                            continue;
                        }

                        statements[i] = Statement::If {
                            test: test.clone(),
                            consequent: Box::new(consequent_stmt),
                            alternate: alternate_stmt,
                        };
                    }
                }
                Statement::While { test, body } => {
                    if let Some(test_val) = test.eval() {
                        if !test_val.to_boolean() {
                            statements.remove(i);
                            continue;
                        }
                    }
                    let mut body_stmt = (**body).clone();
                    Self::eliminate_statement(&mut body_stmt, _cfg);
                    statements[i] = Statement::While { test: test.clone(), body: Box::new(body_stmt) };
                }
                Statement::For { init, test, update, body } => {
                    if let Some(test_expr) = test {
                        if let Some(test_val) = test_expr.eval() {
                            if !test_val.to_boolean() {
                                statements.remove(i);
                                continue;
                            }
                        }
                    }
                    let mut body_stmt = (**body).clone();
                    Self::eliminate_statement(&mut body_stmt, _cfg);
                    statements[i] = Statement::For {
                        init: init.clone(),
                        test: test.clone(),
                        update: update.clone(),
                        body: Box::new(body_stmt),
                    };
                }
                Statement::FunctionDeclaration { name, params, return_type, body, type_params, decorators } => {
                    let is_used = Self::is_function_used(statements, name);
                    if !is_used {
                        statements.remove(i);
                        continue;
                    }
                    let mut optimized_body = body.clone();
                    Self::eliminate(&mut optimized_body, _cfg);
                    statements[i] = Statement::FunctionDeclaration {
                        name: name.clone(),
                        params: params.clone(),
                        return_type: return_type.clone(),
                        body: optimized_body,
                        type_params: type_params.clone(),
                        decorators: decorators.clone(),
                    };
                }
                _ => {}
            }
            i += 1;
        }
    }

    /// 对单个语句执行死代码消除
    pub fn eliminate_statement(stmt: &mut Statement, cfg: &ControlFlowGraph) {
        match stmt {
            Statement::Block(statements) => {
                Self::eliminate(statements, cfg);
            }
            Statement::If { test, consequent, alternate } => {
                if let Some(test_val) = test.eval() {
                    if test_val.to_boolean() {
                        Self::eliminate_statement(consequent, cfg);
                        *stmt = (**consequent).clone();
                    }
                    else if let Some(alt) = alternate {
                        Self::eliminate_statement(alt, cfg);
                        *stmt = (**alt).clone();
                    }
                    else {
                        *stmt = Statement::Block(vec![]);
                    }
                }
                else {
                    Self::eliminate_statement(consequent, cfg);
                    if let Some(alt) = alternate {
                        Self::eliminate_statement(alt, cfg);
                    }
                }
            }
            Statement::While { test, body } => {
                let should_eliminate = if let Some(test_val) = test.eval() { !test_val.to_boolean() } else { false };
                if should_eliminate {
                    *stmt = Statement::Block(vec![]);
                }
                else {
                    Self::eliminate_statement(body, cfg);
                }
            }
            Statement::For { test, body, .. } => {
                let should_eliminate = if let Some(test_expr) = test {
                    if let Some(test_val) = test_expr.eval() { !test_val.to_boolean() } else { false }
                }
                else {
                    false
                };
                if should_eliminate {
                    *stmt = Statement::Block(vec![]);
                }
                else {
                    Self::eliminate_statement(body, cfg);
                }
            }
            Statement::FunctionDeclaration { body, .. } => {
                Self::eliminate(body, cfg);
            }
            _ => {}
        }
    }

    /// 检查变量是否在指定位置之后被使用
    pub fn is_variable_used_after_position(statements: &[Statement], variable_name: &str, start_pos: usize) -> bool {
        for j in start_pos + 1..statements.len() {
            if Self::is_variable_used(&statements[j], variable_name) {
                return true;
            }
        }
        false
    }

    /// 检查变量是否在语句中被使用
    pub fn is_variable_used(stmt: &Statement, variable_name: &str) -> bool {
        match stmt {
            Statement::Expression(expr) => Self::is_variable_used_in_expr(expr, variable_name),
            Statement::VariableDeclaration { initializer, .. } => {
                if let Some(init) = initializer {
                    Self::is_variable_used_in_expr(init, variable_name)
                }
                else {
                    false
                }
            }
            Statement::If { test, consequent, alternate } => {
                Self::is_variable_used_in_expr(test, variable_name)
                    || Self::is_variable_used(consequent, variable_name)
                    || alternate.as_ref().map(|alt| Self::is_variable_used(alt, variable_name)).unwrap_or(false)
            }
            Statement::While { test, body } => {
                Self::is_variable_used_in_expr(test, variable_name) || Self::is_variable_used(body, variable_name)
            }
            Statement::For { init, test, update, body } => {
                init.as_ref().map(|init_stmt| Self::is_variable_used(init_stmt, variable_name)).unwrap_or(false)
                    || test.as_ref().map(|test_expr| Self::is_variable_used_in_expr(test_expr, variable_name)).unwrap_or(false)
                    || update
                        .as_ref()
                        .map(|update_expr| Self::is_variable_used_in_expr(update_expr, variable_name))
                        .unwrap_or(false)
                    || Self::is_variable_used(body, variable_name)
            }
            Statement::Return(expr) => expr.as_ref().map(|e| Self::is_variable_used_in_expr(e, variable_name)).unwrap_or(false),
            Statement::Block(statements) => {
                for s in statements {
                    if Self::is_variable_used(s, variable_name) {
                        return true;
                    }
                }
                false
            }
            Statement::FunctionDeclaration { body, .. } => {
                for s in body {
                    if Self::is_variable_used(s, variable_name) {
                        return true;
                    }
                }
                false
            }
            Statement::ClassDeclaration { methods, .. } => {
                for method in methods {
                    for s in &method.body {
                        if Self::is_variable_used(s, variable_name) {
                            return true;
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// 检查变量是否在表达式中被使用
    pub fn is_variable_used_in_expr(expr: &Expression, variable_name: &str) -> bool {
        match expr {
            Expression::Identifier(name) => name == variable_name,
            Expression::Binary { left, right, .. } => {
                Self::is_variable_used_in_expr(left, variable_name) || Self::is_variable_used_in_expr(right, variable_name)
            }
            Expression::Unary { expr: inner_expr, .. } => Self::is_variable_used_in_expr(inner_expr, variable_name),
            Expression::Call { callee, args } => {
                Self::is_variable_used_in_expr(callee, variable_name)
                    || args.iter().any(|arg| Self::is_variable_used_in_expr(arg, variable_name))
            }
            Expression::Member { object, property } => {
                Self::is_variable_used_in_expr(object, variable_name) || Self::is_variable_used_in_expr(property, variable_name)
            }
            Expression::Index { object, index } => {
                Self::is_variable_used_in_expr(object, variable_name) || Self::is_variable_used_in_expr(index, variable_name)
            }
            Expression::Assignment { left, right, .. } => {
                Self::is_variable_used_in_expr(left, variable_name) || Self::is_variable_used_in_expr(right, variable_name)
            }
            Expression::Conditional { test, consequent, alternate } => {
                Self::is_variable_used_in_expr(test, variable_name)
                    || Self::is_variable_used_in_expr(consequent, variable_name)
                    || Self::is_variable_used_in_expr(alternate, variable_name)
            }
            Expression::Array(elements) => elements.iter().any(|e| Self::is_variable_used_in_expr(e, variable_name)),
            Expression::Object(properties) => properties.iter().any(|(_, v)| Self::is_variable_used_in_expr(v, variable_name)),
            Expression::Function { body, params, .. } => {
                if params.contains(&variable_name.to_string()) {
                    return false;
                }
                body.iter().any(|s| Self::is_variable_used(s, variable_name))
            }
            Expression::ArrowFunction { body, params, .. } => {
                if params.contains(&variable_name.to_string()) {
                    return false;
                }
                Self::is_variable_used_in_expr(body, variable_name)
            }
            Expression::Literal(_) => false,
        }
    }

    /// 检查语句是否为死代码
    pub fn is_dead_code(stmt: &Statement) -> bool {
        match stmt {
            Statement::Block(statements) => statements.is_empty(),
            _ => false,
        }
    }

    /// 检查表达式是否为纯表达式（无副作用）
    pub fn is_pure_expression(expr: &Expression) -> bool {
        match expr {
            Expression::Literal(_) => true,
            Expression::Identifier(_) => true,
            Expression::Binary { left, right, .. } => Self::is_pure_expression(left) && Self::is_pure_expression(right),
            Expression::Unary { expr: inner_expr, .. } => Self::is_pure_expression(inner_expr),
            Expression::Conditional { test, consequent, alternate } => {
                Self::is_pure_expression(test) && Self::is_pure_expression(consequent) && Self::is_pure_expression(alternate)
            }
            Expression::Array(elements) => elements.iter().all(Self::is_pure_expression),
            Expression::Object(properties) => properties.iter().all(|(_, v)| Self::is_pure_expression(v)),
            Expression::Member { object, property } => Self::is_pure_expression(object) && Self::is_pure_expression(property),
            Expression::Index { object, index } => Self::is_pure_expression(object) && Self::is_pure_expression(index),
            Expression::ArrowFunction { .. } => true,
            Expression::Function { .. } => true,
            Expression::Call { .. } => false,
            Expression::Assignment { .. } => false,
        }
    }

    /// 检查表达式是否有副作用
    pub fn has_side_effects(expr: &Expression) -> bool {
        match expr {
            Expression::Call { .. } => true,
            Expression::Assignment { .. } => true,
            Expression::Binary { left, right, .. } => Self::has_side_effects(left) || Self::has_side_effects(right),
            Expression::Unary { expr: inner_expr, .. } => Self::has_side_effects(inner_expr),
            Expression::Conditional { test, consequent, alternate } => {
                Self::has_side_effects(test) || Self::has_side_effects(consequent) || Self::has_side_effects(alternate)
            }
            Expression::Array(elements) => elements.iter().any(Self::has_side_effects),
            Expression::Object(properties) => properties.iter().any(|(_, v)| Self::has_side_effects(v)),
            Expression::Member { object, property } => Self::has_side_effects(object) || Self::has_side_effects(property),
            Expression::Index { object, index } => Self::has_side_effects(object) || Self::has_side_effects(index),
            Expression::Literal(_) => false,
            Expression::Identifier(_) => false,
            Expression::Function { .. } => false,
            Expression::ArrowFunction { .. } => false,
        }
    }

    /// 检查语句是否为空语句
    pub fn is_empty_statement(stmt: &Statement) -> bool {
        match stmt {
            Statement::Block(statements) => statements.is_empty(),
            _ => false,
        }
    }

    /// 检查函数是否被使用
    pub fn is_function_used(statements: &[Statement], function_name: &str) -> bool {
        for stmt in statements {
            if Self::is_function_used_in_statement(stmt, function_name) {
                return true;
            }
        }
        false
    }

    /// 检查函数是否在语句中被调用
    fn is_function_used_in_statement(stmt: &Statement, function_name: &str) -> bool {
        match stmt {
            Statement::Expression(expr) => Self::is_function_used_in_expr(expr, function_name),
            Statement::VariableDeclaration { initializer, .. } => {
                initializer.as_ref().map(|e| Self::is_function_used_in_expr(e, function_name)).unwrap_or(false)
            }
            Statement::If { test, consequent, alternate } => {
                Self::is_function_used_in_expr(test, function_name)
                    || Self::is_function_used_in_statement(consequent, function_name)
                    || alternate.as_ref().map(|a| Self::is_function_used_in_statement(a, function_name)).unwrap_or(false)
            }
            Statement::While { test, body } => {
                Self::is_function_used_in_expr(test, function_name) || Self::is_function_used_in_statement(body, function_name)
            }
            Statement::For { init, test, update, body } => {
                init.as_ref().map(|i| Self::is_function_used_in_statement(i, function_name)).unwrap_or(false)
                    || test.as_ref().map(|t| Self::is_function_used_in_expr(t, function_name)).unwrap_or(false)
                    || update.as_ref().map(|u| Self::is_function_used_in_expr(u, function_name)).unwrap_or(false)
                    || Self::is_function_used_in_statement(body, function_name)
            }
            Statement::Return(expr) => expr.as_ref().map(|e| Self::is_function_used_in_expr(e, function_name)).unwrap_or(false),
            Statement::Block(statements) => statements.iter().any(|s| Self::is_function_used_in_statement(s, function_name)),
            Statement::FunctionDeclaration { body, .. } => {
                body.iter().any(|s| Self::is_function_used_in_statement(s, function_name))
            }
            Statement::ClassDeclaration { methods, .. } => {
                methods.iter().any(|m| m.body.iter().any(|s| Self::is_function_used_in_statement(s, function_name)))
            }
            Statement::ExportDeclaration { declaration, .. } => {
                Self::is_function_used_in_statement(declaration, function_name) || {
                    if let Statement::FunctionDeclaration { name, .. } = declaration.as_ref() {
                        name == function_name
                    }
                    else {
                        false
                    }
                }
            }
            _ => false,
        }
    }

    /// 检查函数是否在表达式中被调用
    fn is_function_used_in_expr(expr: &Expression, function_name: &str) -> bool {
        match expr {
            Expression::Identifier(name) => name == function_name,
            Expression::Binary { left, right, .. } => {
                Self::is_function_used_in_expr(left, function_name) || Self::is_function_used_in_expr(right, function_name)
            }
            Expression::Unary { expr: inner_expr, .. } => Self::is_function_used_in_expr(inner_expr, function_name),
            Expression::Call { callee, args } => {
                Self::is_function_used_in_expr(callee, function_name)
                    || args.iter().any(|a| Self::is_function_used_in_expr(a, function_name))
            }
            Expression::Member { object, property } => {
                Self::is_function_used_in_expr(object, function_name) || Self::is_function_used_in_expr(property, function_name)
            }
            Expression::Index { object, index } => {
                Self::is_function_used_in_expr(object, function_name) || Self::is_function_used_in_expr(index, function_name)
            }
            Expression::Assignment { left, right, .. } => {
                Self::is_function_used_in_expr(left, function_name) || Self::is_function_used_in_expr(right, function_name)
            }
            Expression::Conditional { test, consequent, alternate } => {
                Self::is_function_used_in_expr(test, function_name)
                    || Self::is_function_used_in_expr(consequent, function_name)
                    || Self::is_function_used_in_expr(alternate, function_name)
            }
            Expression::Array(elements) => elements.iter().any(|e| Self::is_function_used_in_expr(e, function_name)),
            Expression::Object(properties) => properties.iter().any(|(_, v)| Self::is_function_used_in_expr(v, function_name)),
            Expression::Function { body, .. } => body.iter().any(|s| Self::is_function_used_in_statement(s, function_name)),
            Expression::ArrowFunction { body, .. } => Self::is_function_used_in_expr(body, function_name),
            Expression::Literal(_) => false,
        }
    }
}
