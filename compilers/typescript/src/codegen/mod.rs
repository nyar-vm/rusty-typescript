use typescript_ir::{Expression, Program, Statement, TypeAnnotation};
use typescript_types::{TsError, TsValue};

/// 二元操作符
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    StrictEq,
    StrictNeq,
    Gt,
    Gte,
    Lt,
    Lte,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    UShr,
    Pow,
}

/// 一元操作符
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Neg,
    Pos,
    BitNot,
    Inc,
    Dec,
    TypeOf,
    Void,
    Delete,
}

/// 从 AST 生成 IR
pub fn ast_to_ir(program: Program) -> Program {
    // 这里直接返回原始程序，实际实现需要进行 AST 到 IR 的转换
    // 由于我们的 AST 和 IR 结构相似，暂时直接使用 AST 作为 IR
    program
}

/// 从 IR 生成 VM 指令
pub fn ir_to_vm_instructions(program: &Program) -> Result<Vec<Instruction>, TsError> {
    let mut instructions = vec![];

    // 遍历所有语句，生成对应的 VM 指令
    for statement in &program.statements {
        generate_statement_instructions(statement, &mut instructions)?;
    }

    // 优化指令
    let optimized_instructions = optimize_instructions(instructions);

    // 打印生成的指令
    println!("Generated instructions: {:?}", optimized_instructions);

    Ok(optimized_instructions)
}

/// 优化 VM 指令
fn optimize_instructions(instructions: Vec<Instruction>) -> Vec<Instruction> {
    let mut optimized = vec![];
    let mut i = 0;

    while i < instructions.len() {
        // 常量折叠优化
        if let Some(optimized_instr) = try_constant_fold(&instructions[i..]) {
            optimized.push(optimized_instr);
            // 跳过被折叠的指令
            i += 3; // 二元操作通常需要3条指令：压入左操作数、压入右操作数、执行操作
            continue;
        }

        // 死代码消除
        if !is_dead_code(&instructions[i]) {
            optimized.push(instructions[i].clone());
        }

        i += 1;
    }

    optimized
}

/// 尝试常量折叠
fn try_constant_fold(instructions: &[Instruction]) -> Option<Instruction> {
    if instructions.len() < 3 {
        return None;
    }

    // 检查是否是二元操作指令序列
    if let (Some(left_val), Some(right_val), Some(binary_op)) = get_binary_operation_values(&instructions[0..3]) {
        // 计算常量结果
        let result = match binary_op {
            BinaryOp::Add => TsValue::Number(left_val.to_number() + right_val.to_number()),
            BinaryOp::Sub => TsValue::Number(left_val.to_number() - right_val.to_number()),
            BinaryOp::Mul => TsValue::Number(left_val.to_number() * right_val.to_number()),
            BinaryOp::Div => TsValue::Number(left_val.to_number() / right_val.to_number()),
            BinaryOp::Mod => TsValue::Number(left_val.to_number() % right_val.to_number()),
            BinaryOp::Eq => TsValue::Boolean(left_val.to_string() == right_val.to_string()),
            BinaryOp::Neq => TsValue::Boolean(left_val.to_string() != right_val.to_string()),
            BinaryOp::StrictEq => TsValue::Boolean(
                left_val.to_string() == right_val.to_string()
                    && left_val.is_number() == right_val.is_number()
                    && left_val.is_string() == right_val.is_string()
                    && left_val.is_boolean() == right_val.is_boolean(),
            ),
            BinaryOp::StrictNeq => TsValue::Boolean(
                left_val.to_string() != right_val.to_string()
                    || left_val.is_number() != right_val.is_number()
                    || left_val.is_string() != right_val.is_string()
                    || left_val.is_boolean() != right_val.is_boolean(),
            ),
            BinaryOp::Gt => TsValue::Boolean(left_val.to_number() > right_val.to_number()),
            BinaryOp::Gte => TsValue::Boolean(left_val.to_number() >= right_val.to_number()),
            BinaryOp::Lt => TsValue::Boolean(left_val.to_number() < right_val.to_number()),
            BinaryOp::Lte => TsValue::Boolean(left_val.to_number() <= right_val.to_number()),
            BinaryOp::And => TsValue::Boolean(left_val.to_boolean() && right_val.to_boolean()),
            BinaryOp::Or => TsValue::Boolean(left_val.to_boolean() || right_val.to_boolean()),
            _ => return None, // 其他操作暂不优化
        };

        // 生成对应的常量指令
        match result {
            TsValue::Undefined => Some(Instruction::PushUndefined),
            TsValue::Null => Some(Instruction::PushNull),
            TsValue::Boolean(b) => Some(Instruction::PushBoolean(b)),
            TsValue::Number(n) => Some(Instruction::PushNumber(n)),
            TsValue::String(s) => Some(Instruction::PushString(s)),
            _ => None,
        }
    }
    else {
        None
    }
}

/// 获取二元操作的操作数和操作符
fn get_binary_operation_values(instructions: &[Instruction]) -> (Option<TsValue>, Option<TsValue>, Option<BinaryOp>) {
    if instructions.len() != 3 {
        return (None, None, None);
    }

    let left_val = get_constant_value(&instructions[0]);
    let right_val = get_constant_value(&instructions[1]);
    let binary_op = get_binary_op(&instructions[2]);

    (left_val, right_val, binary_op)
}

/// 获取常量指令的值
fn get_constant_value(instruction: &Instruction) -> Option<TsValue> {
    match instruction {
        Instruction::PushUndefined => Some(TsValue::Undefined),
        Instruction::PushNull => Some(TsValue::Null),
        Instruction::PushBoolean(b) => Some(TsValue::Boolean(*b)),
        Instruction::PushNumber(n) => Some(TsValue::Number(*n)),
        Instruction::PushString(s) => Some(TsValue::String(s.clone())),
        _ => None,
    }
}

/// 获取二元操作指令的操作符
fn get_binary_op(instruction: &Instruction) -> Option<BinaryOp> {
    match instruction {
        Instruction::BinaryOp(op) => Some(op.clone()),
        _ => None,
    }
}

/// 检查是否是死代码
fn is_dead_code(_instruction: &Instruction) -> bool {
    // 简单的死代码检测：如果在 return 指令之后的指令都是死代码
    // 这里可以扩展更复杂的死代码检测逻辑
    false
}

/// 生成语句的 VM 指令
fn generate_statement_instructions(statement: &Statement, instructions: &mut Vec<Instruction>) -> Result<(), TsError> {
    // 对于函数声明，我们可以优化局部变量的访问
    if let Statement::FunctionDeclaration { name, params, return_type, body, .. } = statement {
        // 为参数分配局部变量索引
        let mut local_indices = std::collections::HashMap::new();
        for (i, param) in params.iter().enumerate() {
            local_indices.insert(param.clone(), i);
        }

        // 生成函数对象
        instructions.push(Instruction::CreateFunction(name.clone(), params.len() as u32));
        // 存储函数
        instructions.push(Instruction::StoreVariable(name.clone()));
        // 生成函数体指令
        let mut function_instructions = vec![];
        for stmt in body {
            generate_statement_instructions(stmt, &mut function_instructions)?;
        }
        // 添加函数返回指令
        function_instructions.push(Instruction::Return);
        // 设置函数体
        instructions.push(Instruction::SetFunctionBody(function_instructions));
        return Ok(());
    }

    // 其他语句类型的处理
    match statement {
        Statement::VariableDeclaration { name, ty, initializer } => {
            // 生成初始化表达式的指令
            if let Some(init) = initializer {
                generate_expression_instructions(init, instructions)?;
            }
            else {
                // 如果没有初始化表达式，压入 undefined
                instructions.push(Instruction::PushUndefined);
            }
            // 存储变量
            instructions.push(Instruction::StoreVariable(name.clone()));
            Ok(())
        }
        Statement::FunctionDeclaration { name, params, return_type, body, .. } => {
            // 生成函数对象
            instructions.push(Instruction::CreateFunction(name.clone(), params.len() as u32));
            // 存储函数
            instructions.push(Instruction::StoreVariable(name.clone()));
            // 生成函数体指令
            let mut function_instructions = vec![];
            for stmt in body {
                generate_statement_instructions(stmt, &mut function_instructions)?;
            }
            // 添加函数返回指令
            function_instructions.push(Instruction::Return);
            // 设置函数体
            instructions.push(Instruction::SetFunctionBody(function_instructions));
            Ok(())
        }
        Statement::Block(statements) => {
            // 生成块内所有语句的指令
            for stmt in statements {
                generate_statement_instructions(stmt, instructions)?;
            }
            Ok(())
        }
        Statement::If { test, consequent, alternate } => {
            // 生成条件表达式的指令
            generate_expression_instructions(test, instructions)?;
            // 生成条件跳转指令
            let else_offset = instructions.len() as u32 + 1;
            instructions.push(Instruction::JumpIfFalse(0)); // 占位符，后续更新
            // 生成 consequent 语句的指令
            generate_statement_instructions(consequent, instructions)?;
            // 生成跳转到结束的指令
            let end_offset = instructions.len() as u32 + 1;
            instructions.push(Instruction::Jump(0)); // 占位符，后续更新
            // 生成 alternate 语句的指令
            if let Some(alt) = alternate {
                generate_statement_instructions(alt, instructions)?;
            }
            // 更新 else 跳转偏移
            let instructions_len = instructions.len() as u32;
            if let Instruction::JumpIfFalse(ref mut offset) = instructions[else_offset as usize - 1] {
                *offset = instructions_len - else_offset;
            }
            // 更新结束跳转偏移
            if let Instruction::Jump(ref mut offset) = instructions[end_offset as usize - 1] {
                *offset = instructions_len - end_offset;
            }
            Ok(())
        }
        Statement::While { test, body } => {
            let loop_start = instructions.len() as u32;
            // 生成条件表达式的指令
            generate_expression_instructions(test, instructions)?;
            // 生成条件跳转指令
            let loop_end = instructions.len() as u32 + 1;
            instructions.push(Instruction::JumpIfFalse(0)); // 占位符，后续更新
            // 生成 body 语句的指令
            generate_statement_instructions(body, instructions)?;
            // 生成跳转到循环开始的指令
            instructions.push(Instruction::Jump(loop_start - instructions.len() as u32));
            // 更新循环结束跳转偏移
            let instructions_len = instructions.len() as u32;
            if let Instruction::JumpIfFalse(ref mut offset) = instructions[loop_end as usize - 1] {
                *offset = instructions_len - loop_end;
            }
            Ok(())
        }
        Statement::For { init, test, update, body } => {
            // 生成初始化语句的指令
            if let Some(init_stmt) = init {
                generate_statement_instructions(init_stmt, instructions)?;
            }

            let loop_start = instructions.len() as u32;

            // 生成条件表达式的指令
            if let Some(test_expr) = test {
                generate_expression_instructions(test_expr, instructions)?;
                // 生成条件跳转指令
                let loop_end = instructions.len() as u32 + 1;
                instructions.push(Instruction::JumpIfFalse(0)); // 占位符，后续更新
            }

            // 生成 body 语句的指令
            generate_statement_instructions(body, instructions)?;

            // 生成更新表达式的指令
            if let Some(update_expr) = update {
                generate_expression_instructions(update_expr, instructions)?;
                // 弹出更新表达式的结果
                instructions.push(Instruction::Pop);
            }

            // 生成跳转到循环开始的指令
            instructions.push(Instruction::Jump(loop_start - instructions.len() as u32));

            // 更新循环结束跳转偏移
            if test.is_some() {
                let instructions_len = instructions.len() as u32;
                let loop_end = loop_start + (if let Some(_) = test { 2 } else { 0 });
                if let Instruction::JumpIfFalse(ref mut offset) = instructions[loop_end as usize - 1] {
                    *offset = instructions_len - loop_end;
                }
            }
            Ok(())
        }
        Statement::Return(expr) => {
            // 生成返回表达式的指令
            if let Some(expr) = expr {
                generate_expression_instructions(expr, instructions)?;
            }
            else {
                // 如果没有返回表达式，压入 undefined
                instructions.push(Instruction::PushUndefined);
            }
            // 生成返回指令
            instructions.push(Instruction::Return);
            Ok(())
        }
        Statement::Break => {
            // 生成跳出循环的指令
            instructions.push(Instruction::JumpLoop(1));
            Ok(())
        }
        Statement::Continue => {
            // 生成继续循环的指令
            instructions.push(Instruction::JumpLoop(0));
            Ok(())
        }
        Statement::ClassDeclaration { name, super_class, methods, .. } => {
            // 生成类对象
            instructions.push(Instruction::CreateClass(name.clone()));
            // 存储类
            instructions.push(Instruction::StoreVariable(name.clone()));
            // 生成类体指令
            let mut class_instructions = vec![];
            for method in methods {
                // 生成方法
                class_instructions.push(Instruction::AddMethod(method.name.clone()));
                // 生成方法体指令
                let mut method_instructions = vec![];
                for stmt in &method.body {
                    generate_statement_instructions(stmt, &mut method_instructions)?;
                }
                // 添加方法返回指令
                method_instructions.push(Instruction::Return);
                // 设置方法体
                class_instructions.push(Instruction::SetFunctionBody(method_instructions));
            }
            // 设置类体
            instructions.push(Instruction::SetClassBody(class_instructions));
            Ok(())
        }
        Statement::InterfaceDeclaration { name, extends, members, type_params } => {
            // 生成接口
            instructions.push(Instruction::CreateInterface(name.clone()));
            Ok(())
        }
        Statement::TypeAlias { name, ty, type_params } => {
            // 生成类型别名
            instructions.push(Instruction::CreateTypeAlias(name.clone()));
            Ok(())
        }
        Statement::Expression(expr) => {
            // 生成表达式的指令
            generate_expression_instructions(expr, instructions)?;
            // 弹出表达式结果（如果不需要）
            instructions.push(Instruction::Pop);
            Ok(())
        }
        _ => Err(TsError::SyntaxError("Unsupported statement type".to_string())),
    }
}

/// 生成带有局部变量优化的语句指令
fn generate_statement_with_locals(
    statement: &Statement,
    instructions: &mut Vec<Instruction>,
    local_indices: &std::collections::HashMap<String, usize>,
) -> Result<(), TsError> {
    match statement {
        Statement::VariableDeclaration { name, ty, initializer } => {
            // 生成初始化表达式的指令
            if let Some(init) = initializer {
                generate_expression_with_locals(init, instructions, local_indices)?;
            }
            else {
                // 如果没有初始化表达式，压入 undefined
                instructions.push(Instruction::PushUndefined);
            }
            // 存储变量
            if let Some(index) = local_indices.get(name) {
                instructions.push(Instruction::StoreLocal(*index));
            }
            else {
                instructions.push(Instruction::StoreVariable(name.clone()));
            }
            Ok(())
        }
        Statement::Block(statements) => {
            // 生成块内所有语句的指令
            for stmt in statements {
                generate_statement_with_locals(stmt, instructions, local_indices)?;
            }
            Ok(())
        }
        Statement::If { test, consequent, alternate } => {
            // 生成条件表达式的指令
            generate_expression_with_locals(test, instructions, local_indices)?;
            // 生成条件跳转指令
            let else_offset = instructions.len() as u32 + 1;
            instructions.push(Instruction::JumpIfFalse(0)); // 占位符，后续更新
            // 生成 consequent 语句的指令
            generate_statement_with_locals(consequent, instructions, local_indices)?;
            // 生成跳转到结束的指令
            let end_offset = instructions.len() as u32 + 1;
            instructions.push(Instruction::Jump(0)); // 占位符，后续更新
            // 生成 alternate 语句的指令
            if let Some(alt) = alternate {
                generate_statement_with_locals(alt, instructions, local_indices)?;
            }
            // 更新 else 跳转偏移
            let instructions_len = instructions.len() as u32;
            if let Instruction::JumpIfFalse(ref mut offset) = instructions[else_offset as usize - 1] {
                *offset = instructions_len - else_offset;
            }
            // 更新结束跳转偏移
            if let Instruction::Jump(ref mut offset) = instructions[end_offset as usize - 1] {
                *offset = instructions_len - end_offset;
            }
            Ok(())
        }
        Statement::While { test, body } => {
            let loop_start = instructions.len() as u32;
            // 生成条件表达式的指令
            generate_expression_with_locals(test, instructions, local_indices)?;
            // 生成条件跳转指令
            let loop_end = instructions.len() as u32 + 1;
            instructions.push(Instruction::JumpIfFalse(0)); // 占位符，后续更新
            // 生成 body 语句的指令
            generate_statement_with_locals(body, instructions, local_indices)?;
            // 生成跳转到循环开始的指令
            instructions.push(Instruction::Jump(loop_start - instructions.len() as u32));
            // 更新循环结束跳转偏移
            let instructions_len = instructions.len() as u32;
            if let Instruction::JumpIfFalse(ref mut offset) = instructions[loop_end as usize - 1] {
                *offset = instructions_len - loop_end;
            }
            Ok(())
        }
        Statement::For { init, test, update, body } => {
            // 生成初始化语句的指令
            if let Some(init_stmt) = init {
                generate_statement_with_locals(init_stmt, instructions, local_indices)?;
            }

            let loop_start = instructions.len() as u32;

            // 生成条件表达式的指令
            if let Some(test_expr) = test {
                generate_expression_with_locals(test_expr, instructions, local_indices)?;
                // 生成条件跳转指令
                let loop_end = instructions.len() as u32 + 1;
                instructions.push(Instruction::JumpIfFalse(0)); // 占位符，后续更新
            }

            // 生成 body 语句的指令
            generate_statement_with_locals(body, instructions, local_indices)?;

            // 生成更新表达式的指令
            if let Some(update_expr) = update {
                generate_expression_with_locals(update_expr, instructions, local_indices)?;
                // 弹出更新表达式的结果
                instructions.push(Instruction::Pop);
            }

            // 生成跳转到循环开始的指令
            instructions.push(Instruction::Jump(loop_start - instructions.len() as u32));

            // 更新循环结束跳转偏移
            if test.is_some() {
                let instructions_len = instructions.len() as u32;
                let loop_end = loop_start + (if let Some(_) = test { 2 } else { 0 });
                if let Instruction::JumpIfFalse(ref mut offset) = instructions[loop_end as usize - 1] {
                    *offset = instructions_len - loop_end;
                }
            }
            Ok(())
        }
        Statement::Return(expr) => {
            // 生成返回表达式的指令
            if let Some(expr) = expr {
                generate_expression_with_locals(expr, instructions, local_indices)?;
            }
            else {
                // 如果没有返回表达式，压入 undefined
                instructions.push(Instruction::PushUndefined);
            }
            // 生成返回指令
            instructions.push(Instruction::Return);
            Ok(())
        }
        Statement::Break => {
            // 生成跳出循环的指令
            instructions.push(Instruction::JumpLoop(1));
            Ok(())
        }
        Statement::Continue => {
            // 生成继续循环的指令
            instructions.push(Instruction::JumpLoop(0));
            Ok(())
        }
        Statement::Expression(expr) => {
            // 生成表达式的指令
            generate_expression_with_locals(expr, instructions, local_indices)?;
            // 弹出表达式结果（如果不需要）
            instructions.push(Instruction::Pop);
            Ok(())
        }
        _ => {
            // 其他语句类型使用默认处理
            generate_statement_instructions(statement, instructions)
        }
    }
}

/// 生成带有局部变量优化的表达式指令
fn generate_expression_with_locals(
    expr: &Expression,
    instructions: &mut Vec<Instruction>,
    local_indices: &std::collections::HashMap<String, usize>,
) -> Result<(), TsError> {
    match expr {
        Expression::Identifier(name) => {
            // 生成加载变量指令
            if let Some(index) = local_indices.get(name) {
                instructions.push(Instruction::LoadLocal(*index));
            }
            else {
                instructions.push(Instruction::LoadVariable(name.clone()));
            }
            Ok(())
        }
        Expression::Binary { left, op, right } => {
            // 生成左表达式的指令
            generate_expression_with_locals(left, instructions, local_indices)?;
            // 生成右表达式的指令
            generate_expression_with_locals(right, instructions, local_indices)?;
            // 生成二元操作指令
            let binary_op = match op {
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
            };
            instructions.push(Instruction::BinaryOp(binary_op));
            Ok(())
        }
        Expression::Unary { op, expr } => {
            // 生成表达式的指令
            generate_expression_with_locals(expr, instructions, local_indices)?;
            // 生成一元操作指令
            let unary_op = match op {
                typescript_ir::UnaryOp::Not => UnaryOp::Not,
                typescript_ir::UnaryOp::Neg => UnaryOp::Neg,
                typescript_ir::UnaryOp::Pos => UnaryOp::Pos,
                typescript_ir::UnaryOp::BitNot => UnaryOp::BitNot,
                typescript_ir::UnaryOp::Inc => UnaryOp::Inc,
                typescript_ir::UnaryOp::Dec => UnaryOp::Dec,
                typescript_ir::UnaryOp::TypeOf => UnaryOp::TypeOf,
                typescript_ir::UnaryOp::Void => UnaryOp::Void,
                typescript_ir::UnaryOp::Delete => UnaryOp::Delete,
            };
            instructions.push(Instruction::UnaryOp(unary_op));
            Ok(())
        }
        Expression::Call { callee, args } => {
            // 生成参数的指令
            for arg in args {
                generate_expression_with_locals(arg, instructions, local_indices)?;
            }
            // 生成被调用者的指令
            generate_expression_with_locals(callee, instructions, local_indices)?;
            // 生成调用指令
            instructions.push(Instruction::Call(args.len() as u32));
            Ok(())
        }
        Expression::Member { object, property } => {
            // 生成对象的指令
            generate_expression_with_locals(object, instructions, local_indices)?;
            // 生成属性的指令
            match property.as_ref() {
                Expression::Identifier(name) => {
                    // 对于 . 操作符后的标识符，生成 PushString 指令
                    instructions.push(Instruction::PushString(name.clone()));
                }
                _ => {
                    // 对于其他类型的属性表达式，正常处理
                    generate_expression_with_locals(property, instructions, local_indices)?;
                }
            }
            // 生成成员访问指令
            instructions.push(Instruction::GetProperty);
            Ok(())
        }
        Expression::Index { object, index } => {
            // 生成对象的指令
            generate_expression_with_locals(object, instructions, local_indices)?;
            // 生成索引的指令
            generate_expression_with_locals(index, instructions, local_indices)?;
            // 生成索引访问指令
            instructions.push(Instruction::GetElement);
            Ok(())
        }
        Expression::Object(properties) => {
            // 创建对象
            instructions.push(Instruction::CreateObject);
            // 设置属性
            for (name, value) in properties {
                // 生成属性值的指令
                generate_expression_with_locals(value, instructions, local_indices)?;
                // 压入属性名
                instructions.push(Instruction::PushString(name.clone()));
                // 设置属性
                instructions.push(Instruction::SetProperty);
            }
            Ok(())
        }
        Expression::Array(elements) => {
            // 创建数组
            instructions.push(Instruction::CreateArray);
            // 设置元素
            for (i, element) in elements.iter().enumerate() {
                // 生成元素值的指令
                generate_expression_with_locals(element, instructions, local_indices)?;
                // 压入索引
                instructions.push(Instruction::PushNumber(i as f64));
                // 设置元素
                instructions.push(Instruction::SetElement);
            }
            Ok(())
        }
        Expression::Assignment { left, op: _, right } => {
            // 生成右表达式的指令
            generate_expression_with_locals(right, instructions, local_indices)?;
            // 处理不同类型的左表达式
            match left.as_ref() {
                Expression::Identifier(name) => {
                    // 存储变量
                    if let Some(index) = local_indices.get(name) {
                        instructions.push(Instruction::StoreLocal(*index));
                    }
                    else {
                        instructions.push(Instruction::StoreVariable(name.clone()));
                    }
                }
                Expression::Member { object, property } => {
                    // 生成对象的指令
                    generate_expression_with_locals(object, instructions, local_indices)?;
                    // 生成属性的指令
                    generate_expression_with_locals(property, instructions, local_indices)?;
                    // 设置属性
                    instructions.push(Instruction::SetProperty);
                }
                Expression::Index { object, index } => {
                    // 生成对象的指令
                    generate_expression_with_locals(object, instructions, local_indices)?;
                    // 生成索引的指令
                    generate_expression_with_locals(index, instructions, local_indices)?;
                    // 设置元素
                    instructions.push(Instruction::SetElement);
                }
                _ => {
                    // 其他类型的左表达式暂时不处理
                    instructions.push(Instruction::Pop);
                }
            }
            Ok(())
        }
        Expression::Conditional { test, consequent, alternate } => {
            // 生成条件表达式的指令
            generate_expression_with_locals(test, instructions, local_indices)?;
            // 生成条件跳转指令
            let else_offset = instructions.len() as u32 + 1;
            instructions.push(Instruction::JumpIfFalse(0)); // 占位符，后续更新
            // 生成 consequent 表达式的指令
            generate_expression_with_locals(consequent, instructions, local_indices)?;
            // 生成跳转到结束的指令
            let end_offset = instructions.len() as u32 + 1;
            instructions.push(Instruction::Jump(0)); // 占位符，后续更新
            // 生成 alternate 表达式的指令
            generate_expression_with_locals(alternate, instructions, local_indices)?;
            // 更新 else 跳转偏移
            let instructions_len = instructions.len() as u32;
            if let Instruction::JumpIfFalse(ref mut offset) = instructions[else_offset as usize - 1] {
                *offset = instructions_len - else_offset;
            }
            // 更新结束跳转偏移
            if let Instruction::Jump(ref mut offset) = instructions[end_offset as usize - 1] {
                *offset = instructions_len - end_offset;
            }
            Ok(())
        }
        Expression::Function { params, body } => {
            // 为嵌套函数的参数分配局部变量索引
            let mut nested_local_indices = local_indices.clone();
            for (i, param) in params.iter().enumerate() {
                nested_local_indices.insert(param.clone(), i);
            }

            // 生成函数对象
            instructions.push(Instruction::CreateFunction("anonymous".to_string(), params.len() as u32));
            // 生成函数体指令
            let mut function_instructions = vec![];
            for stmt in body {
                generate_statement_with_locals(stmt, &mut function_instructions, &nested_local_indices)?;
            }
            // 添加函数返回指令
            function_instructions.push(Instruction::Return);
            // 设置函数体
            instructions.push(Instruction::SetFunctionBody(function_instructions));
            Ok(())
        }
        Expression::ArrowFunction { params, body } => {
            // 为箭头函数的参数分配局部变量索引
            let mut arrow_local_indices = local_indices.clone();
            for (i, param) in params.iter().enumerate() {
                arrow_local_indices.insert(param.clone(), i);
            }

            // 生成函数对象
            instructions.push(Instruction::CreateFunction("arrow".to_string(), params.len() as u32));
            // 生成函数体指令
            let mut function_instructions = vec![];
            generate_expression_with_locals(body, &mut function_instructions, &arrow_local_indices)?;
            // 添加函数返回指令
            function_instructions.push(Instruction::Return);
            // 设置函数体
            instructions.push(Instruction::SetFunctionBody(function_instructions));
            Ok(())
        }
        _ => {
            // 其他表达式类型使用默认处理
            generate_expression_instructions(expr, instructions)
        }
    }
}

/// 生成表达式的 VM 指令
fn generate_expression_instructions(expr: &Expression, instructions: &mut Vec<Instruction>) -> Result<(), TsError> {
    match expr {
        Expression::Literal(value) => {
            // 生成字面量指令
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
                    // 其他类型暂时不处理
                    instructions.push(Instruction::PushUndefined);
                }
            }
            Ok(())
        }
        Expression::Identifier(name) => {
            // 生成加载变量指令
            instructions.push(Instruction::LoadVariable(name.clone()));
            Ok(())
        }
        Expression::Binary { left, op, right } => {
            // 生成左表达式的指令
            generate_expression_instructions(left, instructions)?;
            // 生成右表达式的指令
            generate_expression_instructions(right, instructions)?;
            // 生成二元操作指令
            let binary_op = match op {
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
            };
            instructions.push(Instruction::BinaryOp(binary_op));
            Ok(())
        }
        Expression::Unary { op, expr } => {
            // 生成表达式的指令
            generate_expression_instructions(expr, instructions)?;
            // 生成一元操作指令
            let unary_op = match op {
                typescript_ir::UnaryOp::Not => UnaryOp::Not,
                typescript_ir::UnaryOp::Neg => UnaryOp::Neg,
                typescript_ir::UnaryOp::Pos => UnaryOp::Pos,
                typescript_ir::UnaryOp::BitNot => UnaryOp::BitNot,
                typescript_ir::UnaryOp::Inc => UnaryOp::Inc,
                typescript_ir::UnaryOp::Dec => UnaryOp::Dec,
                typescript_ir::UnaryOp::TypeOf => UnaryOp::TypeOf,
                typescript_ir::UnaryOp::Void => UnaryOp::Void,
                typescript_ir::UnaryOp::Delete => UnaryOp::Delete,
            };
            instructions.push(Instruction::UnaryOp(unary_op));
            Ok(())
        }
        Expression::Call { callee, args } => {
            // 生成参数的指令
            for arg in args {
                generate_expression_instructions(arg, instructions)?;
            }
            // 生成被调用者的指令
            generate_expression_instructions(callee, instructions)?;
            // 生成调用指令
            instructions.push(Instruction::Call(args.len() as u32));
            Ok(())
        }
        Expression::Member { object, property } => {
            // 生成对象的指令
            generate_expression_instructions(object, instructions)?;
            // 生成属性的指令
            match property.as_ref() {
                Expression::Identifier(name) => {
                    // 对于 . 操作符后的标识符，生成 PushString 指令
                    instructions.push(Instruction::PushString(name.clone()));
                }
                _ => {
                    // 对于其他类型的属性表达式，正常处理
                    generate_expression_instructions(property, instructions)?;
                }
            }
            // 生成成员访问指令
            instructions.push(Instruction::GetProperty);
            Ok(())
        }
        Expression::Index { object, index } => {
            // 生成对象的指令
            generate_expression_instructions(object, instructions)?;
            // 生成索引的指令
            generate_expression_instructions(index, instructions)?;
            // 生成索引访问指令
            instructions.push(Instruction::GetElement);
            Ok(())
        }
        Expression::Object(properties) => {
            // 创建对象
            instructions.push(Instruction::CreateObject);
            // 设置属性
            for (name, value) in properties {
                // 生成属性值的指令
                generate_expression_instructions(value, instructions)?;
                // 压入属性名
                instructions.push(Instruction::PushString(name.clone()));
                // 设置属性
                instructions.push(Instruction::SetProperty);
            }
            Ok(())
        }
        Expression::Array(elements) => {
            // 创建数组
            instructions.push(Instruction::CreateArray);
            // 设置元素
            for (i, element) in elements.iter().enumerate() {
                // 生成元素值的指令
                generate_expression_instructions(element, instructions)?;
                // 压入索引
                instructions.push(Instruction::PushNumber(i as f64));
                // 设置元素
                instructions.push(Instruction::SetElement);
            }
            Ok(())
        }
        Expression::Assignment { left, op: _, right } => {
            // 生成右表达式的指令
            generate_expression_instructions(right, instructions)?;
            // 处理不同类型的左表达式
            match left.as_ref() {
                Expression::Identifier(name) => {
                    // 存储变量
                    instructions.push(Instruction::StoreVariable(name.clone()));
                }
                Expression::Member { object, property } => {
                    // 生成对象的指令
                    generate_expression_instructions(object, instructions)?;
                    // 生成属性的指令
                    generate_expression_instructions(property, instructions)?;
                    // 设置属性
                    instructions.push(Instruction::SetProperty);
                }
                Expression::Index { object, index } => {
                    // 生成对象的指令
                    generate_expression_instructions(object, instructions)?;
                    // 生成索引的指令
                    generate_expression_instructions(index, instructions)?;
                    // 设置元素
                    instructions.push(Instruction::SetElement);
                }
                _ => {
                    // 其他类型的左表达式暂时不处理
                    instructions.push(Instruction::Pop);
                }
            }
            Ok(())
        }
        Expression::Conditional { test, consequent, alternate } => {
            // 生成条件表达式的指令
            generate_expression_instructions(test, instructions)?;
            // 生成条件跳转指令
            let else_offset = instructions.len() as u32 + 1;
            instructions.push(Instruction::JumpIfFalse(0)); // 占位符，后续更新
            // 生成 consequent 表达式的指令
            generate_expression_instructions(consequent, instructions)?;
            // 生成跳转到结束的指令
            let end_offset = instructions.len() as u32 + 1;
            instructions.push(Instruction::Jump(0)); // 占位符，后续更新
            // 生成 alternate 表达式的指令
            generate_expression_instructions(alternate, instructions)?;
            // 更新 else 跳转偏移
            let instructions_len = instructions.len() as u32;
            if let Instruction::JumpIfFalse(ref mut offset) = instructions[else_offset as usize - 1] {
                *offset = instructions_len - else_offset;
            }
            // 更新结束跳转偏移
            if let Instruction::Jump(ref mut offset) = instructions[end_offset as usize - 1] {
                *offset = instructions_len - end_offset;
            }
            Ok(())
        }
        Expression::Function { params, body } => {
            // 生成函数对象
            instructions.push(Instruction::CreateFunction("anonymous".to_string(), params.len() as u32));
            // 生成函数体指令
            let mut function_instructions = vec![];
            for stmt in body {
                generate_statement_instructions(stmt, &mut function_instructions)?;
            }
            // 添加函数返回指令
            function_instructions.push(Instruction::Return);
            // 设置函数体
            instructions.push(Instruction::SetFunctionBody(function_instructions));
            Ok(())
        }
        Expression::ArrowFunction { params, body } => {
            // 生成函数对象
            instructions.push(Instruction::CreateFunction("arrow".to_string(), params.len() as u32));
            // 生成函数体指令
            let mut function_instructions = vec![];
            generate_expression_instructions(body, &mut function_instructions)?;
            // 添加函数返回指令
            function_instructions.push(Instruction::Return);
            // 设置函数体
            instructions.push(Instruction::SetFunctionBody(function_instructions));
            Ok(())
        }
        _ => Err(TsError::SyntaxError("Unsupported expression type".to_string())),
    }
}

/// VM 指令
#[derive(Debug, Clone)]
pub enum Instruction {
    // 常量操作
    PushUndefined,
    PushNull,
    PushBoolean(bool),
    PushNumber(f64),
    PushString(String),

    // 变量操作
    LoadVariable(String),
    StoreVariable(String),

    // 对象操作
    CreateObject,
    GetProperty,
    SetProperty,

    // 数组操作
    CreateArray,
    GetElement,
    SetElement,

    // 函数操作
    CreateFunction(String, u32),
    SetFunctionBody(Vec<Instruction>),
    Call(u32),
    Return,

    // 类操作
    CreateClass(String),
    AddMethod(String),
    SetClassBody(Vec<Instruction>),

    // 类型操作
    CreateTypeAlias(String),
    CreateInterface(String),

    // 二元操作
    BinaryOp(BinaryOp),

    // 一元操作
    UnaryOp(UnaryOp),

    // 控制流
    Jump(u32),
    JumpIfFalse(u32),
    JumpLoop(u32), // 用于 break 和 continue

    // 栈操作
    Pop,
    Dup,
    Swap,

    // 局部变量操作
    LoadLocal(usize),
    StoreLocal(usize),

    // 异常处理
    TryStart { handler_ip: usize, finally_ip: Option<usize>, exception_var: Option<String> },
    TryEnd,
    Throw,

    // 模块操作
    ImportModule { name: String, alias: Option<String> },
    Export { name: String },
}

/// 导入 typescript_types
use typescript_types;
