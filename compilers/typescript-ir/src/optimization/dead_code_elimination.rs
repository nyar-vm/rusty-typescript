use crate::{ControlFlowGraph, Expression, Statement};

/// 死代码消除优化器
pub struct DeadCodeElimination;

impl DeadCodeElimination {
    /// 消除死代码
    pub fn eliminate(statements: &mut Vec<Statement>, _cfg: &ControlFlowGraph) {
        let mut i = 0;
        while i < statements.len() {
            match &statements[i] {
                Statement::VariableDeclaration { name, .. } => {
                    let mut is_used = false;
                    for j in i + 1..statements.len() {
                        if Self::is_variable_used(&statements[j], name) {
                            is_used = true;
                            break;
                        }
                    }

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
                _ => {}
            }
            i += 1;
        }
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
                for stmt in statements {
                    if Self::is_variable_used(stmt, variable_name) {
                        return true;
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
            _ => false,
        }
    }

    /// 检查语句是否为死代码
    pub fn is_dead_code(stmt: &Statement) -> bool {
        match stmt {
            Statement::Block(statements) => statements.is_empty(),
            _ => false,
        }
    }
}
