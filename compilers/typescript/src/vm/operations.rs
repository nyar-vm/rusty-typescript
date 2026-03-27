//! 运算操作模块
//!
//! 提供二元和一元运算的实现。

use typescript_types::{TsError, TsValue};

/// 二元运算执行器
pub struct BinaryOperations;

impl BinaryOperations {
    /// 执行二元加法
    pub fn add(left: TsValue, right: TsValue) -> Result<TsValue, TsError> {
        match (&left, &right) {
            (TsValue::Number(a), TsValue::Number(b)) => Ok(TsValue::Number(*a + *b)),
            (TsValue::String(a), TsValue::String(b)) => Ok(TsValue::String(format!("{}{}", a, b))),
            (TsValue::String(a), b) => Ok(TsValue::String(format!("{}{}", a, b.to_string()))),
            (a, TsValue::String(b)) => Ok(TsValue::String(format!("{}{}", a.to_string(), b))),
            _ => Ok(TsValue::Number(left.to_number() + right.to_number())),
        }
    }

    /// 执行二元减法
    pub fn sub(left: TsValue, right: TsValue) -> Result<TsValue, TsError> {
        Ok(TsValue::Number(left.to_number() - right.to_number()))
    }

    /// 执行二元乘法
    pub fn mul(left: TsValue, right: TsValue) -> Result<TsValue, TsError> {
        Ok(TsValue::Number(left.to_number() * right.to_number()))
    }

    /// 执行二元除法
    pub fn div(left: TsValue, right: TsValue) -> Result<TsValue, TsError> {
        Ok(TsValue::Number(left.to_number() / right.to_number()))
    }

    /// 执行二元取模
    pub fn modulo(left: TsValue, right: TsValue) -> Result<TsValue, TsError> {
        Ok(TsValue::Number(left.to_number() % right.to_number()))
    }

    /// 执行二元等于比较
    pub fn eq(left: TsValue, right: TsValue) -> TsValue {
        TsValue::Boolean(left.to_string() == right.to_string())
    }

    /// 执行二元不等于比较
    pub fn neq(left: TsValue, right: TsValue) -> TsValue {
        TsValue::Boolean(left.to_string() != right.to_string())
    }

    /// 执行二元严格等于比较
    pub fn strict_eq(left: TsValue, right: TsValue) -> TsValue {
        match (&left, &right) {
            (TsValue::Undefined, TsValue::Undefined) => TsValue::Boolean(true),
            (TsValue::Null, TsValue::Null) => TsValue::Boolean(true),
            (TsValue::Boolean(a), TsValue::Boolean(b)) => TsValue::Boolean(*a == *b),
            (TsValue::Number(a), TsValue::Number(b)) => TsValue::Boolean(*a == *b),
            (TsValue::String(a), TsValue::String(b)) => TsValue::Boolean(*a == *b),
            _ => TsValue::Boolean(false),
        }
    }

    /// 执行二元严格不等于比较
    pub fn strict_neq(left: TsValue, right: TsValue) -> TsValue {
        TsValue::Boolean(!Self::strict_eq(left, right).to_boolean())
    }

    /// 执行二元大于比较
    pub fn gt(left: TsValue, right: TsValue) -> Result<TsValue, TsError> {
        Ok(TsValue::Boolean(left.to_number() > right.to_number()))
    }

    /// 执行二元大于等于比较
    pub fn gte(left: TsValue, right: TsValue) -> Result<TsValue, TsError> {
        Ok(TsValue::Boolean(left.to_number() >= right.to_number()))
    }

    /// 执行二元小于比较
    pub fn lt(left: TsValue, right: TsValue) -> Result<TsValue, TsError> {
        Ok(TsValue::Boolean(left.to_number() < right.to_number()))
    }

    /// 执行二元小于等于比较
    pub fn lte(left: TsValue, right: TsValue) -> Result<TsValue, TsError> {
        Ok(TsValue::Boolean(left.to_number() <= right.to_number()))
    }

    /// 执行二元逻辑与
    pub fn and(left: TsValue, right: TsValue) -> TsValue {
        if left.to_boolean() { right } else { left }
    }

    /// 执行二元逻辑或
    pub fn or(left: TsValue, right: TsValue) -> TsValue {
        if left.to_boolean() { left } else { right }
    }

    /// 根据操作符执行二元运算
    pub fn execute(op: &str, left: TsValue, right: TsValue) -> Result<TsValue, TsError> {
        match op {
            "Add" => Self::add(left, right),
            "Sub" => Self::sub(left, right),
            "Mul" => Self::mul(left, right),
            "Div" => Self::div(left, right),
            "Mod" => Self::modulo(left, right),
            "Eq" => Ok(Self::eq(left, right)),
            "Neq" => Ok(Self::neq(left, right)),
            "StrictEq" => Ok(Self::strict_eq(left, right)),
            "StrictNeq" => Ok(Self::strict_neq(left, right)),
            "Gt" => Self::gt(left, right),
            "Gte" => Self::gte(left, right),
            "Lt" => Self::lt(left, right),
            "Lte" => Self::lte(left, right),
            "And" => Ok(Self::and(left, right)),
            "Or" => Ok(Self::or(left, right)),
            _ => Ok(TsValue::Undefined),
        }
    }
}

/// 一元运算执行器
pub struct UnaryOperations;

impl UnaryOperations {
    /// 获取值的类型字符串
    pub fn type_of(value: &TsValue) -> String {
        match value {
            TsValue::Undefined => "undefined".to_string(),
            TsValue::Null => "object".to_string(),
            TsValue::Boolean(_) => "boolean".to_string(),
            TsValue::Number(_) => "number".to_string(),
            TsValue::String(_) => "string".to_string(),
            TsValue::Object(_) => "object".to_string(),
            TsValue::Array(_) => "object".to_string(),
            TsValue::Function(_) => "function".to_string(),
            TsValue::Error(_) => "object".to_string(),
            TsValue::Union(_) => "object".to_string(),
            TsValue::Generic(_, _) => "object".to_string(),
            TsValue::Symbol(_) => "symbol".to_string(),
            TsValue::BigInt(_) => "bigint".to_string(),
            TsValue::Date(_) => "object".to_string(),
            TsValue::RegExp(_) => "object".to_string(),
            TsValue::Map(_) => "object".to_string(),
            TsValue::Set(_) => "object".to_string(),
            TsValue::Promise(_) => "object".to_string(),
            TsValue::Iterable(_) => "object".to_string(),
        }
    }

    /// 根据操作符执行一元运算
    pub fn execute(op: &str, value: TsValue) -> Result<TsValue, TsError> {
        let result = match op {
            "Not" => TsValue::Boolean(!value.to_boolean()),
            "Neg" => TsValue::Number(-value.to_number()),
            "Pos" => TsValue::Number(value.to_number()),
            "TypeOf" => TsValue::String(Self::type_of(&value)),
            "Void" => TsValue::Undefined,
            "Delete" => TsValue::Boolean(false),
            "BitNot" => TsValue::Number(!(value.to_number() as i64) as f64),
            "Inc" => TsValue::Number(value.to_number() + 1.0),
            "Dec" => TsValue::Number(value.to_number() - 1.0),
            _ => TsValue::Undefined,
        };
        Ok(result)
    }
}
