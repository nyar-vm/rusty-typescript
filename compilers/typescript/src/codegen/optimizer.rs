use typescript_types::{TsError, TsValue};

use crate::codegen::instruction::{BinaryOp, Instruction};

/// 优化 VM 指令
pub fn optimize_instructions(instructions: Vec<Instruction>) -> Vec<Instruction> {
    let mut optimized = vec![];
    let mut i = 0;

    while i < instructions.len() {
        if let Some(optimized_instr) = try_constant_fold(&instructions[i..]) {
            optimized.push(optimized_instr);
            i += 3;
            continue;
        }

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

    if let (Some(left_val), Some(right_val), Some(binary_op)) = get_binary_operation_values(&instructions[0..3]) {
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
            _ => return None,
        };

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
    false
}
