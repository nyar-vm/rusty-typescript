#![warn(missing_docs)]

use std::{
    hash::{Hash, Hasher},
    rc::Rc,
};

/// TypeScript 值类型枚举
pub enum TsValue {
    /// 未定义
    Undefined,
    /// 空值
    Null,
    /// 布尔值
    Boolean(bool),
    /// 数字
    Number(f64),
    /// 字符串
    String(String),
    /// 对象
    Object(Vec<(String, TsValue)>),
    /// 数组
    Array(Vec<TsValue>),
    /// 函数
    Function(Rc<dyn Fn(&[TsValue]) -> TsValue>),
    /// 错误
    Error(String),
    /// 联合类型
    Union(Vec<TsValue>),
    /// 泛型类型
    Generic(String, Vec<TsValue>),
    /// 符号
    Symbol(String),
    /// 大整数
    BigInt(i128),
    /// 日期
    Date(i64),
    /// 正则表达式
    RegExp(String),
    /// Map
    Map(Vec<(TsValue, TsValue)>),
    /// Set
    Set(Vec<TsValue>),
    /// Promise
    Promise(Box<TsValue>),
    /// 可迭代对象
    Iterable(Box<dyn Iterator<Item = TsValue>>),
}

impl Clone for TsValue {
    fn clone(&self) -> Self {
        match self {
            TsValue::Undefined => TsValue::Undefined,
            TsValue::Null => TsValue::Null,
            TsValue::Boolean(b) => TsValue::Boolean(*b),
            TsValue::Number(n) => TsValue::Number(*n),
            TsValue::String(s) => TsValue::String(s.clone()),
            TsValue::Object(props) => TsValue::Object(props.clone()),
            TsValue::Array(arr) => TsValue::Array(arr.clone()),
            TsValue::Function(f) => TsValue::Function(Rc::clone(f)), // 函数可以克隆
            TsValue::Error(s) => TsValue::Error(s.clone()),
            TsValue::Union(values) => TsValue::Union(values.clone()),
            TsValue::Generic(name, args) => TsValue::Generic(name.clone(), args.clone()),
            TsValue::Symbol(s) => TsValue::Symbol(s.clone()),
            TsValue::BigInt(bi) => TsValue::BigInt(*bi),
            TsValue::Date(d) => TsValue::Date(*d),
            TsValue::RegExp(pattern) => TsValue::RegExp(pattern.clone()),
            TsValue::Map(entries) => TsValue::Map(entries.clone()),
            TsValue::Set(values) => TsValue::Set(values.clone()),
            TsValue::Promise(value) => TsValue::Promise(value.clone()),
            TsValue::Iterable(_) => TsValue::Undefined, // 可迭代对象无法克隆，返回 Undefined
        }
    }
}

impl std::fmt::Debug for TsValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TsValue::Undefined => write!(f, "Undefined"),
            TsValue::Null => write!(f, "Null"),
            TsValue::Boolean(_) => write!(f, "Boolean"),
            TsValue::Number(_) => write!(f, "Number"),
            TsValue::String(_) => write!(f, "String"),
            TsValue::Object(_) => write!(f, "Object"),
            TsValue::Array(_) => write!(f, "Array"),
            TsValue::Function(_) => write!(f, "Function"),
            TsValue::Error(_) => write!(f, "Error"),
            TsValue::Union(_) => write!(f, "Union"),
            TsValue::Generic(name, _) => write!(f, "Generic({})", name),
            TsValue::Symbol(_) => write!(f, "Symbol"),
            TsValue::BigInt(_) => write!(f, "BigInt"),
            TsValue::Date(_) => write!(f, "Date"),
            TsValue::RegExp(_) => write!(f, "RegExp"),
            TsValue::Map(_) => write!(f, "Map"),
            TsValue::Set(_) => write!(f, "Set"),
            TsValue::Promise(_) => write!(f, "Promise"),
            TsValue::Iterable(_) => write!(f, "Iterable"),
        }
    }
}

impl Hash for TsValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            TsValue::Undefined => 0.hash(state),
            TsValue::Null => 1.hash(state),
            TsValue::Boolean(b) => {
                2.hash(state);
                b.hash(state);
            }
            TsValue::Number(n) => {
                3.hash(state);
                n.to_bits().hash(state);
            }
            TsValue::String(s) => {
                4.hash(state);
                s.hash(state);
            }
            TsValue::Object(props) => {
                5.hash(state);
                props.len().hash(state);
                for (key, value) in props {
                    key.hash(state);
                    value.hash(state);
                }
            }
            TsValue::Array(arr) => {
                6.hash(state);
                arr.len().hash(state);
                for item in arr {
                    item.hash(state);
                }
            }
            TsValue::Function(_) => 7.hash(state),
            TsValue::Error(s) => {
                8.hash(state);
                s.hash(state);
            }
            TsValue::Union(values) => {
                9.hash(state);
                values.len().hash(state);
                for value in values {
                    value.hash(state);
                }
            }
            TsValue::Generic(name, args) => {
                10.hash(state);
                name.hash(state);
                args.len().hash(state);
                for arg in args {
                    arg.hash(state);
                }
            }
            TsValue::Symbol(s) => {
                11.hash(state);
                s.hash(state);
            }
            TsValue::BigInt(bi) => {
                12.hash(state);
                bi.hash(state);
            }
            TsValue::Date(d) => {
                13.hash(state);
                d.hash(state);
            }
            TsValue::RegExp(pattern) => {
                14.hash(state);
                pattern.hash(state);
            }
            TsValue::Map(entries) => {
                15.hash(state);
                entries.len().hash(state);
                for (key, value) in entries {
                    key.hash(state);
                    value.hash(state);
                }
            }
            TsValue::Set(values) => {
                16.hash(state);
                values.len().hash(state);
                for value in values {
                    value.hash(state);
                }
            }
            TsValue::Promise(value) => {
                17.hash(state);
                value.hash(state);
            }
            TsValue::Iterable(_) => 18.hash(state),
        }
    }
}

impl TsValue {
    /// 检查是否为未定义
    pub fn is_undefined(&self) -> bool {
        matches!(self, TsValue::Undefined)
    }

    /// 检查是否为空值
    pub fn is_null(&self) -> bool {
        matches!(self, TsValue::Null)
    }

    /// 检查是否为布尔值
    pub fn is_boolean(&self) -> bool {
        matches!(self, TsValue::Boolean(_))
    }

    /// 检查是否为数字
    pub fn is_number(&self) -> bool {
        matches!(self, TsValue::Number(_))
    }

    /// 检查是否为字符串
    pub fn is_string(&self) -> bool {
        matches!(self, TsValue::String(_))
    }

    /// 检查是否为对象
    pub fn is_object(&self) -> bool {
        matches!(self, TsValue::Object(_))
    }

    /// 检查是否为数组
    pub fn is_array(&self) -> bool {
        matches!(self, TsValue::Array(_))
    }

    /// 检查是否为函数
    pub fn is_function(&self) -> bool {
        matches!(self, TsValue::Function(_))
    }

    /// 检查是否为错误
    pub fn is_error(&self) -> bool {
        matches!(self, TsValue::Error(_))
    }

    /// 检查是否为联合类型
    pub fn is_union(&self) -> bool {
        matches!(self, TsValue::Union(_))
    }

    /// 检查是否为泛型类型
    pub fn is_generic(&self) -> bool {
        matches!(self, TsValue::Generic(_, _))
    }

    /// 检查是否为符号
    pub fn is_symbol(&self) -> bool {
        matches!(self, TsValue::Symbol(_))
    }

    /// 检查是否为大整数
    pub fn is_bigint(&self) -> bool {
        matches!(self, TsValue::BigInt(_))
    }

    /// 检查是否为日期
    pub fn is_date(&self) -> bool {
        matches!(self, TsValue::Date(_))
    }

    /// 检查是否为正则表达式
    pub fn is_regexp(&self) -> bool {
        matches!(self, TsValue::RegExp(_))
    }

    /// 检查是否为 Map
    pub fn is_map(&self) -> bool {
        matches!(self, TsValue::Map(_))
    }

    /// 检查是否为 Set
    pub fn is_set(&self) -> bool {
        matches!(self, TsValue::Set(_))
    }

    /// 检查是否为 Promise
    pub fn is_promise(&self) -> bool {
        matches!(self, TsValue::Promise(_))
    }

    /// 检查是否为可迭代对象
    pub fn is_iterable(&self) -> bool {
        matches!(self, TsValue::Iterable(_))
    }

    /// 转换为布尔值
    pub fn to_boolean(&self) -> bool {
        match self {
            TsValue::Undefined => false,
            TsValue::Null => false,
            TsValue::Boolean(b) => *b,
            TsValue::Number(n) => *n != 0.0 && !n.is_nan(),
            TsValue::String(s) => !s.is_empty(),
            TsValue::Object(_) => true,
            TsValue::Array(_) => true,
            TsValue::Function(_) => true,
            TsValue::Error(_) => true,
            TsValue::Union(values) => !values.is_empty(),
            TsValue::Generic(_, _) => true,
            TsValue::Symbol(_) => true,
            TsValue::BigInt(bi) => *bi != 0,
            TsValue::Date(_) => true,
            TsValue::RegExp(_) => true,
            TsValue::Map(entries) => !entries.is_empty(),
            TsValue::Set(values) => !values.is_empty(),
            TsValue::Promise(_) => true,
            TsValue::Iterable(_) => true,
        }
    }

    /// 转换为数字
    pub fn to_number(&self) -> f64 {
        match self {
            TsValue::Undefined => f64::NAN,
            TsValue::Null => 0.0,
            TsValue::Boolean(b) => {
                if *b {
                    1.0
                }
                else {
                    0.0
                }
            }
            TsValue::Number(n) => *n,
            TsValue::String(s) => s.parse().unwrap_or(f64::NAN),
            TsValue::Object(_) => f64::NAN,
            TsValue::Array(_) => {
                if let TsValue::Array(arr) = self {
                    if arr.is_empty() { 0.0 } else { f64::NAN }
                }
                else {
                    f64::NAN
                }
            }
            TsValue::Function(_) => f64::NAN,
            TsValue::Error(_) => f64::NAN,
            TsValue::Union(values) => {
                if values.is_empty() {
                    0.0
                }
                else {
                    values[0].to_number()
                }
            }
            TsValue::Generic(_, _) => f64::NAN,
            TsValue::Symbol(_) => f64::NAN,
            TsValue::BigInt(bi) => *bi as f64,
            TsValue::Date(d) => *d as f64,
            TsValue::RegExp(_) => f64::NAN,
            TsValue::Map(_) => f64::NAN,
            TsValue::Set(_) => f64::NAN,
            TsValue::Promise(_) => f64::NAN,
            TsValue::Iterable(_) => f64::NAN,
        }
    }

    /// 转换为字符串
    pub fn to_string(&self) -> String {
        match self {
            TsValue::Undefined => "undefined".to_string(),
            TsValue::Null => "null".to_string(),
            TsValue::Boolean(b) => b.to_string(),
            TsValue::Number(n) => n.to_string(),
            TsValue::String(s) => s.clone(),
            TsValue::Object(_) => "[object Object]".to_string(),
            TsValue::Array(_) => {
                if let TsValue::Array(arr) = self {
                    let elements: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                    format!("[{}]", elements.join(", "))
                }
                else {
                    "[object Array]".to_string()
                }
            }
            TsValue::Function(_) => "[Function]".to_string(),
            TsValue::Error(s) => format!("Error: {}", s),
            TsValue::Union(values) => {
                let elements: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                format!("Union({})", elements.join(", "))
            }
            TsValue::Generic(name, args) => {
                let args_str: Vec<String> = args.iter().map(|v| v.to_string()).collect();
                format!("{}{{ {} }}", name, args_str.join(", "))
            }
            TsValue::Symbol(s) => format!("Symbol({})", s),
            TsValue::BigInt(bi) => bi.to_string(),
            TsValue::Date(d) => format!("Date({})", d),
            TsValue::RegExp(pattern) => format!("/{}/", pattern),
            TsValue::Map(entries) => {
                let entries_str: Vec<String> =
                    entries.iter().map(|(k, v)| format!("{}: {}", k.to_string(), v.to_string())).collect();
                format!("Map({})", entries_str.join(", "))
            }
            TsValue::Set(values) => {
                let values_str: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                format!("Set({})", values_str.join(", "))
            }
            TsValue::Promise(value) => format!("Promise<{}>", value.to_string()),
            TsValue::Iterable(_) => "[object Iterable]".to_string(),
        }
    }
}

/// TypeScript 错误类型枚举
#[derive(Debug, Clone)]
pub enum TsError {
    /// 类型错误
    TypeError(String),
    /// 引用错误
    ReferenceError(String),
    /// 语法错误
    SyntaxError(String),
    /// 范围错误
    RangeError(String),
    /// 其他错误
    Other(String),
}

impl std::fmt::Display for TsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TsError::TypeError(msg) => write!(f, "TypeError: {}", msg),
            TsError::ReferenceError(msg) => write!(f, "ReferenceError: {}", msg),
            TsError::SyntaxError(msg) => write!(f, "SyntaxError: {}", msg),
            TsError::RangeError(msg) => write!(f, "RangeError: {}", msg),
            TsError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

/// 从 Rust 类型转换为 TypeScript 值的 trait
pub trait ToTsValue {
    /// 转换为 TypeScript 值
    fn to_ts_value(&self) -> TsValue;
}

impl ToTsValue for bool {
    fn to_ts_value(&self) -> TsValue {
        TsValue::Boolean(*self)
    }
}

impl ToTsValue for f64 {
    fn to_ts_value(&self) -> TsValue {
        TsValue::Number(*self)
    }
}

impl ToTsValue for i32 {
    fn to_ts_value(&self) -> TsValue {
        TsValue::Number(*self as f64)
    }
}

impl ToTsValue for &str {
    fn to_ts_value(&self) -> TsValue {
        TsValue::String(self.to_string())
    }
}

impl ToTsValue for String {
    fn to_ts_value(&self) -> TsValue {
        TsValue::String(self.clone())
    }
}

impl<T: ToTsValue> ToTsValue for Vec<T> {
    fn to_ts_value(&self) -> TsValue {
        let values: Vec<TsValue> = self.iter().map(|v| v.to_ts_value()).collect();
        TsValue::Array(values)
    }
}

impl<K: ToString, V: ToTsValue> ToTsValue for Vec<(K, V)> {
    fn to_ts_value(&self) -> TsValue {
        let entries: Vec<(String, TsValue)> = self.iter().map(|(k, v)| (k.to_string(), v.to_ts_value())).collect();
        TsValue::Object(entries)
    }
}

impl ToTsValue for i128 {
    fn to_ts_value(&self) -> TsValue {
        TsValue::BigInt(*self)
    }
}

impl ToTsValue for i64 {
    fn to_ts_value(&self) -> TsValue {
        TsValue::Date(*self)
    }
}

impl<T: ToTsValue> ToTsValue for Option<T> {
    fn to_ts_value(&self) -> TsValue {
        match self {
            Some(value) => value.to_ts_value(),
            None => TsValue::Null,
        }
    }
}

impl<T: ToTsValue, E: ToString> ToTsValue for Result<T, E> {
    fn to_ts_value(&self) -> TsValue {
        match self {
            Ok(value) => value.to_ts_value(),
            Err(error) => TsValue::Error(error.to_string()),
        }
    }
}

impl ToTsValue for std::collections::HashMap<String, TsValue> {
    fn to_ts_value(&self) -> TsValue {
        let entries: Vec<(String, TsValue)> = self.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        TsValue::Object(entries)
    }
}

impl ToTsValue for std::collections::HashSet<TsValue> {
    fn to_ts_value(&self) -> TsValue {
        let values: Vec<TsValue> = self.iter().cloned().collect();
        TsValue::Set(values)
    }
}

impl ToTsValue for std::collections::HashMap<TsValue, TsValue> {
    fn to_ts_value(&self) -> TsValue {
        let entries: Vec<(TsValue, TsValue)> = self.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        TsValue::Map(entries)
    }
}
