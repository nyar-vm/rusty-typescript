//! 常量传播优化模块
//!
//! 该模块实现了常量传播和常量折叠优化，能够在编译期计算常量表达式的值，
//! 并将变量引用替换为已知的常量值。

use crate::{ControlFlowGraph, Expression, Statement};
use std::collections::HashMap;
use typescript_types::TsValue;

/// 常量传播优化器
///
/// 该优化器通过分析程序中的常量定义，将常量引用替换为其具体值，
/// 并在编译期计算可求值的表达式。
pub struct ConstantPropagation;

/// 常量值信息
///
/// 存储变量的常量值信息，用于常量传播分析
#[derive(Debug, Clone)]
pub struct ConstantInfo {
    /// 常量值
    pub value: TsValue,
    /// 是否为不可变常量
    pub is_immutable: bool,
}

/// 常量传播上下文
///
/// 维护常量传播过程中的状态信息
#[derive(Debug, Clone)]
pub struct ConstantPropagationContext {
    /// 变量到常量值的映射
    pub constants: HashMap<String, ConstantInfo>,
    /// 已知纯函数列表
    pub pure_functions: Vec<String>,
}

impl Default for ConstantPropagationContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ConstantPropagationContext {
    /// 创建新的常量传播上下文
    pub fn new() -> Self {
        Self {
            constants: HashMap::new(),
            pure_functions: vec![
                "Math.abs".to_string(),
                "Math.floor".to_string(),
                "Math.ceil".to_string(),
                "Math.round".to_string(),
                "Math.sqrt".to_string(),
                "Math.pow".to_string(),
                "Math.min".to_string(),
                "Math.max".to_string(),
                "parseInt".to_string(),
                "parseFloat".to_string(),
                "isNaN".to_string(),
                "isFinite".to_string(),
            ],
        }
    }

    /// 插入常量值
    pub fn insert(&mut self, name: String, value: TsValue, is_immutable: bool) {
        self.constants.insert(name, ConstantInfo { value, is_immutable });
    }

    /// 获取常量值
    pub fn get(&self, name: &str) -> Option<&ConstantInfo> {
        self.constants.get(name)
    }

    /// 移除变量（当变量被重新赋值时）
    pub fn remove(&mut self, name: &str) {
        self.constants.remove(name);
    }

    /// 检查函数是否为纯函数
    pub fn is_pure_function(&self, name: &str) -> bool {
        self.pure_functions.contains(&name.to_string())
    }
}

impl ConstantPropagation {
    /// 执行常量传播优化
    ///
    /// 分析语句列表，识别常量定义，并将常量引用替换为其值
    pub fn propagate(statements: &mut Vec<Statement>, _cfg: &ControlFlowGraph) {
        let mut ctx = ConstantPropagationContext::new();
        let mut i = 0;

        while i < statements.len() {
            match &statements[i] {
                Statement::VariableDeclaration { name, initializer, .. } => {
                    if let Some(init) = initializer {
                        if let Some(constant_value) = init.eval() {
                            ctx.insert(name.clone(), constant_value, true);
                        }
                    }
                }
                Statement::Expression(expr) => {
                    let optimized_expr = Self::propagate_in_expr(expr, &ctx);
                    statements[i] = Statement::Expression(Box::new(optimized_expr));
                }
                Statement::If { test, consequent, alternate } => {
                    let optimized_test = Self::propagate_in_expr(test, &ctx);
                    if let Some(test_value) = optimized_test.eval() {
                        let condition = test_value.to_boolean();
                        if condition {
                            let mut consequent_stmt = (**consequent).clone();
                            Self::propagate_statement(&mut consequent_stmt, &ctx);
                            statements[i] = consequent_stmt;
                        }
                        else if let Some(alt) = alternate {
                            let mut alt_stmt = (**alt).clone();
                            Self::propagate_statement(&mut alt_stmt, &ctx);
                            statements[i] = alt_stmt;
                        }
                        else {
                            statements.remove(i);
                            continue;
                        }
                    }
                    else {
                        let mut consequent_stmt = (**consequent).clone();
                        Self::propagate_statement(&mut consequent_stmt, &ctx);
                        let mut alternate_stmt = alternate.as_ref().map(|alt| {
                            let mut s = (**alt).clone();
                            Self::propagate_statement(&mut s, &ctx);
                            Box::new(s)
                        });
                        statements[i] = Statement::If {
                            test: Box::new(optimized_test),
                            consequent: Box::new(consequent_stmt),
                            alternate: alternate_stmt,
                        };
                    }
                }
                Statement::While { test, body } => {
                    let optimized_test = Self::propagate_in_expr(test, &ctx);
                    let mut body_stmt = (**body).clone();
                    Self::propagate_statement(&mut body_stmt, &ctx);
                    statements[i] = Statement::While { test: Box::new(optimized_test), body: Box::new(body_stmt) };
                }
                Statement::For { init, test, update, body } => {
                    let optimized_init = init.as_ref().map(|init_stmt| {
                        let mut s = (**init_stmt).clone();
                        Self::propagate_statement(&mut s, &ctx);
                        Box::new(s)
                    });
                    let optimized_test = test.as_ref().map(|t| Self::propagate_in_expr(t, &ctx));
                    let optimized_update = update.as_ref().map(|u| Self::propagate_in_expr(u, &ctx));
                    let mut body_stmt = (**body).clone();
                    Self::propagate_statement(&mut body_stmt, &ctx);
                    statements[i] = Statement::For {
                        init: optimized_init,
                        test: optimized_test.map(Box::new),
                        update: optimized_update.map(Box::new),
                        body: Box::new(body_stmt),
                    };
                }
                Statement::Block(inner_statements) => {
                    let mut optimized_inner = inner_statements.clone();
                    Self::propagate(&mut optimized_inner, _cfg);
                    statements[i] = Statement::Block(optimized_inner);
                }
                Statement::Return(expr) => {
                    let optimized_expr = expr.as_ref().map(|e| Box::new(Self::propagate_in_expr(e, &ctx)));
                    statements[i] = Statement::Return(optimized_expr);
                }
                Statement::FunctionDeclaration { name, params, return_type, body, type_params, decorators } => {
                    let mut optimized_body = body.clone();
                    Self::propagate(&mut optimized_body, _cfg);
                    statements[i] = Statement::FunctionDeclaration {
                        name: name.clone(),
                        params: params.clone(),
                        return_type: return_type.clone(),
                        body: optimized_body,
                        type_params: type_params.clone(),
                        decorators: decorators.clone(),
                    };
                }
                _ => {}
            }
            i += 1;
        }
    }

    /// 对单个语句执行常量传播
    pub fn propagate_statement(stmt: &mut Statement, ctx: &ConstantPropagationContext) {
        match stmt {
            Statement::Expression(expr) => {
                *expr = Box::new(Self::propagate_in_expr(expr, ctx));
            }
            Statement::VariableDeclaration { initializer, .. } => {
                if let Some(init) = initializer {
                    *init = Box::new(Self::propagate_in_expr(init, ctx));
                }
            }
            Statement::If { test, consequent, alternate } => {
                *test = Box::new(Self::propagate_in_expr(test, ctx));
                Self::propagate_statement(consequent, ctx);
                if let Some(alt) = alternate {
                    Self::propagate_statement(alt, ctx);
                }
            }
            Statement::While { test, body } => {
                *test = Box::new(Self::propagate_in_expr(test, ctx));
                Self::propagate_statement(body, ctx);
            }
            Statement::For { init, test, update, body } => {
                if let Some(init_stmt) = init {
                    Self::propagate_statement(init_stmt, ctx);
                }
                if let Some(t) = test {
                    *t = Box::new(Self::propagate_in_expr(t, ctx));
                }
                if let Some(u) = update {
                    *u = Box::new(Self::propagate_in_expr(u, ctx));
                }
                Self::propagate_statement(body, ctx);
            }
            Statement::Return(expr) => {
                if let Some(e) = expr {
                    *e = Box::new(Self::propagate_in_expr(e, ctx));
                }
            }
            Statement::Block(statements) => {
                for s in statements {
                    Self::propagate_statement(s, ctx);
                }
            }
            _ => {}
        }
    }

    /// 在表达式中传播常量
    ///
    /// 递归遍历表达式，将已知的常量变量替换为其值，
    /// 并尝试在编译期计算表达式结果
    pub fn propagate_in_expr(expr: &Expression, ctx: &ConstantPropagationContext) -> Expression {
        match expr {
            Expression::Identifier(name) => {
                if let Some(constant_info) = ctx.get(name) {
                    return Expression::Literal(constant_info.value.clone());
                }
                expr.clone()
            }
            Expression::Binary { left, op, right } => {
                let optimized_left = Self::propagate_in_expr(left, ctx);
                let optimized_right = Self::propagate_in_expr(right, ctx);

                if let (Some(left_val), Some(right_val)) = (optimized_left.eval(), optimized_right.eval()) {
                    let result = Expression::eval_binary_op(left_val, op.clone(), right_val);
                    return Expression::Literal(result);
                }

                let folded = Self::fold_binary(&optimized_left, op, &optimized_right);
                if folded.is_some() {
                    return folded.unwrap();
                }

                Expression::Binary { left: Box::new(optimized_left), op: op.clone(), right: Box::new(optimized_right) }
            }
            Expression::Unary { op, expr: inner_expr } => {
                let optimized_inner = Self::propagate_in_expr(inner_expr, ctx);

                if let Some(inner_val) = optimized_inner.eval() {
                    let result = Expression::eval_unary_op(op.clone(), inner_val);
                    return Expression::Literal(result);
                }

                let folded = Self::fold_unary(op, &optimized_inner);
                if folded.is_some() {
                    return folded.unwrap();
                }

                Expression::Unary { op: op.clone(), expr: Box::new(optimized_inner) }
            }
            Expression::Call { callee, args } => {
                let optimized_callee = Self::propagate_in_expr(callee, ctx);
                let mut optimized_args = Vec::new();
                let mut all_args_constant = true;

                for arg in args {
                    let optimized_arg = Self::propagate_in_expr(arg, ctx);
                    if optimized_arg.eval().is_none() {
                        all_args_constant = false;
                    }
                    optimized_args.push(optimized_arg);
                }

                if all_args_constant {
                    if let Some(result) = Self::try_eval_builtin_call(&optimized_callee, &optimized_args, ctx) {
                        return result;
                    }
                }

                Expression::Call { callee: Box::new(optimized_callee), args: optimized_args }
            }
            Expression::Member { object, property } => {
                let optimized_object = Self::propagate_in_expr(object, ctx);
                let optimized_property = Self::propagate_in_expr(property, ctx);

                if let (Expression::Literal(obj), Expression::Literal(prop)) = (&optimized_object, &optimized_property) {
                    if let Some(result) = Self::try_get_member(obj, prop) {
                        return result;
                    }
                }

                Expression::Member { object: Box::new(optimized_object), property: Box::new(optimized_property) }
            }
            Expression::Index { object, index } => {
                let optimized_object = Self::propagate_in_expr(object, ctx);
                let optimized_index = Self::propagate_in_expr(index, ctx);

                if let (Expression::Literal(obj), Expression::Literal(idx)) = (&optimized_object, &optimized_index) {
                    if let Some(result) = Self::try_get_index(obj, idx) {
                        return result;
                    }
                }

                Expression::Index { object: Box::new(optimized_object), index: Box::new(optimized_index) }
            }
            Expression::Assignment { left, op, right } => {
                let optimized_left = Self::propagate_in_expr(left, ctx);
                let optimized_right = Self::propagate_in_expr(right, ctx);

                Expression::Assignment { left: Box::new(optimized_left), op: op.clone(), right: Box::new(optimized_right) }
            }
            Expression::Conditional { test, consequent, alternate } => {
                let optimized_test = Self::propagate_in_expr(test, ctx);

                if let Some(test_value) = optimized_test.eval() {
                    let condition = test_value.to_boolean();
                    if condition {
                        return Self::propagate_in_expr(consequent, ctx);
                    }
                    else {
                        return Self::propagate_in_expr(alternate, ctx);
                    }
                }

                let optimized_consequent = Self::propagate_in_expr(consequent, ctx);
                let optimized_alternate = Self::propagate_in_expr(alternate, ctx);

                Expression::Conditional {
                    test: Box::new(optimized_test),
                    consequent: Box::new(optimized_consequent),
                    alternate: Box::new(optimized_alternate),
                }
            }
            Expression::Array(elements) => {
                let mut optimized_elements = Vec::new();
                let mut all_constants = true;

                for elem in elements {
                    let optimized_elem = Self::propagate_in_expr(elem, ctx);
                    if optimized_elem.eval().is_none() {
                        all_constants = false;
                    }
                    optimized_elements.push(optimized_elem);
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
                    let optimized_value = Self::propagate_in_expr(value, ctx);
                    if optimized_value.eval().is_none() {
                        all_constants = false;
                    }
                    optimized_properties.push((key.clone(), optimized_value));
                }

                if all_constants {
                    if let Some(object_val) = Expression::Object(optimized_properties.clone()).eval() {
                        return Expression::Literal(object_val);
                    }
                }

                Expression::Object(optimized_properties)
            }
            Expression::Function { params, body } => {
                let mut optimized_body = body.clone();
                let mut inner_ctx = ctx.clone();
                for param in params {
                    inner_ctx.remove(param);
                }
                Self::propagate(&mut optimized_body, &ControlFlowGraph::new());
                Expression::Function { params: params.clone(), body: optimized_body }
            }
            Expression::ArrowFunction { params, body } => {
                let optimized_body = Self::propagate_in_expr(body, ctx);
                Expression::ArrowFunction { params: params.clone(), body: Box::new(optimized_body) }
            }
            Expression::Literal(_) => expr.clone(),
        }
    }

    /// 折叠二元表达式
    ///
    /// 对二元表达式进行代数简化，如 x + 0 = x, x * 1 = x 等
    pub fn fold_binary(left: &Expression, op: &crate::BinaryOp, right: &Expression) -> Option<Expression> {
        use crate::BinaryOp;
        use typescript_types::TsValue;

        match op {
            BinaryOp::Add => {
                if let Expression::Literal(TsValue::Number(n)) = left {
                    if *n == 0.0 {
                        return Some(right.clone());
                    }
                }
                if let Expression::Literal(TsValue::Number(n)) = right {
                    if *n == 0.0 {
                        return Some(left.clone());
                    }
                }
                if let Expression::Literal(TsValue::String(s)) = left {
                    if s.is_empty() {
                        return Some(right.clone());
                    }
                }
                if let Expression::Literal(TsValue::String(s)) = right {
                    if s.is_empty() {
                        return Some(left.clone());
                    }
                }
            }
            BinaryOp::Sub => {
                if let Expression::Literal(TsValue::Number(n)) = right {
                    if *n == 0.0 {
                        return Some(left.clone());
                    }
                }
                if let (Expression::Literal(TsValue::Number(a)), Expression::Literal(TsValue::Number(b))) = (left, right) {
                    if a == b {
                        return Some(Expression::Literal(TsValue::Number(0.0)));
                    }
                }
            }
            BinaryOp::Mul => {
                if let Expression::Literal(TsValue::Number(n)) = left {
                    if *n == 0.0 {
                        return Some(Expression::Literal(TsValue::Number(0.0)));
                    }
                    if *n == 1.0 {
                        return Some(right.clone());
                    }
                }
                if let Expression::Literal(TsValue::Number(n)) = right {
                    if *n == 0.0 {
                        return Some(Expression::Literal(TsValue::Number(0.0)));
                    }
                    if *n == 1.0 {
                        return Some(left.clone());
                    }
                }
            }
            BinaryOp::Div => {
                if let Expression::Literal(TsValue::Number(n)) = right {
                    if *n == 1.0 {
                        return Some(left.clone());
                    }
                }
                if let (Expression::Literal(TsValue::Number(a)), Expression::Literal(TsValue::Number(b))) = (left, right) {
                    if a == b && *a != 0.0 {
                        return Some(Expression::Literal(TsValue::Number(1.0)));
                    }
                }
            }
            BinaryOp::Mod => {
                if let Expression::Literal(TsValue::Number(n)) = right {
                    if *n == 1.0 {
                        return Some(Expression::Literal(TsValue::Number(0.0)));
                    }
                }
            }
            BinaryOp::And => {
                if let Expression::Literal(TsValue::Boolean(b)) = left {
                    if *b {
                        return Some(right.clone());
                    }
                    else {
                        return Some(Expression::Literal(TsValue::Boolean(false)));
                    }
                }
                if let Expression::Literal(TsValue::Boolean(b)) = right {
                    if *b {
                        return Some(left.clone());
                    }
                }
            }
            BinaryOp::Or => {
                if let Expression::Literal(TsValue::Boolean(b)) = left {
                    if *b {
                        return Some(Expression::Literal(TsValue::Boolean(true)));
                    }
                    else {
                        return Some(right.clone());
                    }
                }
                if let Expression::Literal(TsValue::Boolean(b)) = right {
                    if *b {
                        return Some(Expression::Literal(TsValue::Boolean(true)));
                    }
                }
            }
            BinaryOp::Pow => {
                if let Expression::Literal(TsValue::Number(n)) = right {
                    if *n == 0.0 {
                        return Some(Expression::Literal(TsValue::Number(1.0)));
                    }
                    if *n == 1.0 {
                        return Some(left.clone());
                    }
                }
                if let Expression::Literal(TsValue::Number(n)) = left {
                    if *n == 1.0 {
                        return Some(Expression::Literal(TsValue::Number(1.0)));
                    }
                }
            }
            _ => {}
        }
        None
    }

    /// 折叠一元表达式
    ///
    /// 对一元表达式进行简化，如 !!x -> Boolean(x), -(-x) -> x 等
    pub fn fold_unary(op: &crate::UnaryOp, expr: &Expression) -> Option<Expression> {
        use crate::UnaryOp;

        match op {
            UnaryOp::Not => {
                if let Expression::Unary { op: inner_op, expr: inner_expr } = expr {
                    if *inner_op == UnaryOp::Not {
                        return Some((**inner_expr).clone());
                    }
                }
                if let Expression::Literal(TsValue::Boolean(b)) = expr {
                    return Some(Expression::Literal(TsValue::Boolean(!b)));
                }
            }
            UnaryOp::Neg => {
                if let Expression::Unary { op: inner_op, expr: inner_expr } = expr {
                    if *inner_op == UnaryOp::Neg {
                        return Some((**inner_expr).clone());
                    }
                }
            }
            UnaryOp::BitNot => {
                if let Expression::Unary { op: inner_op, expr: inner_expr } = expr {
                    if *inner_op == UnaryOp::BitNot {
                        return Some((**inner_expr).clone());
                    }
                }
            }
            _ => {}
        }
        None
    }

    /// 尝试计算内置函数调用
    ///
    /// 对于已知的纯函数，如果所有参数都是常量，则在编译期计算结果
    pub fn try_eval_builtin_call(
        callee: &Expression,
        args: &[Expression],
        ctx: &ConstantPropagationContext,
    ) -> Option<Expression> {
        use typescript_types::TsValue;

        if let Expression::Member { object, property } = callee {
            if let (Expression::Identifier(obj_name), Expression::Identifier(prop_name)) = (object.as_ref(), property.as_ref())
            {
                if obj_name == "Math" {
                    let func_name = format!("Math.{}", prop_name);
                    if !ctx.is_pure_function(&func_name) {
                        return None;
                    }

                    let num_args: Vec<f64> = args
                        .iter()
                        .filter_map(|arg| if let Expression::Literal(TsValue::Number(n)) = arg { Some(*n) } else { None })
                        .collect();

                    if num_args.len() != args.len() {
                        return None;
                    }

                    let result = match prop_name.as_str() {
                        "abs" if num_args.len() == 1 => TsValue::Number(num_args[0].abs()),
                        "floor" if num_args.len() == 1 => TsValue::Number(num_args[0].floor()),
                        "ceil" if num_args.len() == 1 => TsValue::Number(num_args[0].ceil()),
                        "round" if num_args.len() == 1 => TsValue::Number(num_args[0].round()),
                        "sqrt" if num_args.len() == 1 => TsValue::Number(num_args[0].sqrt()),
                        "pow" if num_args.len() == 2 => TsValue::Number(num_args[0].powf(num_args[1])),
                        "min" if !num_args.is_empty() => {
                            TsValue::Number(num_args.iter().cloned().fold(f64::INFINITY, f64::min))
                        }
                        "max" if !num_args.is_empty() => {
                            TsValue::Number(num_args.iter().cloned().fold(f64::NEG_INFINITY, f64::max))
                        }
                        _ => return None,
                    };
                    return Some(Expression::Literal(result));
                }
            }
        }

        if let Expression::Identifier(func_name) = callee {
            if !ctx.is_pure_function(func_name) {
                return None;
            }

            match func_name.as_str() {
                "parseInt" => {
                    if args.len() >= 1 {
                        if let Expression::Literal(TsValue::String(s)) = &args[0] {
                            let radix = if args.len() > 1 {
                                if let Expression::Literal(TsValue::Number(r)) = &args[1] { *r as i32 } else { 10 }
                            }
                            else {
                                10
                            };
                            if let Ok(n) = i64::from_str_radix(s.trim(), radix as u32) {
                                return Some(Expression::Literal(TsValue::Number(n as f64)));
                            }
                        }
                        else if let Expression::Literal(TsValue::Number(n)) = &args[0] {
                            return Some(Expression::Literal(TsValue::Number(n.trunc())));
                        }
                    }
                }
                "parseFloat" => {
                    if args.len() >= 1 {
                        if let Expression::Literal(TsValue::String(s)) = &args[0] {
                            if let Ok(n) = s.trim().parse::<f64>() {
                                return Some(Expression::Literal(TsValue::Number(n)));
                            }
                        }
                        else if let Expression::Literal(TsValue::Number(n)) = &args[0] {
                            return Some(Expression::Literal(TsValue::Number(*n)));
                        }
                    }
                }
                "isNaN" => {
                    if args.len() == 1 {
                        if let Expression::Literal(TsValue::Number(n)) = &args[0] {
                            return Some(Expression::Literal(TsValue::Boolean(n.is_nan())));
                        }
                    }
                }
                "isFinite" => {
                    if args.len() == 1 {
                        if let Expression::Literal(TsValue::Number(n)) = &args[0] {
                            return Some(Expression::Literal(TsValue::Boolean(n.is_finite())));
                        }
                    }
                }
                _ => {}
            }
        }

        None
    }

    /// 尝试获取对象成员
    ///
    /// 对于常量对象和常量属性名，在编译期获取成员值
    pub fn try_get_member(obj: &TsValue, prop: &TsValue) -> Option<Expression> {
        use typescript_types::TsValue;

        match (obj, prop) {
            (TsValue::Object(map), TsValue::String(key)) => {
                if let Some(value) = map.get(key) {
                    return Some(Expression::Literal(value.clone()));
                }
            }
            (TsValue::Array(arr), TsValue::String(key)) => match key.as_str() {
                "length" => {
                    return Some(Expression::Literal(TsValue::Number(arr.len() as f64)));
                }
                _ => {}
            },
            (TsValue::String(s), TsValue::String(key)) => match key.as_str() {
                "length" => {
                    return Some(Expression::Literal(TsValue::Number(s.len() as f64)));
                }
                _ => {}
            },
            _ => {}
        }
        None
    }

    /// 尝试获取索引访问的值
    ///
    /// 对于常量数组和常量索引，在编译期获取元素值
    pub fn try_get_index(obj: &TsValue, idx: &TsValue) -> Option<Expression> {
        use typescript_types::TsValue;

        match (obj, idx) {
            (TsValue::Array(arr), TsValue::Number(n)) => {
                let index = *n as usize;
                if index < arr.len() {
                    return Some(Expression::Literal(arr[index].clone()));
                }
            }
            (TsValue::String(s), TsValue::Number(n)) => {
                let index = *n as usize;
                if index < s.len() {
                    let ch = s.chars().nth(index)?;
                    return Some(Expression::Literal(TsValue::String(ch.to_string())));
                }
            }
            (TsValue::Object(map), TsValue::String(key)) => {
                if let Some(value) = map.get(key) {
                    return Some(Expression::Literal(value.clone()));
                }
            }
            _ => {}
        }
        None
    }
}
