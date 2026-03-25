//! TypeScript IR 表达式模块

use typescript_types::TsValue;

use crate::types::{AssignmentOp, BinaryOp, TypeAnnotation, UnaryOp};

/// TypeScript 中间表示 - 表达式
#[derive(Debug, Clone)]
pub enum Expression {
    /// 字面量
    Literal(TsValue),
    /// 标识符
    Identifier(String),
    /// 二元表达式
    Binary { left: Box<Expression>, op: BinaryOp, right: Box<Expression> },
    /// 一元表达式
    Unary { op: UnaryOp, expr: Box<Expression> },
    /// 函数调用
    Call { callee: Box<Expression>, args: Vec<Expression> },
    /// 成员访问
    Member { object: Box<Expression>, property: Box<Expression> },
    /// 数组访问
    Index { object: Box<Expression>, index: Box<Expression> },
    /// 对象字面量
    Object(Vec<(String, Expression)>),
    /// 数组字面量
    Array(Vec<Expression>),
    /// 赋值表达式
    Assignment { left: Box<Expression>, op: AssignmentOp, right: Box<Expression> },
    /// 条件表达式
    Conditional { test: Box<Expression>, consequent: Box<Expression>, alternate: Box<Expression> },
    /// 函数表达式
    Function { params: Vec<String>, body: Vec<crate::statement::Statement> },
    /// 箭头函数
    ArrowFunction { params: Vec<String>, body: Box<Expression> },
}

/// TypeScript 中间表示 - 表达式（带类型信息）
#[derive(Debug, Clone)]
pub struct TypedExpression {
    /// 表达式
    pub expr: Expression,
    /// 类型注解
    pub ty: TypeAnnotation,
    /// 是否为常量
    pub is_constant: bool,
    /// 常量值（如果是常量）
    pub constant_value: Option<TsValue>,
}

impl TypedExpression {
    /// 创建一个新的带类型信息的表达式
    pub fn new(expr: Expression, ty: TypeAnnotation) -> Self {
        let constant_value = expr.eval();
        let is_constant = constant_value.is_some();

        Self { expr, ty, is_constant, constant_value }
    }

    /// 优化表达式
    pub fn optimize(&self) -> Self {
        let optimized_expr = match &self.expr {
            Expression::Binary { left, op, right } => {
                let optimized_left = TypedExpression::new(*left.clone(), self.ty.clone()).optimize();
                let optimized_right = TypedExpression::new(*right.clone(), self.ty.clone()).optimize();

                if optimized_left.is_constant && optimized_right.is_constant {
                    if let (Some(left_val), Some(right_val)) = (optimized_left.constant_value, optimized_right.constant_value) {
                        let result = Expression::eval_binary_op(left_val, op.clone(), right_val);
                        Expression::Literal(result)
                    }
                    else {
                        Expression::Binary {
                            left: Box::new(optimized_left.expr),
                            op: op.clone(),
                            right: Box::new(optimized_right.expr),
                        }
                    }
                }
                else {
                    Expression::Binary {
                        left: Box::new(optimized_left.expr),
                        op: op.clone(),
                        right: Box::new(optimized_right.expr),
                    }
                }
            }
            Expression::Unary { op, expr: inner_expr } => {
                let optimized_inner = TypedExpression::new(*inner_expr.clone(), self.ty.clone()).optimize();

                if optimized_inner.is_constant {
                    if let Some(inner_val) = optimized_inner.constant_value {
                        let result = Expression::eval_unary_op(op.clone(), inner_val);
                        Expression::Literal(result)
                    }
                    else {
                        Expression::Unary { op: op.clone(), expr: Box::new(optimized_inner.expr) }
                    }
                }
                else {
                    Expression::Unary { op: op.clone(), expr: Box::new(optimized_inner.expr) }
                }
            }
            _ => self.expr.clone(),
        };

        Self::new(optimized_expr, self.ty.clone())
    }
}

impl Expression {
    /// 尝试计算表达式的值（用于常量折叠）
    pub fn eval(&self) -> Option<TsValue> {
        match self {
            Expression::Literal(value) => Some(value.clone()),
            Expression::Binary { left, op, right } => {
                if let (Some(left_val), Some(right_val)) = (left.eval(), right.eval()) {
                    Some(Self::eval_binary_op(left_val, op.clone(), right_val))
                }
                else {
                    None
                }
            }
            Expression::Unary { op, expr } => {
                if let Some(expr_val) = expr.eval() {
                    Some(Self::eval_unary_op(op.clone(), expr_val))
                }
                else {
                    None
                }
            }
            Expression::Conditional { test, consequent, alternate } => {
                if let Some(test_val) = test.eval() {
                    if test_val.to_boolean() { consequent.eval() } else { alternate.eval() }
                }
                else {
                    None
                }
            }
            Expression::Array(elements) => {
                let mut evaluated = Vec::new();
                for elem in elements {
                    if let Some(val) = elem.eval() {
                        evaluated.push(val);
                    }
                    else {
                        return None;
                    }
                }
                Some(TsValue::Array(evaluated))
            }
            Expression::Object(properties) => {
                let mut evaluated = Vec::new();
                for (key, value) in properties {
                    if let Some(val) = value.eval() {
                        evaluated.push((key.clone(), val));
                    }
                    else {
                        return None;
                    }
                }
                Some(TsValue::Object(evaluated))
            }
            _ => None, // 其他表达式类型无法在编译时求值
        }
    }

    /// 计算二元操作
    pub fn eval_binary_op(left: TsValue, op: BinaryOp, right: TsValue) -> TsValue {
        match op {
            BinaryOp::Add => {
                if left.is_string() || right.is_string() {
                    TsValue::String(format!("{}{}", left.to_string(), right.to_string()))
                }
                else {
                    TsValue::Number(left.to_number() + right.to_number())
                }
            }
            BinaryOp::Sub => TsValue::Number(left.to_number() - right.to_number()),
            BinaryOp::Mul => TsValue::Number(left.to_number() * right.to_number()),
            BinaryOp::Div => TsValue::Number(left.to_number() / right.to_number()),
            BinaryOp::Mod => TsValue::Number(left.to_number() % right.to_number()),
            BinaryOp::Eq => TsValue::Boolean(left.to_string() == right.to_string()),
            BinaryOp::Neq => TsValue::Boolean(left.to_string() != right.to_string()),
            BinaryOp::StrictEq => TsValue::Boolean(
                left.to_string() == right.to_string()
                    && left.is_number() == right.is_number()
                    && left.is_string() == right.is_string()
                    && left.is_boolean() == right.is_boolean(),
            ),
            BinaryOp::StrictNeq => TsValue::Boolean(
                left.to_string() != right.to_string()
                    || left.is_number() != right.is_number()
                    || left.is_string() != right.is_string()
                    || left.is_boolean() != right.is_boolean(),
            ),
            BinaryOp::Gt => TsValue::Boolean(left.to_number() > right.to_number()),
            BinaryOp::Gte => TsValue::Boolean(left.to_number() >= right.to_number()),
            BinaryOp::Lt => TsValue::Boolean(left.to_number() < right.to_number()),
            BinaryOp::Lte => TsValue::Boolean(left.to_number() <= right.to_number()),
            BinaryOp::And => TsValue::Boolean(left.to_boolean() && right.to_boolean()),
            BinaryOp::Or => TsValue::Boolean(left.to_boolean() || right.to_boolean()),
            BinaryOp::BitAnd => TsValue::Number((left.to_number() as i64 & right.to_number() as i64) as f64),
            BinaryOp::BitOr => TsValue::Number((left.to_number() as i64 | right.to_number() as i64) as f64),
            BinaryOp::BitXor => TsValue::Number((left.to_number() as i64 ^ right.to_number() as i64) as f64),
            BinaryOp::Shl => TsValue::Number(((left.to_number() as i64) << (right.to_number() as u32)) as f64),
            BinaryOp::Shr => TsValue::Number(((left.to_number() as i64) >> (right.to_number() as u32)) as f64),
            BinaryOp::UShr => TsValue::Number(((left.to_number() as u64) >> (right.to_number() as u32)) as f64),
            BinaryOp::Pow => TsValue::Number(left.to_number().powf(right.to_number())),
        }
    }

    /// 计算一元操作
    pub fn eval_unary_op(op: UnaryOp, expr: TsValue) -> TsValue {
        match op {
            UnaryOp::Not => TsValue::Boolean(!expr.to_boolean()),
            UnaryOp::Neg => TsValue::Number(-expr.to_number()),
            UnaryOp::Pos => TsValue::Number(expr.to_number()),
            UnaryOp::BitNot => TsValue::Number(!(expr.to_number() as i64) as f64),
            UnaryOp::Inc => TsValue::Number(expr.to_number() + 1.0),
            UnaryOp::Dec => TsValue::Number(expr.to_number() - 1.0),
            UnaryOp::TypeOf => TsValue::String(if expr.is_undefined() {
                "undefined".to_string()
            }
            else if expr.is_null() {
                "object".to_string()
            }
            else if expr.is_boolean() {
                "boolean".to_string()
            }
            else if expr.is_number() {
                "number".to_string()
            }
            else if expr.is_string() {
                "string".to_string()
            }
            else if expr.is_function() {
                "function".to_string()
            }
            else if expr.is_array() {
                "object".to_string()
            }
            else {
                "object".to_string()
            }),
            UnaryOp::Void => TsValue::Undefined,
            UnaryOp::Delete => TsValue::Boolean(true), // 简化处理
        }
    }
}
