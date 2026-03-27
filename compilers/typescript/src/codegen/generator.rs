use std::collections::HashMap;

use typescript_ir::{Expression, Statement};
use typescript_types::TsError;

use crate::codegen::instruction::{BinaryOp, Instruction, UnaryOp};

/// 生成语句的 VM 指令
pub fn generate_statement_instructions(statement: &Statement, instructions: &mut Vec<Instruction>) -> Result<(), TsError> {
    if let Statement::FunctionDeclaration { name, params, body, .. } = statement {
        let mut local_indices = HashMap::new();
        for (i, param) in params.iter().enumerate() {
            local_indices.insert(param.clone(), i);
        }

        instructions.push(Instruction::CreateFunction(name.clone(), params.len() as u32));
        instructions.push(Instruction::StoreVariable(name.clone()));
        let mut function_instructions = vec![];
        for stmt in body {
            generate_statement_instructions(stmt, &mut function_instructions)?;
        }
        function_instructions.push(Instruction::Return);
        instructions.push(Instruction::SetFunctionBody(function_instructions));
        return Ok(());
    }

    match statement {
        Statement::VariableDeclaration { initializer, .. } => {
            if let Some(init) = initializer {
                generate_expression_instructions(init, instructions)?;
            }
            else {
                instructions.push(Instruction::PushUndefined);
            }
            if let Statement::VariableDeclaration { name, .. } = statement {
                instructions.push(Instruction::StoreVariable(name.clone()));
            }
            Ok(())
        }
        Statement::FunctionDeclaration { name, params, body, .. } => {
            instructions.push(Instruction::CreateFunction(name.clone(), params.len() as u32));
            instructions.push(Instruction::StoreVariable(name.clone()));
            let mut function_instructions = vec![];
            for stmt in body {
                generate_statement_instructions(stmt, &mut function_instructions)?;
            }
            function_instructions.push(Instruction::Return);
            instructions.push(Instruction::SetFunctionBody(function_instructions));
            Ok(())
        }
        Statement::Block(statements) => {
            for stmt in statements {
                generate_statement_instructions(stmt, instructions)?;
            }
            Ok(())
        }
        Statement::If { test, consequent, alternate } => {
            generate_expression_instructions(test, instructions)?;
            let else_offset = instructions.len() as u32 + 1;
            instructions.push(Instruction::JumpIfFalse(0));
            generate_statement_instructions(consequent, instructions)?;
            let end_offset = instructions.len() as u32 + 1;
            instructions.push(Instruction::Jump(0));
            if let Some(alt) = alternate {
                generate_statement_instructions(alt, instructions)?;
            }
            let instructions_len = instructions.len() as u32;
            if let Instruction::JumpIfFalse(ref mut offset) = instructions[else_offset as usize - 1] {
                *offset = instructions_len - else_offset;
            }
            if let Instruction::Jump(ref mut offset) = instructions[end_offset as usize - 1] {
                *offset = instructions_len - end_offset;
            }
            Ok(())
        }
        Statement::While { test, body } => {
            let loop_start = instructions.len() as u32;
            generate_expression_instructions(test, instructions)?;
            let loop_end = instructions.len() as u32 + 1;
            instructions.push(Instruction::JumpIfFalse(0));
            generate_statement_instructions(body, instructions)?;
            instructions.push(Instruction::Jump(loop_start - instructions.len() as u32));
            let instructions_len = instructions.len() as u32;
            if let Instruction::JumpIfFalse(ref mut offset) = instructions[loop_end as usize - 1] {
                *offset = instructions_len - loop_end;
            }
            Ok(())
        }
        Statement::For { init, test, update, body } => {
            if let Some(init_stmt) = init {
                generate_statement_instructions(init_stmt, instructions)?;
            }

            let loop_start = instructions.len() as u32;

            if let Some(test_expr) = test {
                generate_expression_instructions(test_expr, instructions)?;
                let loop_end = instructions.len() as u32 + 1;
                instructions.push(Instruction::JumpIfFalse(0));
            }

            generate_statement_instructions(body, instructions)?;

            if let Some(update_expr) = update {
                generate_expression_instructions(update_expr, instructions)?;
                instructions.push(Instruction::Pop);
            }

            instructions.push(Instruction::Jump(loop_start - instructions.len() as u32));

            if test.is_some() {
                let instructions_len = instructions.len() as u32;
                let loop_end = loop_start + (if test.is_some() { 2 } else { 0 });
                if let Instruction::JumpIfFalse(ref mut offset) = instructions[loop_end as usize - 1] {
                    *offset = instructions_len - loop_end;
                }
            }
            Ok(())
        }
        Statement::Return(expr) => {
            if let Some(expr) = expr {
                generate_expression_instructions(expr, instructions)?;
            }
            else {
                instructions.push(Instruction::PushUndefined);
            }
            instructions.push(Instruction::Return);
            Ok(())
        }
        Statement::Break => {
            instructions.push(Instruction::JumpLoop(1));
            Ok(())
        }
        Statement::Continue => {
            instructions.push(Instruction::JumpLoop(0));
            Ok(())
        }
        Statement::ClassDeclaration { name, methods, .. } => {
            instructions.push(Instruction::CreateClass(name.clone()));
            instructions.push(Instruction::StoreVariable(name.clone()));
            let mut class_instructions = vec![];
            for method in methods {
                class_instructions.push(Instruction::AddMethod(method.name.clone()));
                let mut method_instructions = vec![];
                for stmt in &method.body {
                    generate_statement_instructions(stmt, &mut method_instructions)?;
                }
                method_instructions.push(Instruction::Return);
                class_instructions.push(Instruction::SetFunctionBody(method_instructions));
            }
            instructions.push(Instruction::SetClassBody(class_instructions));
            Ok(())
        }
        Statement::InterfaceDeclaration { name, .. } => {
            instructions.push(Instruction::CreateInterface(name.clone()));
            Ok(())
        }
        Statement::TypeAlias { name, .. } => {
            instructions.push(Instruction::CreateTypeAlias(name.clone()));
            Ok(())
        }
        Statement::Expression(expr) => {
            generate_expression_instructions(expr, instructions)?;
            instructions.push(Instruction::Pop);
            Ok(())
        }
        _ => Err(TsError::SyntaxError("Unsupported statement type".to_string())),
    }
}

/// 生成带有局部变量优化的语句指令
pub fn generate_statement_with_locals(
    statement: &Statement,
    instructions: &mut Vec<Instruction>,
    local_indices: &HashMap<String, usize>,
) -> Result<(), TsError> {
    match statement {
        Statement::VariableDeclaration { name, initializer, .. } => {
            if let Some(init) = initializer {
                generate_expression_with_locals(init, instructions, local_indices)?;
            }
            else {
                instructions.push(Instruction::PushUndefined);
            }
            if let Some(index) = local_indices.get(name) {
                instructions.push(Instruction::StoreLocal(*index));
            }
            else {
                instructions.push(Instruction::StoreVariable(name.clone()));
            }
            Ok(())
        }
        Statement::Block(statements) => {
            for stmt in statements {
                generate_statement_with_locals(stmt, instructions, local_indices)?;
            }
            Ok(())
        }
        Statement::If { test, consequent, alternate } => {
            generate_expression_with_locals(test, instructions, local_indices)?;
            let else_offset = instructions.len() as u32 + 1;
            instructions.push(Instruction::JumpIfFalse(0));
            generate_statement_with_locals(consequent, instructions, local_indices)?;
            let end_offset = instructions.len() as u32 + 1;
            instructions.push(Instruction::Jump(0));
            if let Some(alt) = alternate {
                generate_statement_with_locals(alt, instructions, local_indices)?;
            }
            let instructions_len = instructions.len() as u32;
            if let Instruction::JumpIfFalse(ref mut offset) = instructions[else_offset as usize - 1] {
                *offset = instructions_len - else_offset;
            }
            if let Instruction::Jump(ref mut offset) = instructions[end_offset as usize - 1] {
                *offset = instructions_len - end_offset;
            }
            Ok(())
        }
        Statement::While { test, body } => {
            let loop_start = instructions.len() as u32;
            generate_expression_with_locals(test, instructions, local_indices)?;
            let loop_end = instructions.len() as u32 + 1;
            instructions.push(Instruction::JumpIfFalse(0));
            generate_statement_with_locals(body, instructions, local_indices)?;
            instructions.push(Instruction::Jump(loop_start - instructions.len() as u32));
            let instructions_len = instructions.len() as u32;
            if let Instruction::JumpIfFalse(ref mut offset) = instructions[loop_end as usize - 1] {
                *offset = instructions_len - loop_end;
            }
            Ok(())
        }
        Statement::For { init, test, update, body } => {
            if let Some(init_stmt) = init {
                generate_statement_with_locals(init_stmt, instructions, local_indices)?;
            }

            let loop_start = instructions.len() as u32;

            if let Some(test_expr) = test {
                generate_expression_with_locals(test_expr, instructions, local_indices)?;
                let loop_end = instructions.len() as u32 + 1;
                instructions.push(Instruction::JumpIfFalse(0));
            }

            generate_statement_with_locals(body, instructions, local_indices)?;

            if let Some(update_expr) = update {
                generate_expression_with_locals(update_expr, instructions, local_indices)?;
                instructions.push(Instruction::Pop);
            }

            instructions.push(Instruction::Jump(loop_start - instructions.len() as u32));

            if test.is_some() {
                let instructions_len = instructions.len() as u32;
                let loop_end = loop_start + (if test.is_some() { 2 } else { 0 });
                if let Instruction::JumpIfFalse(ref mut offset) = instructions[loop_end as usize - 1] {
                    *offset = instructions_len - loop_end;
                }
            }
            Ok(())
        }
        Statement::Return(expr) => {
            if let Some(expr) = expr {
                generate_expression_with_locals(expr, instructions, local_indices)?;
            }
            else {
                instructions.push(Instruction::PushUndefined);
            }
            instructions.push(Instruction::Return);
            Ok(())
        }
        Statement::Break => {
            instructions.push(Instruction::JumpLoop(1));
            Ok(())
        }
        Statement::Continue => {
            instructions.push(Instruction::JumpLoop(0));
            Ok(())
        }
        Statement::Expression(expr) => {
            generate_expression_with_locals(expr, instructions, local_indices)?;
            instructions.push(Instruction::Pop);
            Ok(())
        }
        _ => generate_statement_instructions(statement, instructions),
    }
}

/// 生成表达式的 VM 指令
pub fn generate_expression_instructions(expr: &Expression, instructions: &mut Vec<Instruction>) -> Result<(), TsError> {
    match expr {
        Expression::Literal(value) => {
            match value {
                typescript_types::TsValue::Undefined => {
                    instructions.push(Instruction::PushUndefined);
                }
                typescript_types::TsValue::Null => {
                    instructions.push(Instruction::PushNull);
                }
                typescript_types::TsValue::Boolean(b) => {
                    instructions.push(Instruction::PushBoolean(*b));
                }
                typescript_types::TsValue::Number(n) => {
                    instructions.push(Instruction::PushNumber(*n));
                }
                typescript_types::TsValue::String(s) => {
                    instructions.push(Instruction::PushString(s.clone()));
                }
                typescript_types::TsValue::Object(_) => {
                    instructions.push(Instruction::CreateObject);
                }
                typescript_types::TsValue::Array(_) => {
                    instructions.push(Instruction::CreateArray);
                }
                _ => {
                    instructions.push(Instruction::PushUndefined);
                }
            }
            Ok(())
        }
        Expression::Identifier(name) => {
            instructions.push(Instruction::LoadVariable(name.clone()));
            Ok(())
        }
        Expression::Binary { left, op, right } => {
            generate_expression_instructions(left, instructions)?;
            generate_expression_instructions(right, instructions)?;
            let binary_op = convert_binary_op(op);
            instructions.push(Instruction::BinaryOp(binary_op));
            Ok(())
        }
        Expression::Unary { op, expr } => {
            generate_expression_instructions(expr, instructions)?;
            let unary_op = convert_unary_op(op);
            instructions.push(Instruction::UnaryOp(unary_op));
            Ok(())
        }
        Expression::Call { callee, args } => {
            for arg in args {
                generate_expression_instructions(arg, instructions)?;
            }
            generate_expression_instructions(callee, instructions)?;
            instructions.push(Instruction::Call(args.len() as u32));
            Ok(())
        }
        Expression::Member { object, property } => {
            generate_expression_instructions(object, instructions)?;
            match property.as_ref() {
                Expression::Identifier(name) => {
                    instructions.push(Instruction::PushString(name.clone()));
                }
                _ => {
                    generate_expression_instructions(property, instructions)?;
                }
            }
            instructions.push(Instruction::GetProperty);
            Ok(())
        }
        Expression::Index { object, index } => {
            generate_expression_instructions(object, instructions)?;
            generate_expression_instructions(index, instructions)?;
            instructions.push(Instruction::GetElement);
            Ok(())
        }
        Expression::Object(properties) => {
            instructions.push(Instruction::CreateObject);
            for (name, value) in properties {
                generate_expression_instructions(value, instructions)?;
                instructions.push(Instruction::PushString(name.clone()));
                instructions.push(Instruction::SetProperty);
            }
            Ok(())
        }
        Expression::Array(elements) => {
            instructions.push(Instruction::CreateArray);
            for (i, element) in elements.iter().enumerate() {
                generate_expression_instructions(element, instructions)?;
                instructions.push(Instruction::PushNumber(i as f64));
                instructions.push(Instruction::SetElement);
            }
            Ok(())
        }
        Expression::Assignment { left, op: _, right } => {
            generate_expression_instructions(right, instructions)?;
            match left.as_ref() {
                Expression::Identifier(name) => {
                    instructions.push(Instruction::StoreVariable(name.clone()));
                }
                Expression::Member { object, property } => {
                    generate_expression_instructions(object, instructions)?;
                    generate_expression_instructions(property, instructions)?;
                    instructions.push(Instruction::SetProperty);
                }
                Expression::Index { object, index } => {
                    generate_expression_instructions(object, instructions)?;
                    generate_expression_instructions(index, instructions)?;
                    instructions.push(Instruction::SetElement);
                }
                _ => {
                    instructions.push(Instruction::Pop);
                }
            }
            Ok(())
        }
        Expression::Conditional { test, consequent, alternate } => {
            generate_expression_instructions(test, instructions)?;
            let else_offset = instructions.len() as u32 + 1;
            instructions.push(Instruction::JumpIfFalse(0));
            generate_expression_instructions(consequent, instructions)?;
            let end_offset = instructions.len() as u32 + 1;
            instructions.push(Instruction::Jump(0));
            generate_expression_instructions(alternate, instructions)?;
            let instructions_len = instructions.len() as u32;
            if let Instruction::JumpIfFalse(ref mut offset) = instructions[else_offset as usize - 1] {
                *offset = instructions_len - else_offset;
            }
            if let Instruction::Jump(ref mut offset) = instructions[end_offset as usize - 1] {
                *offset = instructions_len - end_offset;
            }
            Ok(())
        }
        Expression::Function { params, body } => {
            instructions.push(Instruction::CreateFunction("anonymous".to_string(), params.len() as u32));
            let mut function_instructions = vec![];
            for stmt in body {
                generate_statement_instructions(stmt, &mut function_instructions)?;
            }
            function_instructions.push(Instruction::Return);
            instructions.push(Instruction::SetFunctionBody(function_instructions));
            Ok(())
        }
        Expression::ArrowFunction { params, body } => {
            instructions.push(Instruction::CreateFunction("arrow".to_string(), params.len() as u32));
            let mut function_instructions = vec![];
            generate_expression_instructions(body, &mut function_instructions)?;
            function_instructions.push(Instruction::Return);
            instructions.push(Instruction::SetFunctionBody(function_instructions));
            Ok(())
        }
        _ => Err(TsError::SyntaxError("Unsupported expression type".to_string())),
    }
}

/// 生成带有局部变量优化的表达式指令
pub fn generate_expression_with_locals(
    expr: &Expression,
    instructions: &mut Vec<Instruction>,
    local_indices: &HashMap<String, usize>,
) -> Result<(), TsError> {
    match expr {
        Expression::Identifier(name) => {
            if let Some(index) = local_indices.get(name) {
                instructions.push(Instruction::LoadLocal(*index));
            }
            else {
                instructions.push(Instruction::LoadVariable(name.clone()));
            }
            Ok(())
        }
        Expression::Binary { left, op, right } => {
            generate_expression_with_locals(left, instructions, local_indices)?;
            generate_expression_with_locals(right, instructions, local_indices)?;
            let binary_op = convert_binary_op(op);
            instructions.push(Instruction::BinaryOp(binary_op));
            Ok(())
        }
        Expression::Unary { op, expr } => {
            generate_expression_with_locals(expr, instructions, local_indices)?;
            let unary_op = convert_unary_op(op);
            instructions.push(Instruction::UnaryOp(unary_op));
            Ok(())
        }
        Expression::Call { callee, args } => {
            for arg in args {
                generate_expression_with_locals(arg, instructions, local_indices)?;
            }
            generate_expression_with_locals(callee, instructions, local_indices)?;
            instructions.push(Instruction::Call(args.len() as u32));
            Ok(())
        }
        Expression::Member { object, property } => {
            generate_expression_with_locals(object, instructions, local_indices)?;
            match property.as_ref() {
                Expression::Identifier(name) => {
                    instructions.push(Instruction::PushString(name.clone()));
                }
                _ => {
                    generate_expression_with_locals(property, instructions, local_indices)?;
                }
            }
            instructions.push(Instruction::GetProperty);
            Ok(())
        }
        Expression::Index { object, index } => {
            generate_expression_with_locals(object, instructions, local_indices)?;
            generate_expression_with_locals(index, instructions, local_indices)?;
            instructions.push(Instruction::GetElement);
            Ok(())
        }
        Expression::Object(properties) => {
            instructions.push(Instruction::CreateObject);
            for (name, value) in properties {
                generate_expression_with_locals(value, instructions, local_indices)?;
                instructions.push(Instruction::PushString(name.clone()));
                instructions.push(Instruction::SetProperty);
            }
            Ok(())
        }
        Expression::Array(elements) => {
            instructions.push(Instruction::CreateArray);
            for (i, element) in elements.iter().enumerate() {
                generate_expression_with_locals(element, instructions, local_indices)?;
                instructions.push(Instruction::PushNumber(i as f64));
                instructions.push(Instruction::SetElement);
            }
            Ok(())
        }
        Expression::Assignment { left, op: _, right } => {
            generate_expression_with_locals(right, instructions, local_indices)?;
            match left.as_ref() {
                Expression::Identifier(name) => {
                    if let Some(index) = local_indices.get(name) {
                        instructions.push(Instruction::StoreLocal(*index));
                    }
                    else {
                        instructions.push(Instruction::StoreVariable(name.clone()));
                    }
                }
                Expression::Member { object, property } => {
                    generate_expression_with_locals(object, instructions, local_indices)?;
                    generate_expression_with_locals(property, instructions, local_indices)?;
                    instructions.push(Instruction::SetProperty);
                }
                Expression::Index { object, index } => {
                    generate_expression_with_locals(object, instructions, local_indices)?;
                    generate_expression_with_locals(index, instructions, local_indices)?;
                    instructions.push(Instruction::SetElement);
                }
                _ => {
                    instructions.push(Instruction::Pop);
                }
            }
            Ok(())
        }
        Expression::Conditional { test, consequent, alternate } => {
            generate_expression_with_locals(test, instructions, local_indices)?;
            let else_offset = instructions.len() as u32 + 1;
            instructions.push(Instruction::JumpIfFalse(0));
            generate_expression_with_locals(consequent, instructions, local_indices)?;
            let end_offset = instructions.len() as u32 + 1;
            instructions.push(Instruction::Jump(0));
            generate_expression_with_locals(alternate, instructions, local_indices)?;
            let instructions_len = instructions.len() as u32;
            if let Instruction::JumpIfFalse(ref mut offset) = instructions[else_offset as usize - 1] {
                *offset = instructions_len - else_offset;
            }
            if let Instruction::Jump(ref mut offset) = instructions[end_offset as usize - 1] {
                *offset = instructions_len - end_offset;
            }
            Ok(())
        }
        Expression::Function { params, body } => {
            let mut nested_local_indices = local_indices.clone();
            for (i, param) in params.iter().enumerate() {
                nested_local_indices.insert(param.clone(), i);
            }

            instructions.push(Instruction::CreateFunction("anonymous".to_string(), params.len() as u32));
            let mut function_instructions = vec![];
            for stmt in body {
                generate_statement_with_locals(stmt, &mut function_instructions, &nested_local_indices)?;
            }
            function_instructions.push(Instruction::Return);
            instructions.push(Instruction::SetFunctionBody(function_instructions));
            Ok(())
        }
        Expression::ArrowFunction { params, body } => {
            let mut arrow_local_indices = local_indices.clone();
            for (i, param) in params.iter().enumerate() {
                arrow_local_indices.insert(param.clone(), i);
            }

            instructions.push(Instruction::CreateFunction("arrow".to_string(), params.len() as u32));
            let mut function_instructions = vec![];
            generate_expression_with_locals(body, &mut function_instructions, &arrow_local_indices)?;
            function_instructions.push(Instruction::Return);
            instructions.push(Instruction::SetFunctionBody(function_instructions));
            Ok(())
        }
        _ => generate_expression_instructions(expr, instructions),
    }
}

/// 转换二元操作符
fn convert_binary_op(op: &typescript_ir::BinaryOp) -> BinaryOp {
    match op {
        typescript_ir::BinaryOp::Add => BinaryOp::Add,
        typescript_ir::BinaryOp::Sub => BinaryOp::Sub,
        typescript_ir::BinaryOp::Mul => BinaryOp::Mul,
        typescript_ir::BinaryOp::Div => BinaryOp::Div,
        typescript_ir::BinaryOp::Mod => BinaryOp::Mod,
        typescript_ir::BinaryOp::Eq => BinaryOp::Eq,
        typescript_ir::BinaryOp::Neq => BinaryOp::Neq,
        typescript_ir::BinaryOp::StrictEq => BinaryOp::StrictEq,
        typescript_ir::BinaryOp::StrictNeq => BinaryOp::StrictNeq,
        typescript_ir::BinaryOp::Gt => BinaryOp::Gt,
        typescript_ir::BinaryOp::Gte => BinaryOp::Gte,
        typescript_ir::BinaryOp::Lt => BinaryOp::Lt,
        typescript_ir::BinaryOp::Lte => BinaryOp::Lte,
        typescript_ir::BinaryOp::And => BinaryOp::And,
        typescript_ir::BinaryOp::Or => BinaryOp::Or,
        typescript_ir::BinaryOp::BitAnd => BinaryOp::BitAnd,
        typescript_ir::BinaryOp::BitOr => BinaryOp::BitOr,
        typescript_ir::BinaryOp::BitXor => BinaryOp::BitXor,
        typescript_ir::BinaryOp::Shl => BinaryOp::Shl,
        typescript_ir::BinaryOp::Shr => BinaryOp::Shr,
        typescript_ir::BinaryOp::UShr => BinaryOp::UShr,
        typescript_ir::BinaryOp::Pow => BinaryOp::Pow,
    }
}

/// 转换一元操作符
fn convert_unary_op(op: &typescript_ir::UnaryOp) -> UnaryOp {
    match op {
        typescript_ir::UnaryOp::Not => UnaryOp::Not,
        typescript_ir::UnaryOp::Neg => UnaryOp::Neg,
        typescript_ir::UnaryOp::Pos => UnaryOp::Pos,
        typescript_ir::UnaryOp::BitNot => UnaryOp::BitNot,
        typescript_ir::UnaryOp::Inc => UnaryOp::Inc,
        typescript_ir::UnaryOp::Dec => UnaryOp::Dec,
        typescript_ir::UnaryOp::TypeOf => UnaryOp::TypeOf,
        typescript_ir::UnaryOp::Void => UnaryOp::Void,
        typescript_ir::UnaryOp::Delete => UnaryOp::Delete,
    }
}
