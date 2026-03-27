use crate::{
    Expression, Method, OptimizationLevel, Statement,
    optimization::{dead_code_elimination::DeadCodeElimination, expression_optimization::ExpressionOptimization},
};

/// 语句优化器
pub struct StatementOptimization;

/// 优化语句
pub fn optimize_statement(stmt: &Statement, optimization_level: OptimizationLevel) -> Statement {
    match stmt {
        Statement::Expression(expr) => {
            let optimized_expr = ExpressionOptimization::optimize(expr, optimization_level);
            Statement::Expression(Box::new(optimized_expr))
        }
        Statement::VariableDeclaration { name, ty, initializer } => {
            let optimized_initializer =
                initializer.as_ref().map(|init| Box::new(ExpressionOptimization::optimize(init, optimization_level)));
            let optimized_stmt =
                Statement::VariableDeclaration { name: name.clone(), ty: ty.clone(), initializer: optimized_initializer };

            if optimization_level >= OptimizationLevel::Medium {
                optimize_variable_declaration(optimized_stmt)
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
                    continue;
                }

                let optimized_stmt = optimize_statement(stmt, optimization_level);
                if !DeadCodeElimination::is_dead_code(&optimized_stmt) {
                    optimized_statements.push(optimized_stmt.clone());
                    if matches!(optimized_stmt, Statement::Return(_)) {
                        has_return = true;
                    }
                }
            }

            Statement::Block(optimized_statements)
        }
        Statement::If { test, consequent, alternate } => {
            let optimized_test = ExpressionOptimization::optimize(test, optimization_level);

            if let Some(test_value) = optimized_test.eval() {
                let condition = test_value.to_boolean();
                if condition {
                    return optimize_statement(consequent, optimization_level);
                }
                else if let Some(alt) = alternate {
                    return optimize_statement(alt, optimization_level);
                }
                else {
                    return Statement::Block(vec![]);
                }
            }

            let optimized_consequent = optimize_statement(consequent, optimization_level);
            let optimized_alternate = alternate.as_ref().map(|alt| Box::new(optimize_statement(alt, optimization_level)));

            Statement::If {
                test: Box::new(optimized_test),
                consequent: Box::new(optimized_consequent),
                alternate: optimized_alternate,
            }
        }
        Statement::While { test, body } => {
            let optimized_test = ExpressionOptimization::optimize(test, optimization_level);

            if let Some(test_value) = optimized_test.eval() {
                let condition = test_value.to_boolean();
                if !condition {
                    return Statement::Block(vec![]);
                }
            }

            let optimized_body = optimize_statement(body, optimization_level);

            Statement::While { test: Box::new(optimized_test), body: Box::new(optimized_body) }
        }
        Statement::For { init, test, update, body } => {
            let optimized_init = init.as_ref().map(|init_stmt| Box::new(optimize_statement(init_stmt, optimization_level)));
            let optimized_test =
                test.as_ref().map(|test_expr| Box::new(ExpressionOptimization::optimize(test_expr, optimization_level)));
            let optimized_update =
                update.as_ref().map(|update_expr| Box::new(ExpressionOptimization::optimize(update_expr, optimization_level)));
            let optimized_body = optimize_statement(body, optimization_level);

            Statement::For {
                init: optimized_init,
                test: optimized_test,
                update: optimized_update,
                body: Box::new(optimized_body),
            }
        }
        Statement::Return(expr) => {
            let optimized_expr = expr.as_ref().map(|e| Box::new(ExpressionOptimization::optimize(e, optimization_level)));
            Statement::Return(optimized_expr)
        }
        Statement::FunctionDeclaration { name, params, return_type, body, type_params, decorators } => {
            let mut optimized_body = Vec::new();
            let mut has_return = false;

            for stmt in body {
                if has_return {
                    continue;
                }

                let optimized_stmt = optimize_statement(stmt, optimization_level);
                if !DeadCodeElimination::is_dead_code(&optimized_stmt) {
                    optimized_body.push(optimized_stmt.clone());
                    if matches!(optimized_stmt, Statement::Return(_)) {
                        has_return = true;
                    }
                }
            }

            if optimization_level == OptimizationLevel::High && should_inline_function(body) {}

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
                        continue;
                    }

                    let optimized_stmt = optimize_statement(stmt, optimization_level);
                    if !DeadCodeElimination::is_dead_code(&optimized_stmt) {
                        optimized_body.push(optimized_stmt.clone());
                        if matches!(optimized_stmt, Statement::Return(_)) {
                            has_return = true;
                        }
                    }
                }

                optimized_methods.push(Method {
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
fn optimize_variable_declaration(stmt: Statement) -> Statement {
    match stmt {
        Statement::VariableDeclaration { name, ty, initializer } => {
            if let Some(ref init) = initializer {
                if let Some(_constant_value) = init.eval() {}
            }
            Statement::VariableDeclaration { name, ty, initializer }
        }
        _ => stmt,
    }
}

/// 检查函数是否应该内联
fn should_inline_function(body: &[Statement]) -> bool {
    body.len() < 10
        && !body.iter().any(|stmt| matches!(stmt, Statement::If { .. } | Statement::While { .. } | Statement::For { .. }))
}

/// 内联函数调用
pub fn inline_function_call(callee: &Expression, _args: &[Expression]) -> Option<Expression> {
    if let Expression::Identifier(_name) = callee { None } else { None }
}
