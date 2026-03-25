//! 从源代码转换为 IR

use typescript_types::TsError;

use crate::{
    expression::Expression,
    program::Program,
    statement::Statement,
    types::{AssignmentOp, BinaryOp, TypeAnnotation, TypeParameter, UnaryOp},
};

/// 简化的表达式转换函数
pub fn expression(source: &str) -> Result<Expression, TsError> {
    // 简化实现，返回一个基本的表达式
    Ok(Expression::Literal(typescript_types::TsValue::String(source.to_string())))
}

/// 简化的类型注解转换函数
pub fn type_annotation(_source: &str) -> Result<TypeAnnotation, TsError> {
    // 简化实现，暂时返回 Any 类型
    Ok(TypeAnnotation::Any)
}

/// 简化的语句转换函数
pub fn statement(source: &str) -> Result<Statement, TsError> {
    // 简化实现，返回一个基本的表达式语句
    let expr = expression(source)?;
    Ok(Statement::Expression(Box::new(expr)))
}

/// 简化的程序转换函数
pub fn program(source: &str) -> Result<Program, TsError> {
    // 简化实现，返回一个空的程序
    Ok(Program { statements: vec![] })
}
