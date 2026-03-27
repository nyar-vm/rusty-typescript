use crate::{Expression, Statement};
use std::collections::HashSet;
use typescript_types::TsValue;

/// 循环优化器
pub struct LoopOptimization;

impl LoopOptimization {
    /// 循环不变式外提
    pub fn hoist_loop_invariants(statements: &mut Vec<Statement>) {
        let mut i = 0;
        while i < statements.len() {
            match &mut statements[i] {
                Statement::While { test, body } => {
                    let invariants = Self::find_loop_invariants(body, test);
                    if !invariants.is_empty() {
                        let invariants_clone = invariants.clone();
                        statements.splice(i..i, invariants);
                        i += invariants_clone.len();
                    }
                }
                Statement::For { init, test, update, body } => {
                    let invariants = Self::find_loop_invariants(
                        body,
                        test.as_deref().unwrap_or(&Expression::Literal(TsValue::Boolean(true))),
                    );
                    if !invariants.is_empty() {
                        let invariants_clone = invariants.clone();
                        statements.splice(i..i, invariants);
                        i += invariants_clone.len();
                    }
                }
                Statement::Block(inner_statements) => {
                    Self::hoist_loop_invariants(inner_statements);
                }
                _ => {}
            }
            i += 1;
        }
    }

    /// 查找循环不变式
    pub fn find_loop_invariants(body: &Statement, test: &Expression) -> Vec<Statement> {
        let mut invariants = Vec::new();
        let loop_variables = Self::find_loop_variables(body, test);

        if let Statement::Block(inner_statements) = body {
            for stmt in inner_statements {
                if let Statement::VariableDeclaration { name, ty, initializer } = stmt {
                    if let Some(init) = initializer {
                        if !Self::depends_on_variables(init, &loop_variables) {
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
    pub fn find_loop_variables(body: &Statement, test: &Expression) -> HashSet<String> {
        let mut variables = HashSet::new();
        Self::collect_variables(body, &mut variables);
        Self::collect_variables_in_expr(test, &mut variables);
        variables
    }

    /// 收集表达式中使用的变量
    pub fn collect_variables_in_expr(expr: &Expression, variables: &mut HashSet<String>) {
        match expr {
            Expression::Identifier(name) => {
                variables.insert(name.clone());
            }
            Expression::Binary { left, right, .. } => {
                Self::collect_variables_in_expr(left, variables);
                Self::collect_variables_in_expr(right, variables);
            }
            Expression::Unary { expr: inner_expr, .. } => {
                Self::collect_variables_in_expr(inner_expr, variables);
            }
            Expression::Call { callee, args } => {
                Self::collect_variables_in_expr(callee, variables);
                for arg in args {
                    Self::collect_variables_in_expr(arg, variables);
                }
            }
            Expression::Member { object, property } => {
                Self::collect_variables_in_expr(object, variables);
                Self::collect_variables_in_expr(property, variables);
            }
            _ => {}
        }
    }

    /// 收集语句中使用的变量
    pub fn collect_variables(stmt: &Statement, variables: &mut HashSet<String>) {
        match stmt {
            Statement::VariableDeclaration { name, initializer, .. } => {
                variables.insert(name.clone());
                if let Some(init) = initializer {
                    Self::collect_variables_in_expr(init, variables);
                }
            }
            Statement::Expression(expr) => {
                Self::collect_variables_in_expr(expr, variables);
            }
            Statement::If { test, consequent, alternate } => {
                Self::collect_variables_in_expr(test, variables);
                Self::collect_variables(consequent, variables);
                if let Some(alt) = alternate {
                    Self::collect_variables(alt, variables);
                }
            }
            Statement::While { test, body } => {
                Self::collect_variables_in_expr(test, variables);
                Self::collect_variables(body, variables);
            }
            Statement::For { init, test, update, body } => {
                if let Some(init_stmt) = init {
                    Self::collect_variables(init_stmt, variables);
                }
                if let Some(test_expr) = test {
                    Self::collect_variables_in_expr(test_expr, variables);
                }
                if let Some(update_expr) = update {
                    Self::collect_variables_in_expr(update_expr, variables);
                }
                Self::collect_variables(body, variables);
            }
            Statement::Return(expr) => {
                if let Some(e) = expr {
                    Self::collect_variables_in_expr(e, variables);
                }
            }
            Statement::Block(statements) => {
                for stmt in statements {
                    Self::collect_variables(stmt, variables);
                }
            }
            _ => {}
        }
    }

    /// 检查表达式是否依赖指定变量
    pub fn depends_on_variables(expr: &Expression, variables: &HashSet<String>) -> bool {
        match expr {
            Expression::Identifier(name) => variables.contains(name),
            Expression::Binary { left, right, .. } => {
                Self::depends_on_variables(left, variables) || Self::depends_on_variables(right, variables)
            }
            Expression::Unary { expr: inner_expr, .. } => Self::depends_on_variables(inner_expr, variables),
            Expression::Call { callee, args } => {
                Self::depends_on_variables(callee, variables)
                    || args.iter().any(|arg| Self::depends_on_variables(arg, variables))
            }
            Expression::Member { object, property } => {
                Self::depends_on_variables(object, variables) || Self::depends_on_variables(property, variables)
            }
            _ => false,
        }
    }
}
