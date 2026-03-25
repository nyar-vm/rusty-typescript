use crate::{ControlFlowGraph, Expression, Module, Program, Statement};

/// 优化器
pub struct Optimizer {
    /// 优化级别
    pub optimization_level: OptimizationLevel,
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
        Self { optimization_level }
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
    fn optimize_using_analysis(&self, _statements: &mut Vec<Statement>, _cfg: &ControlFlowGraph) {
        // 这里可以实现基于到达定义和活跃变量的优化
        // 例如：死代码消除、常量传播、变量重命名等
        // 由于这是一个示例，我们暂时只做简单的死代码消除
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
                    if let Some(_constant_value) = init.eval() {
                        // 如果初始化表达式是常量，尝试在后续使用中替换
                        // 这里可以实现常量传播逻辑
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
