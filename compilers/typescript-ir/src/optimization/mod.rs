use crate::{ControlFlowGraph, Expression, Module, Program, Statement};
use typescript_types::TsValue;

/// 性能分析数据
#[derive(Debug, Clone)]
pub struct ProfileData {
    /// 函数调用次数
    pub function_calls: std::collections::HashMap<String, usize>,
    /// 循环执行次数
    pub loop_executions: std::collections::HashMap<usize, usize>,
    /// 热点表达式
    pub hot_expressions: std::collections::HashMap<String, usize>,
}

/// 优化器
pub struct Optimizer {
    /// 优化级别
    pub optimization_level: OptimizationLevel,
    /// 性能分析数据
    pub profile_data: Option<ProfileData>,
}

/// 优化级别
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum OptimizationLevel {
    /// 无优化
    None,
    /// 基本优化
    Basic,
    /// 中级优化
    Medium,
    /// 高级优化
    High,
}

impl Optimizer {
    /// 创建一个新的优化器
    pub fn new(optimization_level: OptimizationLevel) -> Self {
        Self { optimization_level, profile_data: None }
    }

    /// 创建带有性能分析数据的优化器
    pub fn new_with_profile(optimization_level: OptimizationLevel, profile_data: ProfileData) -> Self {
        Self { optimization_level, profile_data: Some(profile_data) }
    }

    /// 优化程序
    pub fn optimize_program(&self, program: &Program) -> Program {
        if self.optimization_level == OptimizationLevel::None {
            return program.clone();
        }

        let mut optimized_statements = Vec::new();
        let mut has_return = false;

        for stmt in &program.statements {
            if has_return {
                // 跳过 return 后的死代码
                continue;
            }

            let optimized_stmt = self.optimize_statement(stmt);
            if !self.is_dead_code(&optimized_stmt) {
                optimized_statements.push(optimized_stmt.clone());
                // 检查是否是 return 语句
                if matches!(optimized_stmt, Statement::Return(_)) {
                    has_return = true;
                }
            }
        }

        // 对于高级优化，执行控制流分析和优化
        if self.optimization_level == OptimizationLevel::High {
            self.optimize_with_cfg(&mut optimized_statements);
        }

        Program { statements: optimized_statements }
    }

    /// 优化模块
    pub fn optimize_module(&self, module: &Module) -> Module {
        if self.optimization_level == OptimizationLevel::None {
            return module.clone();
        }

        let mut optimized_statements = Vec::new();
        let mut has_return = false;

        for stmt in &module.statements {
            if has_return {
                // 跳过 return 后的死代码
                continue;
            }

            let optimized_stmt = self.optimize_statement(stmt);
            if !self.is_dead_code(&optimized_stmt) {
                optimized_statements.push(optimized_stmt.clone());
                // 检查是否是 return 语句
                if matches!(optimized_stmt, Statement::Return(_)) {
                    has_return = true;
                }
            }
        }

        Module { name: module.name.clone(), statements: optimized_statements }
    }

    /// 使用控制流图进行优化
    fn optimize_with_cfg(&self, statements: &mut Vec<Statement>) {
        // 构建控制流图
        let mut cfg = ControlFlowGraph::from_statements(statements);

        // 执行到达定义分析
        cfg.reaching_definitions_analysis();

        // 执行活跃变量分析
        cfg.live_variables_analysis();

        // 基于分析结果进行优化
        self.optimize_using_analysis(statements, &cfg);
    }

    /// 基于分析结果进行优化
    fn optimize_using_analysis(&self, statements: &mut Vec<Statement>, cfg: &ControlFlowGraph) {
        // 实现基于到达定义和活跃变量的优化
        // 1. 常量传播
        self.propagate_constants(statements, cfg);

        // 2. 死代码消除
        self.eliminate_dead_code(statements, cfg);

        // 3. 循环不变式外提
        self.hoist_loop_invariants(statements);

        // 4. 公共子表达式消除
        self.eliminate_common_subexpressions(statements);

        // 5. 强度削弱
        self.strength_reduction(statements);

        // 6. 基于 profile 的优化
        if self.profile_data.is_some() {
            self.optimize_based_on_profile(statements);
        }

        // 7. 变量重命名（可选）
        // self.rename_variables(statements, cfg);
    }

    /// 循环不变式外提
    fn hoist_loop_invariants(&self, statements: &mut Vec<Statement>) {
        let mut i = 0;
        while i < statements.len() {
            match &mut statements[i] {
                Statement::While { test, body } => {
                    let invariants = self.find_loop_invariants(body, test);
                    if !invariants.is_empty() {
                        // 将不变式移到循环前
                        let invariants_clone = invariants.clone();
                        statements.splice(i..i, invariants);
                        i += invariants_clone.len();
                    }
                }
                Statement::For { init, test, update, body } => {
                    let invariants = self
                        .find_loop_invariants(body, test.as_deref().unwrap_or(&Expression::Literal(TsValue::Boolean(true))));
                    if !invariants.is_empty() {
                        // 将不变式移到循环前
                        let invariants_clone = invariants.clone();
                        statements.splice(i..i, invariants);
                        i += invariants_clone.len();
                    }
                }
                Statement::Block(inner_statements) => {
                    self.hoist_loop_invariants(inner_statements);
                }
                _ => {}
            }
            i += 1;
        }
    }

    /// 查找循环不变式
    fn find_loop_invariants(&self, body: &Statement, test: &Expression) -> Vec<Statement> {
        let mut invariants = Vec::new();
        let mut loop_variables = self.find_loop_variables(body, test);

        // 查找循环体中的不变变量声明
        if let Statement::Block(inner_statements) = body {
            for stmt in inner_statements {
                if let Statement::VariableDeclaration { name, ty, initializer } = stmt {
                    if let Some(init) = initializer {
                        // 检查初始化表达式是否不依赖循环变量
                        if !self.depends_on_variables(init, &loop_variables) {
                            invariants.push(Statement::VariableDeclaration {
                                name: name.clone(),
                                ty: ty.clone(),
                                initializer: Some(init.clone()),
                            });
                        }
                    }
                }
            }
        }

        invariants
    }

    /// 查找循环变量
    fn find_loop_variables(&self, body: &Statement, test: &Expression) -> std::collections::HashSet<String> {
        let mut variables = std::collections::HashSet::new();
        self.collect_variables(body, &mut variables);
        self.collect_variables_in_expr(test, &mut variables);
        variables
    }

    /// 收集表达式中使用的变量
    fn collect_variables_in_expr(&self, expr: &Expression, variables: &mut std::collections::HashSet<String>) {
        match expr {
            Expression::Identifier(name) => {
                variables.insert(name.clone());
            }
            Expression::Binary { left, right, .. } => {
                self.collect_variables_in_expr(left, variables);
                self.collect_variables_in_expr(right, variables);
            }
            Expression::Unary { expr: inner_expr, .. } => {
                self.collect_variables_in_expr(inner_expr, variables);
            }
            Expression::Call { callee, args } => {
                self.collect_variables_in_expr(callee, variables);
                for arg in args {
                    self.collect_variables_in_expr(arg, variables);
                }
            }
            Expression::Member { object, property } => {
                self.collect_variables_in_expr(object, variables);
                self.collect_variables_in_expr(property, variables);
            }
            _ => {}
        }
    }

    /// 收集语句中使用的变量
    fn collect_variables(&self, stmt: &Statement, variables: &mut std::collections::HashSet<String>) {
        match stmt {
            Statement::VariableDeclaration { name, initializer, .. } => {
                variables.insert(name.clone());
                if let Some(init) = initializer {
                    self.collect_variables_in_expr(init, variables);
                }
            }
            Statement::Expression(expr) => {
                self.collect_variables_in_expr(expr, variables);
            }
            Statement::If { test, consequent, alternate } => {
                self.collect_variables_in_expr(test, variables);
                self.collect_variables(consequent, variables);
                if let Some(alt) = alternate {
                    self.collect_variables(alt, variables);
                }
            }
            Statement::While { test, body } => {
                self.collect_variables_in_expr(test, variables);
                self.collect_variables(body, variables);
            }
            Statement::For { init, test, update, body } => {
                if let Some(init_stmt) = init {
                    self.collect_variables(init_stmt, variables);
                }
                if let Some(test_expr) = test {
                    self.collect_variables_in_expr(test_expr, variables);
                }
                if let Some(update_expr) = update {
                    self.collect_variables_in_expr(update_expr, variables);
                }
                self.collect_variables(body, variables);
            }
            Statement::Return(expr) => {
                if let Some(e) = expr {
                    self.collect_variables_in_expr(e, variables);
                }
            }
            Statement::Block(statements) => {
                for stmt in statements {
                    self.collect_variables(stmt, variables);
                }
            }
            _ => {}
        }
    }

    /// 检查表达式是否依赖指定变量
    fn depends_on_variables(&self, expr: &Expression, variables: &std::collections::HashSet<String>) -> bool {
        match expr {
            Expression::Identifier(name) => variables.contains(name),
            Expression::Binary { left, right, .. } => {
                self.depends_on_variables(left, variables) || self.depends_on_variables(right, variables)
            }
            Expression::Unary { expr: inner_expr, .. } => self.depends_on_variables(inner_expr, variables),
            Expression::Call { callee, args } => {
                self.depends_on_variables(callee, variables) || args.iter().any(|arg| self.depends_on_variables(arg, variables))
            }
            Expression::Member { object, property } => {
                self.depends_on_variables(object, variables) || self.depends_on_variables(property, variables)
            }
            _ => false,
        }
    }

    /// 公共子表达式消除
    fn eliminate_common_subexpressions(&self, statements: &mut Vec<Statement>) {
        // 实现公共子表达式消除
        // 这里是一个简化的实现
        for stmt in statements {
            match stmt {
                Statement::Expression(expr) => {
                    *expr = Box::new(self.eliminate_common_subexpressions_in_expr(expr));
                }
                Statement::Block(inner_statements) => {
                    self.eliminate_common_subexpressions(inner_statements);
                }
                _ => {}
            }
        }
    }

    /// 在表达式中消除公共子表达式
    fn eliminate_common_subexpressions_in_expr(&self, expr: &Expression) -> Expression {
        match expr {
            Expression::Binary { left, op, right } => {
                let optimized_left = self.eliminate_common_subexpressions_in_expr(left);
                let optimized_right = self.eliminate_common_subexpressions_in_expr(right);
                Expression::Binary { left: Box::new(optimized_left), op: op.clone(), right: Box::new(optimized_right) }
            }
            Expression::Unary { op, expr: inner_expr } => {
                let optimized_inner = self.eliminate_common_subexpressions_in_expr(inner_expr);
                Expression::Unary { op: op.clone(), expr: Box::new(optimized_inner) }
            }
            Expression::Call { callee, args } => {
                let optimized_callee = self.eliminate_common_subexpressions_in_expr(callee);
                let mut optimized_args = Vec::new();
                for arg in args {
                    optimized_args.push(self.eliminate_common_subexpressions_in_expr(arg));
                }
                Expression::Call { callee: Box::new(optimized_callee), args: optimized_args }
            }
            Expression::Member { object, property } => {
                let optimized_object = self.eliminate_common_subexpressions_in_expr(object);
                let optimized_property = self.eliminate_common_subexpressions_in_expr(property);
                Expression::Member { object: Box::new(optimized_object), property: Box::new(optimized_property) }
            }
            _ => expr.clone(),
        }
    }

    /// 强度削弱
    fn strength_reduction(&self, statements: &mut Vec<Statement>) {
        for stmt in statements {
            match stmt {
                Statement::Expression(expr) => {
                    *expr = Box::new(self.strength_reduction_in_expr(expr));
                }
                Statement::Block(inner_statements) => {
                    self.strength_reduction(inner_statements);
                }
                _ => {}
            }
        }
    }

    /// 在表达式中进行强度削弱
    fn strength_reduction_in_expr(&self, expr: &Expression) -> Expression {
        match expr {
            Expression::Binary { left, op, right } => {
                let optimized_left = self.strength_reduction_in_expr(left);
                let optimized_right = self.strength_reduction_in_expr(right);

                // 强度削弱优化
                if let (Expression::Literal(TsValue::Number(a)), Expression::Literal(TsValue::Number(b))) =
                    (&optimized_left, &optimized_right)
                {
                    // 例如：a * 2 可以优化为 a << 1
                    // 这里是一个简化的实现
                }

                Expression::Binary { left: Box::new(optimized_left), op: op.clone(), right: Box::new(optimized_right) }
            }
            Expression::Unary { op, expr: inner_expr } => {
                let optimized_inner = self.strength_reduction_in_expr(inner_expr);
                Expression::Unary { op: op.clone(), expr: Box::new(optimized_inner) }
            }
            _ => expr.clone(),
        }
    }

    /// 基于性能分析数据进行优化
    fn optimize_based_on_profile(&self, statements: &mut Vec<Statement>) {
        // 实现基于性能分析数据的优化
        // 例如：根据热点信息进行针对性优化
        if let Some(profile) = &self.profile_data {
            // 基于函数调用次数进行优化
            for stmt in statements {
                match stmt {
                    Statement::Expression(expr) => {
                        // 检查是否是热点表达式
                    }
                    _ => {}
                }
            }
        }
    }

    /// 常量传播
    fn propagate_constants(&self, statements: &mut Vec<Statement>, _cfg: &ControlFlowGraph) {
        // 简单的常量传播实现
        use std::collections::HashMap;

        let mut constant_map: HashMap<String, TsValue> = HashMap::new();
        let mut i = 0;

        while i < statements.len() {
            match &statements[i] {
                Statement::VariableDeclaration { name, initializer, .. } => {
                    if let Some(init) = initializer {
                        if let Some(constant_value) = init.eval() {
                            // 记录常量变量
                            constant_map.insert(name.clone(), constant_value);
                        }
                    }
                }
                Statement::Expression(expr) => {
                    // 尝试替换表达式中的常量
                    let optimized_expr = self.propagate_constants_in_expr(expr, &constant_map);
                    statements[i] = Statement::Expression(Box::new(optimized_expr));
                }
                _ => {}
            }
            i += 1;
        }
    }

    /// 在表达式中传播常量
    fn propagate_constants_in_expr(
        &self,
        expr: &Expression,
        constant_map: &std::collections::HashMap<String, TsValue>,
    ) -> Expression {
        match expr {
            Expression::Identifier(name) => {
                // 如果是常量变量，替换为常量
                if let Some(constant) = constant_map.get(name) {
                    return Expression::Literal(constant.clone());
                }
                expr.clone()
            }
            Expression::Binary { left, op, right } => {
                let optimized_left = self.propagate_constants_in_expr(left, constant_map);
                let optimized_right = self.propagate_constants_in_expr(right, constant_map);

                // 尝试常量折叠
                if let (Some(left_val), Some(right_val)) = (optimized_left.eval(), optimized_right.eval()) {
                    let result = Expression::eval_binary_op(left_val, op.clone(), right_val);
                    return Expression::Literal(result);
                }

                Expression::Binary { left: Box::new(optimized_left), op: op.clone(), right: Box::new(optimized_right) }
            }
            Expression::Unary { op, expr: inner_expr } => {
                let optimized_inner = self.propagate_constants_in_expr(inner_expr, constant_map);

                // 尝试常量折叠
                if let Some(inner_val) = optimized_inner.eval() {
                    let result = Expression::eval_unary_op(op.clone(), inner_val);
                    return Expression::Literal(result);
                }

                Expression::Unary { op: op.clone(), expr: Box::new(optimized_inner) }
            }
            Expression::Call { callee, args } => {
                let optimized_callee = self.propagate_constants_in_expr(callee, constant_map);
                let mut optimized_args = Vec::new();
                for arg in args {
                    optimized_args.push(self.propagate_constants_in_expr(arg, constant_map));
                }

                Expression::Call { callee: Box::new(optimized_callee), args: optimized_args }
            }
            Expression::Member { object, property } => {
                let optimized_object = self.propagate_constants_in_expr(object, constant_map);
                let optimized_property = self.propagate_constants_in_expr(property, constant_map);

                Expression::Member { object: Box::new(optimized_object), property: Box::new(optimized_property) }
            }
            Expression::Index { object, index } => {
                let optimized_object = self.propagate_constants_in_expr(object, constant_map);
                let optimized_index = self.propagate_constants_in_expr(index, constant_map);

                Expression::Index { object: Box::new(optimized_object), index: Box::new(optimized_index) }
            }
            Expression::Assignment { left, op, right } => {
                let optimized_left = self.propagate_constants_in_expr(left, constant_map);
                let optimized_right = self.propagate_constants_in_expr(right, constant_map);

                Expression::Assignment { left: Box::new(optimized_left), op: op.clone(), right: Box::new(optimized_right) }
            }
            Expression::Conditional { test, consequent, alternate } => {
                let optimized_test = self.propagate_constants_in_expr(test, constant_map);

                // 尝试常量折叠条件表达式
                if let Some(test_value) = optimized_test.eval() {
                    let condition = test_value.to_boolean();
                    if condition {
                        return self.propagate_constants_in_expr(consequent, constant_map);
                    }
                    else {
                        return self.propagate_constants_in_expr(alternate, constant_map);
                    }
                }

                let optimized_consequent = self.propagate_constants_in_expr(consequent, constant_map);
                let optimized_alternate = self.propagate_constants_in_expr(alternate, constant_map);

                Expression::Conditional {
                    test: Box::new(optimized_test),
                    consequent: Box::new(optimized_consequent),
                    alternate: Box::new(optimized_alternate),
                }
            }
            _ => expr.clone(),
        }
    }

    /// 死代码消除
    fn eliminate_dead_code(&self, statements: &mut Vec<Statement>, _cfg: &ControlFlowGraph) {
        // 1. 消除未使用的变量声明
        let mut i = 0;
        while i < statements.len() {
            match &statements[i] {
                Statement::VariableDeclaration { name, .. } => {
                    // 检查变量是否被使用
                    let mut is_used = false;
                    for j in i + 1..statements.len() {
                        if self.is_variable_used(&statements[j], name) {
                            is_used = true;
                            break;
                        }
                    }

                    if !is_used {
                        // 删除未使用的变量声明
                        statements.remove(i);
                        continue;
                    }
                }
                Statement::Block(inner_statements) => {
                    // 递归处理块内语句
                    let mut optimized_inner = inner_statements.clone();
                    self.eliminate_dead_code(&mut optimized_inner, _cfg);
                    if optimized_inner.is_empty() {
                        // 如果块为空，删除整个块
                        statements.remove(i);
                        continue;
                    }
                    else {
                        // 更新块内语句
                        statements[i] = Statement::Block(optimized_inner);
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }

    /// 检查变量是否在语句中被使用
    fn is_variable_used(&self, stmt: &Statement, variable_name: &str) -> bool {
        match stmt {
            Statement::Expression(expr) => self.is_variable_used_in_expr(expr, variable_name),
            Statement::VariableDeclaration { initializer, .. } => {
                if let Some(init) = initializer {
                    self.is_variable_used_in_expr(init, variable_name)
                }
                else {
                    false
                }
            }
            Statement::If { test, consequent, alternate } => {
                self.is_variable_used_in_expr(test, variable_name)
                    || self.is_variable_used(consequent, variable_name)
                    || alternate.as_ref().map(|alt| self.is_variable_used(alt, variable_name)).unwrap_or(false)
            }
            Statement::While { test, body } => {
                self.is_variable_used_in_expr(test, variable_name) || self.is_variable_used(body, variable_name)
            }
            Statement::For { init, test, update, body } => {
                init.as_ref().map(|init_stmt| self.is_variable_used(init_stmt, variable_name)).unwrap_or(false)
                    || test.as_ref().map(|test_expr| self.is_variable_used_in_expr(test_expr, variable_name)).unwrap_or(false)
                    || update
                        .as_ref()
                        .map(|update_expr| self.is_variable_used_in_expr(update_expr, variable_name))
                        .unwrap_or(false)
                    || self.is_variable_used(body, variable_name)
            }
            Statement::Return(expr) => expr.as_ref().map(|e| self.is_variable_used_in_expr(e, variable_name)).unwrap_or(false),
            Statement::Block(statements) => {
                for stmt in statements {
                    if self.is_variable_used(stmt, variable_name) {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// 检查变量是否在表达式中被使用
    fn is_variable_used_in_expr(&self, expr: &Expression, variable_name: &str) -> bool {
        match expr {
            Expression::Identifier(name) => name == variable_name,
            Expression::Binary { left, right, .. } => {
                self.is_variable_used_in_expr(left, variable_name) || self.is_variable_used_in_expr(right, variable_name)
            }
            Expression::Unary { expr: inner_expr, .. } => self.is_variable_used_in_expr(inner_expr, variable_name),
            Expression::Call { callee, args } => {
                self.is_variable_used_in_expr(callee, variable_name)
                    || args.iter().any(|arg| self.is_variable_used_in_expr(arg, variable_name))
            }
            Expression::Member { object, property } => {
                self.is_variable_used_in_expr(object, variable_name) || self.is_variable_used_in_expr(property, variable_name)
            }
            Expression::Index { object, index } => {
                self.is_variable_used_in_expr(object, variable_name) || self.is_variable_used_in_expr(index, variable_name)
            }
            Expression::Assignment { left, right, .. } => {
                self.is_variable_used_in_expr(left, variable_name) || self.is_variable_used_in_expr(right, variable_name)
            }
            Expression::Conditional { test, consequent, alternate } => {
                self.is_variable_used_in_expr(test, variable_name)
                    || self.is_variable_used_in_expr(consequent, variable_name)
                    || self.is_variable_used_in_expr(alternate, variable_name)
            }
            _ => false,
        }
    }

    /// 优化语句
    pub fn optimize_statement(&self, stmt: &Statement) -> Statement {
        match stmt {
            Statement::Expression(expr) => {
                let optimized_expr = self.optimize_expression(expr);
                Statement::Expression(Box::new(optimized_expr))
            }
            Statement::VariableDeclaration { name, ty, initializer } => {
                let optimized_initializer = initializer.as_ref().map(|init| Box::new(self.optimize_expression(init)));
                let optimized_stmt =
                    Statement::VariableDeclaration { name: name.clone(), ty: ty.clone(), initializer: optimized_initializer };

                // 对于中级及以上优化，尝试进行常量传播和死变量消除
                if self.optimization_level >= OptimizationLevel::Medium {
                    self.optimize_variable_declaration(optimized_stmt)
                }
                else {
                    optimized_stmt
                }
            }
            Statement::Block(statements) => {
                let mut optimized_statements = Vec::new();
                let mut has_return = false;

                for stmt in statements {
                    if has_return {
                        // 跳过 return 后的死代码
                        continue;
                    }

                    let optimized_stmt = self.optimize_statement(stmt);
                    if !self.is_dead_code(&optimized_stmt) {
                        optimized_statements.push(optimized_stmt.clone());
                        // 检查是否是 return 语句
                        if matches!(optimized_stmt, Statement::Return(_)) {
                            has_return = true;
                        }
                    }
                }

                Statement::Block(optimized_statements)
            }
            Statement::If { test, consequent, alternate } => {
                let optimized_test = self.optimize_expression(test);

                // 尝试常量折叠条件表达式
                if let Some(test_value) = optimized_test.eval() {
                    let condition = test_value.to_boolean();
                    if condition {
                        // 条件为真，只保留 consequent
                        return self.optimize_statement(consequent);
                    }
                    else if let Some(alt) = alternate {
                        // 条件为假，只保留 alternate
                        return self.optimize_statement(alt);
                    }
                    else {
                        // 条件为假且无 else，删除整个 if 语句
                        return Statement::Block(vec![]);
                    }
                }

                let optimized_consequent = self.optimize_statement(consequent);
                let optimized_alternate = alternate.as_ref().map(|alt| Box::new(self.optimize_statement(alt)));

                Statement::If {
                    test: Box::new(optimized_test),
                    consequent: Box::new(optimized_consequent),
                    alternate: optimized_alternate,
                }
            }
            Statement::While { test, body } => {
                let optimized_test = self.optimize_expression(test);

                // 尝试常量折叠条件表达式
                if let Some(test_value) = optimized_test.eval() {
                    let condition = test_value.to_boolean();
                    if !condition {
                        // 条件永远为假，删除整个 while 循环
                        return Statement::Block(vec![]);
                    }
                }

                let optimized_body = self.optimize_statement(body);

                Statement::While { test: Box::new(optimized_test), body: Box::new(optimized_body) }
            }
            Statement::For { init, test, update, body } => {
                let optimized_init = init.as_ref().map(|init_stmt| Box::new(self.optimize_statement(init_stmt)));
                let optimized_test = test.as_ref().map(|test_expr| Box::new(self.optimize_expression(test_expr)));
                let optimized_update = update.as_ref().map(|update_expr| Box::new(self.optimize_expression(update_expr)));
                let optimized_body = self.optimize_statement(body);

                Statement::For {
                    init: optimized_init,
                    test: optimized_test,
                    update: optimized_update,
                    body: Box::new(optimized_body),
                }
            }
            Statement::Return(expr) => {
                let optimized_expr = expr.as_ref().map(|e| Box::new(self.optimize_expression(e)));
                Statement::Return(optimized_expr)
            }
            Statement::FunctionDeclaration { name, params, return_type, body, type_params, decorators } => {
                let mut optimized_body = Vec::new();
                let mut has_return = false;

                for stmt in body {
                    if has_return {
                        // 跳过 return 后的死代码
                        continue;
                    }

                    let optimized_stmt = self.optimize_statement(stmt);
                    if !self.is_dead_code(&optimized_stmt) {
                        optimized_body.push(optimized_stmt.clone());
                        // 检查是否是 return 语句
                        if matches!(optimized_stmt, Statement::Return(_)) {
                            has_return = true;
                        }
                    }
                }

                // 对于高级优化，尝试内联小函数
                if self.optimization_level == OptimizationLevel::High && self.should_inline_function(body) {
                    // 这里可以实现函数内联逻辑
                }

                Statement::FunctionDeclaration {
                    name: name.clone(),
                    params: params.clone(),
                    return_type: return_type.clone(),
                    body: optimized_body,
                    type_params: type_params.clone(),
                    decorators: decorators.clone(),
                }
            }
            Statement::ClassDeclaration { name, super_class, methods, decorators } => {
                let mut optimized_methods = Vec::new();
                for method in methods {
                    let mut optimized_body = Vec::new();
                    let mut has_return = false;

                    for stmt in &method.body {
                        if has_return {
                            // 跳过 return 后的死代码
                            continue;
                        }

                        let optimized_stmt = self.optimize_statement(stmt);
                        if !self.is_dead_code(&optimized_stmt) {
                            optimized_body.push(optimized_stmt.clone());
                            // 检查是否是 return 语句
                            if matches!(optimized_stmt, Statement::Return(_)) {
                                has_return = true;
                            }
                        }
                    }

                    optimized_methods.push(crate::Method {
                        name: method.name.clone(),
                        type_params: method.type_params.clone(),
                        params: method.params.clone(),
                        return_type: method.return_type.clone(),
                        body: optimized_body,
                        decorators: method.decorators.clone(),
                    });
                }

                Statement::ClassDeclaration {
                    name: name.clone(),
                    super_class: super_class.clone(),
                    methods: optimized_methods,
                    decorators: decorators.clone(),
                }
            }
            _ => stmt.clone(),
        }
    }

    /// 优化变量声明
    fn optimize_variable_declaration(&self, stmt: Statement) -> Statement {
        match stmt {
            Statement::VariableDeclaration { name, ty, initializer } => {
                // 尝试常量传播
                if let Some(ref init) = initializer {
                    if let Some(constant_value) = init.eval() {
                        // 如果初始化表达式是常量，创建一个常量变量
                        // 实际的常量传播需要在整个程序范围内进行
                        // 这里我们只是标记这个变量是常量
                    }
                }
                Statement::VariableDeclaration { name, ty, initializer }
            }
            _ => stmt,
        }
    }

    /// 检查函数是否应该内联
    fn should_inline_function(&self, body: &[Statement]) -> bool {
        // 简单的启发式规则：函数体较小（少于 10 条语句）且没有复杂控制流
        body.len() < 10
            && !body.iter().any(|stmt| matches!(stmt, Statement::If { .. } | Statement::While { .. } | Statement::For { .. }))
    }

    /// 内联函数调用
    fn inline_function_call(&self, callee: &Expression, args: &[Expression]) -> Option<Expression> {
        // 目前只支持简单的函数标识符调用
        if let Expression::Identifier(name) = callee {
            // 这里需要查找函数定义，暂时返回 None
            // 实际实现需要维护函数表
            None
        }
        else {
            None
        }
    }

    /// 优化表达式
    pub fn optimize_expression(&self, expr: &Expression) -> Expression {
        match expr {
            Expression::Literal(_) => expr.clone(),
            Expression::Identifier(_) => expr.clone(),
            Expression::Binary { left, op, right } => {
                let optimized_left = self.optimize_expression(left);
                let optimized_right = self.optimize_expression(right);

                // 尝试常量折叠
                if let (Some(left_val), Some(right_val)) = (optimized_left.eval(), optimized_right.eval()) {
                    let result = Expression::eval_binary_op(left_val, op.clone(), right_val);
                    return Expression::Literal(result);
                }

                // 尝试其他优化，如算术简化
                if self.optimization_level >= OptimizationLevel::Medium {
                    self.optimize_binary_expression(optimized_left, op.clone(), optimized_right)
                }
                else {
                    Expression::Binary { left: Box::new(optimized_left), op: op.clone(), right: Box::new(optimized_right) }
                }
            }
            Expression::Array(elements) => {
                let mut optimized_elements = Vec::new();
                let mut all_constants = true;

                for elem in elements {
                    let optimized_elem = self.optimize_expression(elem);
                    optimized_elements.push(optimized_elem.clone());
                    if optimized_elem.eval().is_none() {
                        all_constants = false;
                    }
                }

                // 如果所有元素都是常量，尝试折叠
                if all_constants {
                    if let Some(array_val) = Expression::Array(optimized_elements.clone()).eval() {
                        return Expression::Literal(array_val);
                    }
                }

                Expression::Array(optimized_elements)
            }
            Expression::Object(properties) => {
                let mut optimized_properties = Vec::new();
                let mut all_constants = true;

                for (key, value) in properties {
                    let optimized_value = self.optimize_expression(value);
                    optimized_properties.push((key.clone(), optimized_value.clone()));
                    if optimized_value.eval().is_none() {
                        all_constants = false;
                    }
                }

                // 如果所有属性值都是常量，尝试折叠
                if all_constants {
                    if let Some(object_val) = Expression::Object(optimized_properties.clone()).eval() {
                        return Expression::Literal(object_val);
                    }
                }

                Expression::Object(optimized_properties)
            }
            Expression::Unary { op, expr: inner_expr } => {
                let optimized_inner = self.optimize_expression(inner_expr);

                // 尝试常量折叠
                if let Some(inner_val) = optimized_inner.eval() {
                    let result = Expression::eval_unary_op(op.clone(), inner_val);
                    return Expression::Literal(result);
                }

                Expression::Unary { op: op.clone(), expr: Box::new(optimized_inner) }
            }
            Expression::Call { callee, args } => {
                let optimized_callee = self.optimize_expression(callee);
                let mut optimized_args = Vec::new();
                for arg in args {
                    optimized_args.push(self.optimize_expression(arg));
                }

                // 尝试函数内联
                if self.optimization_level == OptimizationLevel::High {
                    if let Some(inlined_expr) = self.inline_function_call(&optimized_callee, &optimized_args) {
                        return inlined_expr;
                    }
                }

                Expression::Call { callee: Box::new(optimized_callee), args: optimized_args }
            }
            Expression::Member { object, property } => {
                let optimized_object = self.optimize_expression(object);
                let optimized_property = self.optimize_expression(property);

                Expression::Member { object: Box::new(optimized_object), property: Box::new(optimized_property) }
            }
            Expression::Index { object, index } => {
                let optimized_object = self.optimize_expression(object);
                let optimized_index = self.optimize_expression(index);

                Expression::Index { object: Box::new(optimized_object), index: Box::new(optimized_index) }
            }
            Expression::Object(properties) => {
                let mut optimized_properties = Vec::new();
                for (key, value) in properties {
                    let optimized_value = self.optimize_expression(value);
                    optimized_properties.push((key.clone(), optimized_value));
                }

                Expression::Object(optimized_properties)
            }
            Expression::Array(elements) => {
                let mut optimized_elements = Vec::new();
                for elem in elements {
                    optimized_elements.push(self.optimize_expression(elem));
                }

                Expression::Array(optimized_elements)
            }
            Expression::Assignment { left, op, right } => {
                let optimized_left = self.optimize_expression(left);
                let optimized_right = self.optimize_expression(right);

                Expression::Assignment { left: Box::new(optimized_left), op: op.clone(), right: Box::new(optimized_right) }
            }
            Expression::Conditional { test, consequent, alternate } => {
                let optimized_test = self.optimize_expression(test);

                // 尝试常量折叠条件表达式
                if let Some(test_value) = optimized_test.eval() {
                    let condition = test_value.to_boolean();
                    if condition {
                        return self.optimize_expression(consequent);
                    }
                    else {
                        return self.optimize_expression(alternate);
                    }
                }

                let optimized_consequent = self.optimize_expression(consequent);
                let optimized_alternate = self.optimize_expression(alternate);

                Expression::Conditional {
                    test: Box::new(optimized_test),
                    consequent: Box::new(optimized_consequent),
                    alternate: Box::new(optimized_alternate),
                }
            }
            Expression::Function { params, body } => {
                let mut optimized_body = Vec::new();
                let mut has_return = false;

                for stmt in body {
                    if has_return {
                        // 跳过 return 后的死代码
                        continue;
                    }

                    let optimized_stmt = self.optimize_statement(stmt);
                    if !self.is_dead_code(&optimized_stmt) {
                        optimized_body.push(optimized_stmt.clone());
                        // 检查是否是 return 语句
                        if matches!(optimized_stmt, Statement::Return(_)) {
                            has_return = true;
                        }
                    }
                }

                Expression::Function { params: params.clone(), body: optimized_body }
            }
            Expression::ArrowFunction { params, body } => {
                let optimized_body = self.optimize_expression(body);

                Expression::ArrowFunction { params: params.clone(), body: Box::new(optimized_body) }
            }
        }
    }

    /// 优化二元表达式
    fn optimize_binary_expression(&self, left: Expression, op: crate::BinaryOp, right: Expression) -> Expression {
        // 实现一些简单的算术优化
        match op {
            crate::BinaryOp::Add => {
                // 0 + x = x
                if let Expression::Literal(ref value) = left {
                    if value.to_number() == 0.0 {
                        return right;
                    }
                }
                if let Expression::Literal(ref value) = right {
                    if value.to_number() == 0.0 {
                        return left;
                    }
                }
            }
            crate::BinaryOp::Mul => {
                // 0 * x = 0
                if let Expression::Literal(ref value) = left {
                    if value.to_number() == 0.0 {
                        return left;
                    }
                }
                if let Expression::Literal(ref value) = right {
                    if value.to_number() == 0.0 {
                        return right;
                    }
                }
                // 1 * x = x
                if let Expression::Literal(ref value) = left {
                    if value.to_number() == 1.0 {
                        return right;
                    }
                }
                if let Expression::Literal(ref value) = right {
                    if value.to_number() == 1.0 {
                        return left;
                    }
                }
            }
            _ => {}
        }
        Expression::Binary { left: Box::new(left), op, right: Box::new(right) }
    }

    /// 检查语句是否为死代码
    fn is_dead_code(&self, stmt: &Statement) -> bool {
        match stmt {
            Statement::Block(statements) => statements.is_empty(),
            _ => false,
        }
    }
}

/// 执行基本优化
pub fn optimize_basic(program: &Program) -> Program {
    let optimizer = Optimizer::new(OptimizationLevel::Basic);
    optimizer.optimize_program(program)
}

/// 执行中级优化
pub fn optimize_medium(program: &Program) -> Program {
    let optimizer = Optimizer::new(OptimizationLevel::Medium);
    optimizer.optimize_program(program)
}

/// 执行高级优化
pub fn optimize_high(program: &Program) -> Program {
    let optimizer = Optimizer::new(OptimizationLevel::High);
    optimizer.optimize_program(program)
}

/// 执行所有优化
pub fn optimize_full(program: &Program) -> Program {
    let optimizer = Optimizer::new(OptimizationLevel::High);
    optimizer.optimize_program(program)
}
