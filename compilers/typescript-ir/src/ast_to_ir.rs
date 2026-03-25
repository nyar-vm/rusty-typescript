//! 从 oak-typescript AST 转换为 IR

use oak_typescript::ast;
use typescript_types::TsError;

use crate::{
    expression::Expression,
    program::Program,
    statement::Statement,
    types::{AssignmentOp, BinaryOp, TypeAnnotation, TypeParameter, UnaryOp},
};

/// 将 oak-typescript Expression 转换为 IR Expression
pub fn expression(ast_expr: &ast::Expression) -> Result<Expression, TsError> {
    match &*ast_expr.kind {
        ast::ExpressionKind::Identifier(name) => Ok(Expression::Identifier(name.clone())),
        ast::ExpressionKind::NumericLiteral(value) => Ok(Expression::Literal(typescript_types::TsValue::Number(*value))),
        ast::ExpressionKind::StringLiteral(value) => Ok(Expression::Literal(typescript_types::TsValue::String(value.clone()))),
        ast::ExpressionKind::BooleanLiteral(value) => Ok(Expression::Literal(typescript_types::TsValue::Boolean(*value))),
        ast::ExpressionKind::NullLiteral => Ok(Expression::Literal(typescript_types::TsValue::Null)),
        ast::ExpressionKind::BinaryExpression { left, operator, right } => {
            let binary_op = match operator.as_str() {
                "+" => BinaryOp::Add,
                "-" => BinaryOp::Sub,
                "*" => BinaryOp::Mul,
                "/" => BinaryOp::Div,
                "%" => BinaryOp::Mod,
                "==" => BinaryOp::Eq,
                "!=" => BinaryOp::Neq,
                "===" => BinaryOp::StrictEq,
                "!==" => BinaryOp::StrictNeq,
                ">" => BinaryOp::Gt,
                ">=" => BinaryOp::Gte,
                "<" => BinaryOp::Lt,
                "<=" => BinaryOp::Lte,
                "&&" => BinaryOp::And,
                "||" => BinaryOp::Or,
                "&" => BinaryOp::BitAnd,
                "|" => BinaryOp::BitOr,
                "^" => BinaryOp::BitXor,
                "<<" => BinaryOp::Shl,
                ">>" => BinaryOp::Shr,
                ">>>" => BinaryOp::UShr,
                "**" => BinaryOp::Pow,
                _ => BinaryOp::Add, // 默认为加法操作
            };
            let left_expr = expression(left)?;
            let right_expr = expression(right)?;
            Ok(Expression::Binary { left: Box::new(left_expr), op: binary_op, right: Box::new(right_expr) })
        }
        ast::ExpressionKind::UnaryExpression { operator, argument } => {
            let unary_op = match operator.as_str() {
                "!" => UnaryOp::Not,
                "-" => UnaryOp::Neg,
                "+" => UnaryOp::Pos,
                "~" => UnaryOp::BitNot,
                "typeof" => UnaryOp::TypeOf,
                "void" => UnaryOp::Void,
                "delete" => UnaryOp::Delete,
                _ => UnaryOp::Not, // 默认为逻辑非
            };
            let arg_expr = expression(argument)?;
            Ok(Expression::Unary { op: unary_op, expr: Box::new(arg_expr) })
        }
        ast::ExpressionKind::MemberExpression { object, property, computed: _, optional: _ } => {
            let object_expr = expression(object)?;
            let property_expr = expression(property)?;
            Ok(Expression::Member { object: Box::new(object_expr), property: Box::new(property_expr) })
        }
        ast::ExpressionKind::CallExpression { func, args } => {
            let func_expr = expression(func)?;
            let args_exprs = args.iter().map(expression).collect::<Result<Vec<_>, _>>()?;
            Ok(Expression::Call { callee: Box::new(func_expr), args: args_exprs })
        }
        ast::ExpressionKind::AssignmentExpression { left, operator, right } => {
            let assignment_op = match operator.as_str() {
                "=" => AssignmentOp::Assign,
                "+=" => AssignmentOp::AddAssign,
                "-=" => AssignmentOp::SubAssign,
                "*=" => AssignmentOp::MulAssign,
                "/=" => AssignmentOp::DivAssign,
                "%=" => AssignmentOp::ModAssign,
                "&=" => AssignmentOp::BitAndAssign,
                "|=" => AssignmentOp::BitOrAssign,
                "^=" => AssignmentOp::BitXorAssign,
                "<<=" => AssignmentOp::ShlAssign,
                ">>=" => AssignmentOp::ShrAssign,
                ">>>=" => AssignmentOp::UShrAssign,
                "**=" => AssignmentOp::PowAssign,
                _ => AssignmentOp::Assign, // 默认为简单赋值
            };
            let left_expr = expression(left)?;
            let right_expr = expression(right)?;
            Ok(Expression::Assignment { left: Box::new(left_expr), op: assignment_op, right: Box::new(right_expr) })
        }
        ast::ExpressionKind::ConditionalExpression { test, consequent, alternate } => {
            let test_expr = expression(test)?;
            let consequent_expr = expression(consequent)?;
            let alternate_expr = expression(alternate)?;
            Ok(Expression::Conditional {
                test: Box::new(test_expr),
                consequent: Box::new(consequent_expr),
                alternate: Box::new(alternate_expr),
            })
        }
        ast::ExpressionKind::ObjectLiteral { properties } => {
            let mut ir_properties = vec![];
            for prop in properties {
                if let ast::ObjectProperty::Property { name, value, .. } = prop {
                    let value_expr = expression(value)?;
                    ir_properties.push((name.clone(), value_expr));
                }
            }
            Ok(Expression::Object(ir_properties))
        }
        ast::ExpressionKind::ArrayLiteral { elements } => {
            let elements_exprs = elements.iter().map(expression).collect::<Result<Vec<_>, _>>()?;
            Ok(Expression::Array(elements_exprs))
        }
        ast::ExpressionKind::FunctionExpression { name: _, params, body, .. } => {
            let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
            let ir_body: Vec<Statement> = body.iter().map(statement).collect::<Result<Vec<_>, _>>()?;
            Ok(Expression::Function { params: param_names, body: ir_body })
        }
        ast::ExpressionKind::ArrowFunction { params, body, .. } => {
            let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
            // 箭头函数的 body 是 Statement，需要特殊处理
            match &**body {
                ast::Statement::ExpressionStatement(expr_stmt) => {
                    let body_expr = expression(&expr_stmt.expression)?;
                    Ok(Expression::ArrowFunction { params: param_names, body: Box::new(body_expr) })
                }
                _ => {
                    // 对于其他类型的 body，创建一个块语句
                    let ir_body = statement(body)?;
                    Ok(Expression::Function { params: param_names, body: vec![ir_body] })
                }
            }
        }
        _ => Err(TsError::SyntaxError(format!("Unsupported expression type: {:?}", ast_expr.kind))),
    }
}

/// 将 oak-typescript TypeAnnotation 转换为 IR TypeAnnotation
pub fn type_annotation(_ast_type: &ast::TypeAnnotation) -> Result<TypeAnnotation, TsError> {
    // 简化实现，暂时返回 Any 类型
    Ok(TypeAnnotation::Any)
}

/// 将 oak-typescript Statement 转换为 IR Statement
pub fn statement(ast_stmt: &ast::Statement) -> Result<Statement, TsError> {
    match ast_stmt {
        ast::Statement::ExpressionStatement(expr_stmt) => {
            let expr = expression(&expr_stmt.expression)?;
            Ok(Statement::Expression(Box::new(expr)))
        }
        ast::Statement::VariableDeclaration(var_decl) => {
            let ty = var_decl.ty.as_ref().map(type_annotation).transpose()?;
            let initializer = var_decl.value.as_ref().map(|expr| expression(expr)).transpose()?;
            Ok(Statement::VariableDeclaration {
                name: var_decl.name.clone(),
                ty,
                initializer: initializer.map(|expr| Box::new(expr)),
            })
        }
        ast::Statement::BlockStatement(block_stmt) => {
            let statements = block_stmt.statements.iter().map(statement).collect::<Result<Vec<_>, _>>()?;
            Ok(Statement::Block(statements))
        }
        ast::Statement::IfStatement(if_stmt) => {
            let test = expression(&if_stmt.test)?;
            let consequent = statement(&if_stmt.consequent)?;
            let alternate = if_stmt.alternate.as_ref().map(|stmt| statement(stmt)).transpose()?;
            Ok(Statement::If {
                test: Box::new(test),
                consequent: Box::new(consequent),
                alternate: alternate.map(|stmt| Box::new(stmt)),
            })
        }
        ast::Statement::WhileStatement(while_stmt) => {
            let test = expression(&while_stmt.test)?;
            let body = statement(&while_stmt.body)?;
            Ok(Statement::While { test: Box::new(test), body: Box::new(body) })
        }
        ast::Statement::ForStatement(for_stmt) => {
            let init = for_stmt.initializer.as_ref().map(|stmt| statement(stmt)).transpose()?;
            let test = for_stmt.test.as_ref().map(|expr| expression(expr)).transpose()?;
            let update = for_stmt.incrementor.as_ref().map(|expr| expression(expr)).transpose()?;
            let body = statement(&for_stmt.body)?;
            Ok(Statement::For {
                init: init.map(|stmt| Box::new(stmt)),
                test: test.map(|expr| Box::new(expr)),
                update: update.map(|expr| Box::new(expr)),
                body: Box::new(body),
            })
        }
        ast::Statement::ReturnStatement(return_stmt) => {
            let argument = return_stmt.argument.as_ref().map(|expr| expression(expr)).transpose()?;
            Ok(Statement::Return(argument.map(|expr| Box::new(expr))))
        }
        ast::Statement::BreakStatement(_) => Ok(Statement::Break),
        ast::Statement::ContinueStatement(_) => Ok(Statement::Continue),
        ast::Statement::FunctionDeclaration(func_decl) => {
            let param_names: Vec<String> = func_decl.params.iter().map(|p| p.name.clone()).collect();
            let ir_body: Vec<Statement> = func_decl.body.iter().map(statement).collect::<Result<Vec<_>, _>>()?;
            let type_params: Vec<TypeParameter> = func_decl
                .type_params
                .iter()
                .map(|tp| {
                    let constraint = tp.constraint.as_ref().map(type_annotation).transpose()?;
                    let default = tp.default.as_ref().map(type_annotation).transpose()?;
                    Ok(TypeParameter { name: tp.name.clone(), constraint, default })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let return_type = func_decl.return_type.as_ref().map(type_annotation).transpose()?;
            Ok(Statement::FunctionDeclaration {
                name: func_decl.name.clone(),
                params: param_names,
                return_type,
                body: ir_body,
                type_params,
                decorators: Vec::new(), // 暂时不处理装饰器
            })
        }
        ast::Statement::ClassDeclaration(class_decl) => {
            // 简化实现，暂时返回空方法列表
            Ok(Statement::ClassDeclaration {
                name: class_decl.name.clone(),
                super_class: None, // 暂时不支持继承
                methods: Vec::new(),
                decorators: Vec::new(), // 暂时不处理装饰器
            })
        }
        _ => Err(TsError::SyntaxError(format!("Unsupported statement type: {:?}", ast_stmt))),
    }
}

/// 将 oak-typescript TypeScriptRoot 转换为 IR Program
pub fn program(ast_program: &ast::TypeScriptRoot) -> Result<Program, TsError> {
    let statements = ast_program.statements.iter().map(statement).collect::<Result<Vec<_>, _>>()?;
    Ok(Program { statements })
}
