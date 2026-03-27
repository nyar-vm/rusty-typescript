use crate::{ControlFlowGraph, Expression, Statement};
use std::collections::HashMap;
use typescript_types::TsValue;

/// 常量传播优化器
pub struct ConstantPropagation;

impl ConstantPropagation {
    /// 常量传播
    pub fn propagate(statements: &mut Vec<Statement>, _cfg: &ControlFlowGraph) {
        let mut constant_map: HashMap<String, TsValue> = HashMap::new();
        let mut i = 0;

        while i < statements.len() {
            match &statements[i] {
                Statement::VariableDeclaration { name, initializer, .. } => {
                    if let Some(init) = initializer {
                        if let Some(constant_value) = init.eval() {
                            constant_map.insert(name.clone(), constant_value);
                        }
                    }
                }
                Statement::Expression(expr) => {
                    let optimized_expr = Self::propagate_in_expr(expr, &constant_map);
                    statements[i] = Statement::Expression(Box::new(optimized_expr));
                }
                _ => {}
            }
            i += 1;
        }
    }

    /// 在表达式中传播常量
    pub fn propagate_in_expr(expr: &Expression, constant_map: &HashMap<String, TsValue>) -> Expression {
        match expr {
            Expression::Identifier(name) => {
                if let Some(constant) = constant_map.get(name) {
                    return Expression::Literal(constant.clone());
                }
                expr.clone()
            }
            Expression::Binary { left, op, right } => {
                let optimized_left = Self::propagate_in_expr(left, constant_map);
                let optimized_right = Self::propagate_in_expr(right, constant_map);

                if let (Some(left_val), Some(right_val)) = (optimized_left.eval(), optimized_right.eval()) {
                    let result = Expression::eval_binary_op(left_val, op.clone(), right_val);
                    return Expression::Literal(result);
                }

                Expression::Binary { left: Box::new(optimized_left), op: op.clone(), right: Box::new(optimized_right) }
            }
            Expression::Unary { op, expr: inner_expr } => {
                let optimized_inner = Self::propagate_in_expr(inner_expr, constant_map);

                if let Some(inner_val) = optimized_inner.eval() {
                    let result = Expression::eval_unary_op(op.clone(), inner_val);
                    return Expression::Literal(result);
                }

                Expression::Unary { op: op.clone(), expr: Box::new(optimized_inner) }
            }
            Expression::Call { callee, args } => {
                let optimized_callee = Self::propagate_in_expr(callee, constant_map);
                let mut optimized_args = Vec::new();
                for arg in args {
                    optimized_args.push(Self::propagate_in_expr(arg, constant_map));
                }

                Expression::Call { callee: Box::new(optimized_callee), args: optimized_args }
            }
            Expression::Member { object, property } => {
                let optimized_object = Self::propagate_in_expr(object, constant_map);
                let optimized_property = Self::propagate_in_expr(property, constant_map);

                Expression::Member { object: Box::new(optimized_object), property: Box::new(optimized_property) }
            }
            Expression::Index { object, index } => {
                let optimized_object = Self::propagate_in_expr(object, constant_map);
                let optimized_index = Self::propagate_in_expr(index, constant_map);

                Expression::Index { object: Box::new(optimized_object), index: Box::new(optimized_index) }
            }
            Expression::Assignment { left, op, right } => {
                let optimized_left = Self::propagate_in_expr(left, constant_map);
                let optimized_right = Self::propagate_in_expr(right, constant_map);

                Expression::Assignment { left: Box::new(optimized_left), op: op.clone(), right: Box::new(optimized_right) }
            }
            Expression::Conditional { test, consequent, alternate } => {
                let optimized_test = Self::propagate_in_expr(test, constant_map);

                if let Some(test_value) = optimized_test.eval() {
                    let condition = test_value.to_boolean();
                    if condition {
                        return Self::propagate_in_expr(consequent, constant_map);
                    }
                    else {
                        return Self::propagate_in_expr(alternate, constant_map);
                    }
                }

                let optimized_consequent = Self::propagate_in_expr(consequent, constant_map);
                let optimized_alternate = Self::propagate_in_expr(alternate, constant_map);

                Expression::Conditional {
                    test: Box::new(optimized_test),
                    consequent: Box::new(optimized_consequent),
                    alternate: Box::new(optimized_alternate),
                }
            }
            _ => expr.clone(),
        }
    }
}
