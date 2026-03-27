use crate::{BinaryOp, Expression, Statement};
use typescript_types::TsValue;

/// 表达式优化器
pub struct ExpressionOptimization;

impl ExpressionOptimization {
    /// 公共子表达式消除
    pub fn eliminate_common_subexpressions(statements: &mut Vec<Statement>) {
        for stmt in statements {
            match stmt {
                Statement::Expression(expr) => {
                    *expr = Box::new(Self::eliminate_common_subexpressions_in_expr(expr));
                }
                Statement::Block(inner_statements) => {
                    Self::eliminate_common_subexpressions(inner_statements);
                }
                _ => {}
            }
        }
    }

    /// 在表达式中消除公共子表达式
    pub fn eliminate_common_subexpressions_in_expr(expr: &Expression) -> Expression {
        match expr {
            Expression::Binary { left, op, right } => {
                let optimized_left = Self::eliminate_common_subexpressions_in_expr(left);
                let optimized_right = Self::eliminate_common_subexpressions_in_expr(right);
                Expression::Binary { left: Box::new(optimized_left), op: op.clone(), right: Box::new(optimized_right) }
            }
            Expression::Unary { op, expr: inner_expr } => {
                let optimized_inner = Self::eliminate_common_subexpressions_in_expr(inner_expr);
                Expression::Unary { op: op.clone(), expr: Box::new(optimized_inner) }
            }
            Expression::Call { callee, args } => {
                let optimized_callee = Self::eliminate_common_subexpressions_in_expr(callee);
                let mut optimized_args = Vec::new();
                for arg in args {
                    optimized_args.push(Self::eliminate_common_subexpressions_in_expr(arg));
                }
                Expression::Call { callee: Box::new(optimized_callee), args: optimized_args }
            }
            Expression::Member { object, property } => {
                let optimized_object = Self::eliminate_common_subexpressions_in_expr(object);
                let optimized_property = Self::eliminate_common_subexpressions_in_expr(property);
                Expression::Member { object: Box::new(optimized_object), property: Box::new(optimized_property) }
            }
            _ => expr.clone(),
        }
    }

    /// 强度削弱
    pub fn strength_reduction(statements: &mut Vec<Statement>) {
        for stmt in statements {
            match stmt {
                Statement::Expression(expr) => {
                    *expr = Box::new(Self::strength_reduction_in_expr(expr));
                }
                Statement::Block(inner_statements) => {
                    Self::strength_reduction(inner_statements);
                }
                _ => {}
            }
        }
    }

    /// 在表达式中进行强度削弱
    pub fn strength_reduction_in_expr(expr: &Expression) -> Expression {
        match expr {
            Expression::Binary { left, op, right } => {
                let optimized_left = Self::strength_reduction_in_expr(left);
                let optimized_right = Self::strength_reduction_in_expr(right);

                if let (Expression::Literal(TsValue::Number(a)), Expression::Literal(TsValue::Number(b))) =
                    (&optimized_left, &optimized_right)
                {
                    let _ = (a, b);
                }

                Expression::Binary { left: Box::new(optimized_left), op: op.clone(), right: Box::new(optimized_right) }
            }
            Expression::Unary { op, expr: inner_expr } => {
                let optimized_inner = Self::strength_reduction_in_expr(inner_expr);
                Expression::Unary { op: op.clone(), expr: Box::new(optimized_inner) }
            }
            _ => expr.clone(),
        }
    }

    /// 优化二元表达式
    pub fn optimize_binary(left: Expression, op: BinaryOp, right: Expression) -> Expression {
        match op {
            BinaryOp::Add => {
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
            BinaryOp::Mul => {
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

    /// 优化表达式
    pub fn optimize(expr: &Expression, optimization_level: crate::OptimizationLevel) -> Expression {
        match expr {
            Expression::Literal(_) => expr.clone(),
            Expression::Identifier(_) => expr.clone(),
            Expression::Binary { left, op, right } => {
                let optimized_left = Self::optimize(left, optimization_level);
                let optimized_right = Self::optimize(right, optimization_level);

                if let (Some(left_val), Some(right_val)) = (optimized_left.eval(), optimized_right.eval()) {
                    let result = Expression::eval_binary_op(left_val, op.clone(), right_val);
                    return Expression::Literal(result);
                }

                if optimization_level >= crate::OptimizationLevel::Medium {
                    Self::optimize_binary(optimized_left, op.clone(), optimized_right)
                }
                else {
                    Expression::Binary { left: Box::new(optimized_left), op: op.clone(), right: Box::new(optimized_right) }
                }
            }
            Expression::Array(elements) => {
                let mut optimized_elements = Vec::new();
                let mut all_constants = true;

                for elem in elements {
                    let optimized_elem = Self::optimize(elem, optimization_level);
                    optimized_elements.push(optimized_elem.clone());
                    if optimized_elem.eval().is_none() {
                        all_constants = false;
                    }
                }

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
                    let optimized_value = Self::optimize(value, optimization_level);
                    optimized_properties.push((key.clone(), optimized_value.clone()));
                    if optimized_value.eval().is_none() {
                        all_constants = false;
                    }
                }

                if all_constants {
                    if let Some(object_val) = Expression::Object(optimized_properties.clone()).eval() {
                        return Expression::Literal(object_val);
                    }
                }

                Expression::Object(optimized_properties)
            }
            Expression::Unary { op, expr: inner_expr } => {
                let optimized_inner = Self::optimize(inner_expr, optimization_level);

                if let Some(inner_val) = optimized_inner.eval() {
                    let result = Expression::eval_unary_op(op.clone(), inner_val);
                    return Expression::Literal(result);
                }

                Expression::Unary { op: op.clone(), expr: Box::new(optimized_inner) }
            }
            Expression::Call { callee, args } => {
                let optimized_callee = Self::optimize(callee, optimization_level);
                let mut optimized_args = Vec::new();
                for arg in args {
                    optimized_args.push(Self::optimize(arg, optimization_level));
                }

                Expression::Call { callee: Box::new(optimized_callee), args: optimized_args }
            }
            Expression::Member { object, property } => {
                let optimized_object = Self::optimize(object, optimization_level);
                let optimized_property = Self::optimize(property, optimization_level);

                Expression::Member { object: Box::new(optimized_object), property: Box::new(optimized_property) }
            }
            Expression::Index { object, index } => {
                let optimized_object = Self::optimize(object, optimization_level);
                let optimized_index = Self::optimize(index, optimization_level);

                Expression::Index { object: Box::new(optimized_object), index: Box::new(optimized_index) }
            }
            Expression::Assignment { left, op, right } => {
                let optimized_left = Self::optimize(left, optimization_level);
                let optimized_right = Self::optimize(right, optimization_level);

                Expression::Assignment { left: Box::new(optimized_left), op: op.clone(), right: Box::new(optimized_right) }
            }
            Expression::Conditional { test, consequent, alternate } => {
                let optimized_test = Self::optimize(test, optimization_level);

                if let Some(test_value) = optimized_test.eval() {
                    let condition = test_value.to_boolean();
                    if condition {
                        return Self::optimize(consequent, optimization_level);
                    }
                    else {
                        return Self::optimize(alternate, optimization_level);
                    }
                }

                let optimized_consequent = Self::optimize(consequent, optimization_level);
                let optimized_alternate = Self::optimize(alternate, optimization_level);

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
                        continue;
                    }

                    let optimized_stmt =
                        crate::optimization::statement_optimization::optimize_statement(stmt, optimization_level);
                    if !crate::optimization::dead_code_elimination::DeadCodeElimination::is_dead_code(&optimized_stmt) {
                        optimized_body.push(optimized_stmt.clone());
                        if matches!(optimized_stmt, Statement::Return(_)) {
                            has_return = true;
                        }
                    }
                }

                Expression::Function { params: params.clone(), body: optimized_body }
            }
            Expression::ArrowFunction { params, body } => {
                let optimized_body = Self::optimize(body, optimization_level);

                Expression::ArrowFunction { params: params.clone(), body: Box::new(optimized_body) }
            }
        }
    }
}
