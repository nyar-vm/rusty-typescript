use typescript_ir::{Expression, Program, Statement, TypeAnnotation};

/// 从 AST 生成 IR
pub fn ast_to_ir(program: Program) -> Program {
    // 这里直接返回原始程序，实际实现需要进行 AST 到 IR 的转换
    // 由于我们的 AST 和 IR 结构相似，暂时直接使用 AST 作为 IR
    program
}

/// 从 IR 生成 VM 指令
pub fn ir_to_vm_instructions(program: &Program) -> Vec<Instruction> {
    let mut instructions = vec![];

    // 遍历所有语句，生成对应的 VM 指令
    for statement in &program.statements {
        generate_statement_instructions(statement, &mut instructions);
    }

    // 打印生成的指令
    println!("Generated instructions: {:?}", instructions);

    instructions
}

/// 生成语句的 VM 指令
fn generate_statement_instructions(statement: &Statement, instructions: &mut Vec<Instruction>) {
    match statement {
        Statement::VariableDeclaration { name, ty, initializer } => {
            // 生成初始化表达式的指令
            if let Some(init) = initializer {
                generate_expression_instructions(init, instructions);
            }
            else {
                // 如果没有初始化表达式，压入 undefined
                instructions.push(Instruction::PushUndefined);
            }
            // 存储变量
            instructions.push(Instruction::StoreVariable(name.clone()));
        }
        Statement::FunctionDeclaration { name, params, return_type, body } => {
            // 生成函数对象
            instructions.push(Instruction::CreateFunction(name.clone(), params.len() as u32));
            // 存储函数
            instructions.push(Instruction::StoreVariable(name.clone()));
            // 生成函数体指令
            let mut function_instructions = vec![];
            for stmt in body {
                generate_statement_instructions(stmt, &mut function_instructions);
            }
            // 添加函数返回指令
            function_instructions.push(Instruction::Return);
            // 设置函数体
            instructions.push(Instruction::SetFunctionBody(function_instructions));
        }
        Statement::Block(statements) => {
            // 生成块内所有语句的指令
            for stmt in statements {
                generate_statement_instructions(stmt, instructions);
            }
        }
        Statement::If { test, consequent, alternate } => {
            // 生成条件表达式的指令
            generate_expression_instructions(test, instructions);
            // 生成条件跳转指令
            let else_offset = instructions.len() as u32 + 1;
            instructions.push(Instruction::JumpIfFalse(0)); // 占位符，后续更新
            // 生成 consequent 语句的指令
            generate_statement_instructions(consequent, instructions);
            // 生成跳转到结束的指令
            let end_offset = instructions.len() as u32 + 1;
            instructions.push(Instruction::Jump(0)); // 占位符，后续更新
            // 生成 alternate 语句的指令
            if let Some(alt) = alternate {
                generate_statement_instructions(alt, instructions);
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
        }
        Statement::While { test, body } => {
            let loop_start = instructions.len() as u32;
            // 生成条件表达式的指令
            generate_expression_instructions(test, instructions);
            // 生成条件跳转指令
            let loop_end = instructions.len() as u32 + 1;
            instructions.push(Instruction::JumpIfFalse(0)); // 占位符，后续更新
            // 生成 body 语句的指令
            generate_statement_instructions(body, instructions);
            // 生成跳转到循环开始的指令
            instructions.push(Instruction::Jump(loop_start - instructions.len() as u32));
            // 更新循环结束跳转偏移
            let instructions_len = instructions.len() as u32;
            if let Instruction::JumpIfFalse(ref mut offset) = instructions[loop_end as usize - 1] {
                *offset = instructions_len - loop_end;
            }
        }
        Statement::For { init, test, update, body } => {
            // 生成初始化语句的指令
            if let Some(init_stmt) = init {
                generate_statement_instructions(init_stmt, instructions);
            }

            let loop_start = instructions.len() as u32;

            // 生成条件表达式的指令
            if let Some(test_expr) = test {
                generate_expression_instructions(test_expr, instructions);
                // 生成条件跳转指令
                let loop_end = instructions.len() as u32 + 1;
                instructions.push(Instruction::JumpIfFalse(0)); // 占位符，后续更新
            }

            // 生成 body 语句的指令
            generate_statement_instructions(body, instructions);

            // 生成更新表达式的指令
            if let Some(update_expr) = update {
                generate_expression_instructions(update_expr, instructions);
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
        }
        Statement::Return(expr) => {
            // 生成返回表达式的指令
            if let Some(expr) = expr {
                generate_expression_instructions(expr, instructions);
            }
            else {
                // 如果没有返回表达式，压入 undefined
                instructions.push(Instruction::PushUndefined);
            }
            // 生成返回指令
            instructions.push(Instruction::Return);
        }
        Statement::Break => {
            // 生成跳出循环的指令
            instructions.push(Instruction::JumpLoop(1));
        }
        Statement::Continue => {
            // 生成继续循环的指令
            instructions.push(Instruction::JumpLoop(0));
        }
        Statement::ClassDeclaration { name, super_class, methods } => {
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
                    generate_statement_instructions(stmt, &mut method_instructions);
                }
                // 添加方法返回指令
                method_instructions.push(Instruction::Return);
                // 设置方法体
                class_instructions.push(Instruction::SetFunctionBody(method_instructions));
            }
            // 设置类体
            instructions.push(Instruction::SetClassBody(class_instructions));
        }
        Statement::InterfaceDeclaration { name, extends, members } => {
            // 生成接口
            instructions.push(Instruction::CreateInterface(name.clone()));
        }
        Statement::TypeAlias { name, ty } => {
            // 生成类型别名
            instructions.push(Instruction::CreateTypeAlias(name.clone()));
        }
        Statement::Expression(expr) => {
            // 生成表达式的指令
            generate_expression_instructions(expr, instructions);
            // 弹出表达式结果（如果不需要）
            instructions.push(Instruction::Pop);
        }
    }
}

/// 生成表达式的 VM 指令
fn generate_expression_instructions(expr: &Expression, instructions: &mut Vec<Instruction>) {
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
        }
        Expression::Identifier(name) => {
            // 生成加载变量指令
            instructions.push(Instruction::LoadVariable(name.clone()));
        }
        Expression::Binary { left, op, right } => {
            // 生成左表达式的指令
            generate_expression_instructions(left, instructions);
            // 生成右表达式的指令
            generate_expression_instructions(right, instructions);
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
        }
        Expression::Unary { op, expr } => {
            // 生成表达式的指令
            generate_expression_instructions(expr, instructions);
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
        }
        Expression::Call { callee, args } => {
            // 生成参数的指令
            for arg in args {
                generate_expression_instructions(arg, instructions);
            }
            // 生成被调用者的指令
            generate_expression_instructions(callee, instructions);
            // 生成调用指令
            instructions.push(Instruction::Call(args.len() as u32));
        }
        Expression::Member { object, property } => {
            // 生成对象的指令
            generate_expression_instructions(object, instructions);
            // 生成属性的指令
            match property.as_ref() {
                Expression::Identifier(name) => {
                    // 对于 . 操作符后的标识符，生成 PushString 指令
                    instructions.push(Instruction::PushString(name.clone()));
                }
                _ => {
                    // 对于其他类型的属性表达式，正常处理
                    generate_expression_instructions(property, instructions);
                }
            }
            // 生成成员访问指令
            instructions.push(Instruction::GetProperty);
        }
        Expression::Index { object, index } => {
            // 生成对象的指令
            generate_expression_instructions(object, instructions);
            // 生成索引的指令
            generate_expression_instructions(index, instructions);
            // 生成索引访问指令
            instructions.push(Instruction::GetElement);
        }
        Expression::Object(properties) => {
            // 创建对象
            instructions.push(Instruction::CreateObject);
            // 设置属性
            for (name, value) in properties {
                // 生成属性值的指令
                generate_expression_instructions(value, instructions);
                // 压入属性名
                instructions.push(Instruction::PushString(name.clone()));
                // 设置属性
                instructions.push(Instruction::SetProperty);
            }
        }
        Expression::Array(elements) => {
            // 创建数组
            instructions.push(Instruction::CreateArray);
            // 设置元素
            for (i, element) in elements.iter().enumerate() {
                // 生成元素值的指令
                generate_expression_instructions(element, instructions);
                // 压入索引
                instructions.push(Instruction::PushNumber(i as f64));
                // 设置元素
                instructions.push(Instruction::SetElement);
            }
        }
        Expression::Assignment { left, op, right } => {
            // 生成右表达式的指令
            generate_expression_instructions(right, instructions);
            // 处理不同类型的左表达式
            match left.as_ref() {
                Expression::Identifier(name) => {
                    // 存储变量
                    instructions.push(Instruction::StoreVariable(name.clone()));
                }
                Expression::Member { object, property } => {
                    // 生成对象的指令
                    generate_expression_instructions(object, instructions);
                    // 生成属性的指令
                    generate_expression_instructions(property, instructions);
                    // 设置属性
                    instructions.push(Instruction::SetProperty);
                }
                Expression::Index { object, index } => {
                    // 生成对象的指令
                    generate_expression_instructions(object, instructions);
                    // 生成索引的指令
                    generate_expression_instructions(index, instructions);
                    // 设置元素
                    instructions.push(Instruction::SetElement);
                }
                _ => {
                    // 其他类型的左表达式暂时不处理
                    instructions.push(Instruction::Pop);
                }
            }
        }
        Expression::Conditional { test, consequent, alternate } => {
            // 生成条件表达式的指令
            generate_expression_instructions(test, instructions);
            // 生成条件跳转指令
            let else_offset = instructions.len() as u32 + 1;
            instructions.push(Instruction::JumpIfFalse(0)); // 占位符，后续更新
            // 生成 consequent 表达式的指令
            generate_expression_instructions(consequent, instructions);
            // 生成跳转到结束的指令
            let end_offset = instructions.len() as u32 + 1;
            instructions.push(Instruction::Jump(0)); // 占位符，后续更新
            // 生成 alternate 表达式的指令
            generate_expression_instructions(alternate, instructions);
            // 更新 else 跳转偏移
            let instructions_len = instructions.len() as u32;
            if let Instruction::JumpIfFalse(ref mut offset) = instructions[else_offset as usize - 1] {
                *offset = instructions_len - else_offset;
            }
            // 更新结束跳转偏移
            if let Instruction::Jump(ref mut offset) = instructions[end_offset as usize - 1] {
                *offset = instructions_len - end_offset;
            }
        }
        Expression::Function { params, body } => {
            // 生成函数对象
            instructions.push(Instruction::CreateFunction("anonymous".to_string(), params.len() as u32));
            // 生成函数体指令
            let mut function_instructions = vec![];
            for stmt in body {
                generate_statement_instructions(stmt, &mut function_instructions);
            }
            // 添加函数返回指令
            function_instructions.push(Instruction::Return);
            // 设置函数体
            instructions.push(Instruction::SetFunctionBody(function_instructions));
        }
        Expression::ArrowFunction { params, body } => {
            // 生成函数对象
            instructions.push(Instruction::CreateFunction("arrow".to_string(), params.len() as u32));
            // 生成函数体指令
            let mut function_instructions = vec![];
            generate_expression_instructions(body, &mut function_instructions);
            // 添加函数返回指令
            function_instructions.push(Instruction::Return);
            // 设置函数体
            instructions.push(Instruction::SetFunctionBody(function_instructions));
        }
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
}

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

/// 导入 typescript_types
use typescript_types;
