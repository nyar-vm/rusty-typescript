#![warn(missing_docs)]
#![doc = "TypeScript 高级类型系统实现"]

use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    rc::Rc,
};

/// 条件类型约束修饰符
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConditionalModifier {
    /// 无修饰符，普通条件类型
    None,
    /// infer 关键字，用于类型推断
    Infer {
        /// 推断的类型参数名称
        type_param: String,
    },
}

/// 条件类型，表示 T extends U ? X : Y
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Conditional {
    /// 检查类型 (T)
    pub check_type: Box<TsValue>,
    /// 扩展类型 (U)
    pub extends_type: Box<TsValue>,
    /// 真分支类型 (X)
    pub true_type: Box<TsValue>,
    /// 假分支类型 (Y)
    pub false_type: Box<TsValue>,
    /// 约束修饰符
    pub modifier: ConditionalModifier,
}

impl Conditional {
    /// 创建新的条件类型
    pub fn new(check_type: TsValue, extends_type: TsValue, true_type: TsValue, false_type: TsValue) -> Self {
        Self {
            check_type: Box::new(check_type),
            extends_type: Box::new(extends_type),
            true_type: Box::new(true_type),
            false_type: Box::new(false_type),
            modifier: ConditionalModifier::None,
        }
    }

    /// 创建带有 infer 修饰符的条件类型
    pub fn with_infer(check_type: TsValue, extends_type: TsValue, type_param: String) -> Self {
        Self {
            check_type: Box::new(check_type),
            extends_type: Box::new(extends_type),
            true_type: Box::new(TsValue::Undefined),
            false_type: Box::new(TsValue::Undefined),
            modifier: ConditionalModifier::Infer { type_param },
        }
    }

    /// 设置真分支类型
    pub fn with_true_type(mut self, true_type: TsValue) -> Self {
        self.true_type = Box::new(true_type);
        self
    }

    /// 设置假分支类型
    pub fn with_false_type(mut self, false_type: TsValue) -> Self {
        self.false_type = Box::new(false_type);
        self
    }
}

/// 映射类型约束
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MappedConstraint {
    /// 无约束
    None,
    /// 可选属性 (?)
    Optional,
    /// 必选属性 (-?)
    Required,
    /// 只读属性 (readonly)
    Readonly,
    /// 可写属性 (-readonly)
    Writable,
}

/// 映射类型键修饰符
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MappedKeyModifier {
    /// 是否为可选
    pub optional: bool,
    /// 是否为只读
    pub readonly: bool,
}

impl Default for MappedKeyModifier {
    fn default() -> Self {
        Self { optional: false, readonly: false }
    }
}

/// 映射类型，表示 { [K in keyof T]: V } 或类似形式
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Mapped {
    /// 类型参数名称 (K)
    pub type_param: String,
    /// 键的来源类型 (keyof T)
    pub constraint: Box<TsValue>,
    /// 值类型表达式，可以引用类型参数
    pub value_type: Box<TsValue>,
    /// 键修饰符
    pub key_modifier: MappedKeyModifier,
    /// 约束类型
    pub constraint_type: MappedConstraint,
}

impl Mapped {
    /// 创建新的映射类型
    pub fn new(type_param: String, constraint: TsValue, value_type: TsValue) -> Self {
        Self {
            type_param,
            constraint: Box::new(constraint),
            value_type: Box::new(value_type),
            key_modifier: MappedKeyModifier::default(),
            constraint_type: MappedConstraint::None,
        }
    }

    /// 设置为可选属性
    pub fn with_optional(mut self) -> Self {
        self.key_modifier.optional = true;
        self
    }

    /// 设置为只读属性
    pub fn with_readonly(mut self) -> Self {
        self.key_modifier.readonly = true;
        self
    }

    /// 设置约束类型
    pub fn with_constraint(mut self, constraint: MappedConstraint) -> Self {
        self.constraint_type = constraint;
        self
    }
}

/// 模板字面量类型片段
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TemplateLiteralPart {
    /// 字符串字面量
    String(String),
    /// 类型占位符，引用另一个类型
    Type(Box<TsValue>),
    /// 数字占位符
    Number,
    /// 字符串占位符
    StringType,
    /// 大整数占位符
    BigInt,
    /// 联合类型占位符
    Union(Vec<TsValue>),
}

/// 模板字面量类型，表示 `${string}` 或更复杂的模板
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TemplateLiteral {
    /// 模板片段列表
    pub parts: Vec<TemplateLiteralPart>,
}

impl TemplateLiteral {
    /// 创建空的模板字面量类型
    pub fn new() -> Self {
        Self { parts: Vec::new() }
    }

    /// 从片段列表创建模板字面量类型
    pub fn from_parts(parts: Vec<TemplateLiteralPart>) -> Self {
        Self { parts }
    }

    /// 添加字符串字面量片段
    pub fn push_string(&mut self, s: String) {
        self.parts.push(TemplateLiteralPart::String(s));
    }

    /// 添加类型占位符
    pub fn push_type(&mut self, ty: TsValue) {
        self.parts.push(TemplateLiteralPart::Type(Box::new(ty)));
    }

    /// 添加数字占位符
    pub fn push_number(&mut self) {
        self.parts.push(TemplateLiteralPart::Number);
    }

    /// 添加字符串类型占位符
    pub fn push_string_type(&mut self) {
        self.parts.push(TemplateLiteralPart::StringType);
    }

    /// 添加大整数占位符
    pub fn push_bigint(&mut self) {
        self.parts.push(TemplateLiteralPart::BigInt);
    }
}

impl Default for TemplateLiteral {
    fn default() -> Self {
        Self::new()
    }
}

/// 类型推断结果
#[derive(Debug, Clone, PartialEq)]
pub struct InferenceResult {
    /// 推断出的类型映射
    pub inferred_types: HashMap<String, TsValue>,
    /// 是否推断成功
    pub success: bool,
}

impl InferenceResult {
    /// 创建成功的推断结果
    pub fn success(inferred_types: HashMap<String, TsValue>) -> Self {
        Self { inferred_types, success: true }
    }

    /// 创建失败的推断结果
    pub fn failure() -> Self {
        Self { inferred_types: HashMap::new(), success: false }
    }

    /// 合并两个推断结果
    pub fn merge(self, other: InferenceResult) -> Self {
        if !self.success || !other.success {
            return InferenceResult::failure();
        }
        let mut merged = self.inferred_types;
        merged.extend(other.inferred_types);
        InferenceResult::success(merged)
    }
}

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
    Object(HashMap<String, TsValue>),
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
    /// 条件类型 T extends U ? X : Y
    Conditional(Conditional),
    /// 映射类型 { [K in keyof T]: V }
    Mapped(Mapped),
    /// 模板字面量类型
    TemplateLiteral(TemplateLiteral),
    /// keyof 类型操作符结果
    KeyOf(Box<TsValue>),
    /// typeof 类型操作符结果
    TypeOf(Box<TsValue>),
    /// 索引访问类型 T[K]
    IndexedAccess {
        /// 对象类型
        object_type: Box<TsValue>,
        /// 索引类型
        index_type: Box<TsValue>,
    },
    /// 元组类型
    Tuple(Vec<TsValue>),
    /// 只读类型
    Readonly(Box<TsValue>),
    /// 可空类型 (T | null)
    Nullable(Box<TsValue>),
    /// 不可空类型 (NonNullable<T>)
    NonNullable(Box<TsValue>),
    /// 推断类型 (infer U)
    Infer {
        /// 类型参数名称
        type_param: String,
        /// 约束类型
        constraint: Option<Box<TsValue>>,
    },
    /// 函数类型
    FunctionType {
        /// 参数列表
        params: Vec<(String, TsValue)>,
        /// 返回类型
        return_type: Box<TsValue>,
    },
    /// 构造函数类型
    ConstructorType {
        /// 参数列表
        params: Vec<(String, TsValue)>,
        /// 返回类型
        return_type: Box<TsValue>,
    },
    /// this 类型
    ThisType,
    /// never 类型
    Never,
    /// unknown 类型
    Unknown,
    /// any 类型
    Any,
    /// void 类型
    Void,
}

impl PartialEq for TsValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TsValue::Undefined, TsValue::Undefined) => true,
            (TsValue::Null, TsValue::Null) => true,
            (TsValue::Boolean(a), TsValue::Boolean(b)) => a == b,
            (TsValue::Number(a), TsValue::Number(b)) => a.to_bits() == b.to_bits(),
            (TsValue::String(a), TsValue::String(b)) => a == b,
            (TsValue::Object(a), TsValue::Object(b)) => a == b,
            (TsValue::Array(a), TsValue::Array(b)) => a == b,
            (TsValue::Function(_), TsValue::Function(_)) => false,
            (TsValue::Error(a), TsValue::Error(b)) => a == b,
            (TsValue::Union(a), TsValue::Union(b)) => a == b,
            (TsValue::Generic(name_a, args_a), TsValue::Generic(name_b, args_b)) => name_a == name_b && args_a == args_b,
            (TsValue::Symbol(a), TsValue::Symbol(b)) => a == b,
            (TsValue::BigInt(a), TsValue::BigInt(b)) => a == b,
            (TsValue::Date(a), TsValue::Date(b)) => a == b,
            (TsValue::RegExp(a), TsValue::RegExp(b)) => a == b,
            (TsValue::Map(a), TsValue::Map(b)) => a == b,
            (TsValue::Set(a), TsValue::Set(b)) => a == b,
            (TsValue::Promise(a), TsValue::Promise(b)) => a == b,
            (TsValue::Iterable(_), TsValue::Iterable(_)) => false,
            (TsValue::Conditional(a), TsValue::Conditional(b)) => a == b,
            (TsValue::Mapped(a), TsValue::Mapped(b)) => a == b,
            (TsValue::TemplateLiteral(a), TsValue::TemplateLiteral(b)) => a == b,
            (TsValue::KeyOf(a), TsValue::KeyOf(b)) => a == b,
            (TsValue::TypeOf(a), TsValue::TypeOf(b)) => a == b,
            (
                TsValue::IndexedAccess { object_type: obj_a, index_type: idx_a },
                TsValue::IndexedAccess { object_type: obj_b, index_type: idx_b },
            ) => obj_a == obj_b && idx_a == idx_b,
            (TsValue::Tuple(a), TsValue::Tuple(b)) => a == b,
            (TsValue::Readonly(a), TsValue::Readonly(b)) => a == b,
            (TsValue::Nullable(a), TsValue::Nullable(b)) => a == b,
            (TsValue::NonNullable(a), TsValue::NonNullable(b)) => a == b,
            (
                TsValue::Infer { type_param: param_a, constraint: constraint_a },
                TsValue::Infer { type_param: param_b, constraint: constraint_b },
            ) => param_a == param_b && constraint_a == constraint_b,
            (
                TsValue::FunctionType { params: params_a, return_type: ret_a },
                TsValue::FunctionType { params: params_b, return_type: ret_b },
            ) => params_a == params_b && ret_a == ret_b,
            (
                TsValue::ConstructorType { params: params_a, return_type: ret_a },
                TsValue::ConstructorType { params: params_b, return_type: ret_b },
            ) => params_a == params_b && ret_a == ret_b,
            (TsValue::ThisType, TsValue::ThisType) => true,
            (TsValue::Never, TsValue::Never) => true,
            (TsValue::Unknown, TsValue::Unknown) => true,
            (TsValue::Any, TsValue::Any) => true,
            (TsValue::Void, TsValue::Void) => true,
            _ => false,
        }
    }
}

impl Eq for TsValue {}

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
            TsValue::Function(f) => TsValue::Function(Rc::clone(f)),
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
            TsValue::Iterable(_) => TsValue::Undefined,
            TsValue::Conditional(cond) => TsValue::Conditional(cond.clone()),
            TsValue::Mapped(mapped) => TsValue::Mapped(mapped.clone()),
            TsValue::TemplateLiteral(tpl) => TsValue::TemplateLiteral(tpl.clone()),
            TsValue::KeyOf(ty) => TsValue::KeyOf(ty.clone()),
            TsValue::TypeOf(ty) => TsValue::TypeOf(ty.clone()),
            TsValue::IndexedAccess { object_type, index_type } => {
                TsValue::IndexedAccess { object_type: object_type.clone(), index_type: index_type.clone() }
            }
            TsValue::Tuple(elements) => TsValue::Tuple(elements.clone()),
            TsValue::Readonly(ty) => TsValue::Readonly(ty.clone()),
            TsValue::Nullable(ty) => TsValue::Nullable(ty.clone()),
            TsValue::NonNullable(ty) => TsValue::NonNullable(ty.clone()),
            TsValue::Infer { type_param, constraint } => {
                TsValue::Infer { type_param: type_param.clone(), constraint: constraint.clone() }
            }
            TsValue::FunctionType { params, return_type } => {
                TsValue::FunctionType { params: params.clone(), return_type: return_type.clone() }
            }
            TsValue::ConstructorType { params, return_type } => {
                TsValue::ConstructorType { params: params.clone(), return_type: return_type.clone() }
            }
            TsValue::ThisType => TsValue::ThisType,
            TsValue::Never => TsValue::Never,
            TsValue::Unknown => TsValue::Unknown,
            TsValue::Any => TsValue::Any,
            TsValue::Void => TsValue::Void,
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
            TsValue::Conditional(_) => write!(f, "Conditional"),
            TsValue::Mapped(_) => write!(f, "Mapped"),
            TsValue::TemplateLiteral(_) => write!(f, "TemplateLiteral"),
            TsValue::KeyOf(_) => write!(f, "KeyOf"),
            TsValue::TypeOf(_) => write!(f, "TypeOf"),
            TsValue::IndexedAccess { .. } => write!(f, "IndexedAccess"),
            TsValue::Tuple(_) => write!(f, "Tuple"),
            TsValue::Readonly(_) => write!(f, "Readonly"),
            TsValue::Nullable(_) => write!(f, "Nullable"),
            TsValue::NonNullable(_) => write!(f, "NonNullable"),
            TsValue::Infer { type_param, .. } => write!(f, "Infer({})", type_param),
            TsValue::FunctionType { .. } => write!(f, "FunctionType"),
            TsValue::ConstructorType { .. } => write!(f, "ConstructorType"),
            TsValue::ThisType => write!(f, "ThisType"),
            TsValue::Never => write!(f, "Never"),
            TsValue::Unknown => write!(f, "Unknown"),
            TsValue::Any => write!(f, "Any"),
            TsValue::Void => write!(f, "Void"),
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
            TsValue::Conditional(cond) => {
                19.hash(state);
                cond.hash(state);
            }
            TsValue::Mapped(mapped) => {
                20.hash(state);
                mapped.hash(state);
            }
            TsValue::TemplateLiteral(tpl) => {
                21.hash(state);
                tpl.hash(state);
            }
            TsValue::KeyOf(ty) => {
                22.hash(state);
                ty.hash(state);
            }
            TsValue::TypeOf(ty) => {
                23.hash(state);
                ty.hash(state);
            }
            TsValue::IndexedAccess { object_type, index_type } => {
                24.hash(state);
                object_type.hash(state);
                index_type.hash(state);
            }
            TsValue::Tuple(elements) => {
                25.hash(state);
                elements.len().hash(state);
                for elem in elements {
                    elem.hash(state);
                }
            }
            TsValue::Readonly(ty) => {
                26.hash(state);
                ty.hash(state);
            }
            TsValue::Nullable(ty) => {
                27.hash(state);
                ty.hash(state);
            }
            TsValue::NonNullable(ty) => {
                28.hash(state);
                ty.hash(state);
            }
            TsValue::Infer { type_param, constraint } => {
                29.hash(state);
                type_param.hash(state);
                constraint.hash(state);
            }
            TsValue::FunctionType { params, return_type } => {
                30.hash(state);
                params.len().hash(state);
                for (name, ty) in params {
                    name.hash(state);
                    ty.hash(state);
                }
                return_type.hash(state);
            }
            TsValue::ConstructorType { params, return_type } => {
                31.hash(state);
                params.len().hash(state);
                for (name, ty) in params {
                    name.hash(state);
                    ty.hash(state);
                }
                return_type.hash(state);
            }
            TsValue::ThisType => 32.hash(state),
            TsValue::Never => 33.hash(state),
            TsValue::Unknown => 34.hash(state),
            TsValue::Any => 35.hash(state),
            TsValue::Void => 36.hash(state),
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

    /// 检查是否为条件类型
    pub fn is_conditional(&self) -> bool {
        matches!(self, TsValue::Conditional(_))
    }

    /// 检查是否为映射类型
    pub fn is_mapped(&self) -> bool {
        matches!(self, TsValue::Mapped(_))
    }

    /// 检查是否为模板字面量类型
    pub fn is_template_literal(&self) -> bool {
        matches!(self, TsValue::TemplateLiteral(_))
    }

    /// 检查是否为 keyof 类型
    pub fn is_keyof(&self) -> bool {
        matches!(self, TsValue::KeyOf(_))
    }

    /// 检查是否为 typeof 类型
    pub fn is_typeof(&self) -> bool {
        matches!(self, TsValue::TypeOf(_))
    }

    /// 检查是否为索引访问类型
    pub fn is_indexed_access(&self) -> bool {
        matches!(self, TsValue::IndexedAccess { .. })
    }

    /// 检查是否为元组类型
    pub fn is_tuple(&self) -> bool {
        matches!(self, TsValue::Tuple(_))
    }

    /// 检查是否为只读类型
    pub fn is_readonly(&self) -> bool {
        matches!(self, TsValue::Readonly(_))
    }

    /// 检查是否为可空类型
    pub fn is_nullable(&self) -> bool {
        matches!(self, TsValue::Nullable(_))
    }

    /// 检查是否为不可空类型
    pub fn is_non_nullable(&self) -> bool {
        matches!(self, TsValue::NonNullable(_))
    }

    /// 检查是否为推断类型
    pub fn is_infer(&self) -> bool {
        matches!(self, TsValue::Infer { .. })
    }

    /// 检查是否为函数类型
    pub fn is_function_type(&self) -> bool {
        matches!(self, TsValue::FunctionType { .. })
    }

    /// 检查是否为构造函数类型
    pub fn is_constructor_type(&self) -> bool {
        matches!(self, TsValue::ConstructorType { .. })
    }

    /// 检查是否为 this 类型
    pub fn is_this_type(&self) -> bool {
        matches!(self, TsValue::ThisType)
    }

    /// 检查是否为 never 类型
    pub fn is_never(&self) -> bool {
        matches!(self, TsValue::Never)
    }

    /// 检查是否为 unknown 类型
    pub fn is_unknown(&self) -> bool {
        matches!(self, TsValue::Unknown)
    }

    /// 检查是否为 any 类型
    pub fn is_any(&self) -> bool {
        matches!(self, TsValue::Any)
    }

    /// 检查是否为 void 类型
    pub fn is_void(&self) -> bool {
        matches!(self, TsValue::Void)
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
            TsValue::Conditional(_) => true,
            TsValue::Mapped(_) => true,
            TsValue::TemplateLiteral(_) => true,
            TsValue::KeyOf(_) => true,
            TsValue::TypeOf(_) => true,
            TsValue::IndexedAccess { .. } => true,
            TsValue::Tuple(_) => true,
            TsValue::Readonly(_) => true,
            TsValue::Nullable(_) => true,
            TsValue::NonNullable(_) => true,
            TsValue::Infer { .. } => true,
            TsValue::FunctionType { .. } => true,
            TsValue::ConstructorType { .. } => true,
            TsValue::ThisType => true,
            TsValue::Never => false,
            TsValue::Unknown => true,
            TsValue::Any => true,
            TsValue::Void => false,
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
            TsValue::Conditional(_) => f64::NAN,
            TsValue::Mapped(_) => f64::NAN,
            TsValue::TemplateLiteral(_) => f64::NAN,
            TsValue::KeyOf(_) => f64::NAN,
            TsValue::TypeOf(_) => f64::NAN,
            TsValue::IndexedAccess { .. } => f64::NAN,
            TsValue::Tuple(_) => f64::NAN,
            TsValue::Readonly(_) => f64::NAN,
            TsValue::Nullable(_) => f64::NAN,
            TsValue::NonNullable(_) => f64::NAN,
            TsValue::Infer { .. } => f64::NAN,
            TsValue::FunctionType { .. } => f64::NAN,
            TsValue::ConstructorType { .. } => f64::NAN,
            TsValue::ThisType => f64::NAN,
            TsValue::Never => f64::NAN,
            TsValue::Unknown => f64::NAN,
            TsValue::Any => f64::NAN,
            TsValue::Void => f64::NAN,
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
                format!("{}<{}>", name, args_str.join(", "))
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
            TsValue::Conditional(cond) => {
                format!(
                    "{} extends {} ? {} : {}",
                    cond.check_type.to_string(),
                    cond.extends_type.to_string(),
                    cond.true_type.to_string(),
                    cond.false_type.to_string()
                )
            }
            TsValue::Mapped(mapped) => {
                let modifier = if mapped.key_modifier.readonly { "readonly " } else { "" };
                let optional = if mapped.key_modifier.optional { "?" } else { "" };
                format!(
                    "{{ {}[{} in {}]{}: {} }}",
                    modifier,
                    mapped.type_param,
                    mapped.constraint.to_string(),
                    optional,
                    mapped.value_type.to_string()
                )
            }
            TsValue::TemplateLiteral(tpl) => {
                let parts: Vec<String> = tpl
                    .parts
                    .iter()
                    .map(|p| match p {
                        TemplateLiteralPart::String(s) => s.clone(),
                        TemplateLiteralPart::Type(ty) => format!("${{{}}}", ty.to_string()),
                        TemplateLiteralPart::Number => "${number}".to_string(),
                        TemplateLiteralPart::StringType => "${string}".to_string(),
                        TemplateLiteralPart::BigInt => "${bigint}".to_string(),
                        TemplateLiteralPart::Union(types) => {
                            let types_str: Vec<String> = types.iter().map(|t| t.to_string()).collect();
                            format!("${{{}}}", types_str.join(" | "))
                        }
                    })
                    .collect();
                format!("`{}`", parts.join(""))
            }
            TsValue::KeyOf(ty) => format!("keyof {}", ty.to_string()),
            TsValue::TypeOf(ty) => format!("typeof {}", ty.to_string()),
            TsValue::IndexedAccess { object_type, index_type } => {
                format!("{}[{}]", object_type.to_string(), index_type.to_string())
            }
            TsValue::Tuple(elements) => {
                let elements_str: Vec<String> = elements.iter().map(|e| e.to_string()).collect();
                format!("[{}]", elements_str.join(", "))
            }
            TsValue::Readonly(ty) => format!("readonly {}", ty.to_string()),
            TsValue::Nullable(ty) => format!("{} | null", ty.to_string()),
            TsValue::NonNullable(ty) => format!("NonNullable<{}>", ty.to_string()),
            TsValue::Infer { type_param, .. } => format!("infer {}", type_param),
            TsValue::FunctionType { params, return_type } => {
                let params_str: Vec<String> = params.iter().map(|(name, ty)| format!("{}: {}", name, ty.to_string())).collect();
                format!("({}) => {}", params_str.join(", "), return_type.to_string())
            }
            TsValue::ConstructorType { params, return_type } => {
                let params_str: Vec<String> = params.iter().map(|(name, ty)| format!("{}: {}", name, ty.to_string())).collect();
                format!("new ({}) => {}", params_str.join(", "), return_type.to_string())
            }
            TsValue::ThisType => "this".to_string(),
            TsValue::Never => "never".to_string(),
            TsValue::Unknown => "unknown".to_string(),
            TsValue::Any => "any".to_string(),
            TsValue::Void => "void".to_string(),
        }
    }

    /// 检查类型是否可赋值给目标类型
    pub fn is_assignable_to(&self, target: &TsValue) -> bool {
        // 快速路径：基本类型直接比较
        match (self, target) {
            (TsValue::Any, _) | (_, TsValue::Any) => true,
            (TsValue::Unknown, _) => false,
            (_, TsValue::Unknown) => true,
            (TsValue::Never, _) => true,
            (_, TsValue::Never) => false,
            (TsValue::Void, TsValue::Void) => true,
            (TsValue::Void, _) | (_, TsValue::Void) => false,
            (TsValue::Null, TsValue::Null) => true,
            (TsValue::Undefined, TsValue::Undefined) => true,
            (TsValue::Boolean(_), TsValue::Boolean(_)) => true,
            (TsValue::Number(_), TsValue::Number(_)) => true,
            (TsValue::String(_), TsValue::String(_)) => true,
            (TsValue::BigInt(a), TsValue::BigInt(b)) => a == b,
            (TsValue::ThisType, TsValue::ThisType) => true,
            // 可空类型处理
            (TsValue::Nullable(_), TsValue::Null) => true,
            (TsValue::Nullable(inner), target) => inner.is_assignable_to(target),
            (source, TsValue::Nullable(inner)) => source.is_null() || source.is_assignable_to(inner),
            // 不可空类型处理
            (TsValue::NonNullable(inner), target) => !self.is_null() && inner.is_assignable_to(target),
            (source, TsValue::NonNullable(inner)) => !source.is_null() && source.is_assignable_to(inner),
            // 联合类型处理
            (TsValue::Union(types), target) => types.iter().all(|t| t.is_assignable_to(target)),
            (source, TsValue::Union(types)) => types.iter().any(|t| source.is_assignable_to(t)),
            // 条件类型处理
            (TsValue::Conditional(cond), target) => {
                cond.true_type.is_assignable_to(target) && cond.false_type.is_assignable_to(target)
            }
            (source, TsValue::Conditional(cond)) => {
                if source.is_assignable_to(&cond.extends_type) {
                    source.is_assignable_to(&cond.true_type)
                }
                else {
                    source.is_assignable_to(&cond.false_type)
                }
            }
            // 其他类型直接比较
            _ => self == target,
        }
    }

    /// 获取类型的所有属性键
    pub fn get_property_keys(&self) -> Vec<String> {
        match self {
            TsValue::Object(props) => props.keys().cloned().collect(),
            TsValue::Tuple(elements) => (0..elements.len()).map(|i| i.to_string()).collect(),
            TsValue::Array(_) => vec!["length".to_string(), "push".to_string(), "pop".to_string()],
            TsValue::String(_) => vec!["length".to_string(), "charAt".to_string(), "concat".to_string(), "indexOf".to_string()],
            TsValue::Mapped(mapped) => {
                if let TsValue::KeyOf(inner) = mapped.constraint.as_ref() {
                    inner.get_property_keys()
                }
                else {
                    Vec::new()
                }
            }
            TsValue::Nullable(inner) => inner.get_property_keys(),
            TsValue::NonNullable(inner) => inner.get_property_keys(),
            TsValue::Union(types) => {
                let mut keys = Vec::new();
                for ty in types {
                    for key in ty.get_property_keys() {
                        if !keys.contains(&key) {
                            keys.push(key);
                        }
                    }
                }
                keys
            }
            _ => Vec::new(),
        }
    }

    /// 获取指定属性的类型
    pub fn get_property_type(&self, key: &str) -> Option<TsValue> {
        match self {
            TsValue::Object(props) => props.get(key).cloned(),
            TsValue::Tuple(elements) => {
                if let Ok(index) = key.parse::<usize>() {
                    elements.get(index).cloned()
                }
                else {
                    None
                }
            }
            TsValue::Array(_) => match key {
                "length" => Some(TsValue::Number(0.0)),
                _ => None,
            },
            TsValue::String(_) => match key {
                "length" => Some(TsValue::Number(0.0)),
                _ => None,
            },
            TsValue::IndexedAccess { object_type, index_type } => {
                if let TsValue::String(key_str) = index_type.as_ref() {
                    object_type.get_property_type(key_str)
                }
                else {
                    None
                }
            }
            TsValue::Nullable(inner) => inner.get_property_type(key),
            TsValue::NonNullable(inner) => inner.get_property_type(key),
            TsValue::Union(types) => {
                let results: Vec<TsValue> = types.iter().filter_map(|t| t.get_property_type(key)).collect();
                if results.is_empty() {
                    None
                }
                else if results.len() == 1 {
                    results.into_iter().next()
                }
                else {
                    Some(TsValue::Union(results))
                }
            }
            _ => None,
        }
    }

    /// 评估条件类型
    pub fn evaluate_conditional(&self, check_type: &TsValue, extends_type: &TsValue) -> Option<bool> {
        match self {
            TsValue::Conditional(cond) => Some(check_type.is_assignable_to(&cond.extends_type)),
            _ => match extends_type {
                TsValue::Any => Some(true),
                TsValue::Unknown => Some(true),
                TsValue::Never => Some(false),
                TsValue::Union(types) => {
                    let results: Vec<bool> = types.iter().filter_map(|t| self.evaluate_conditional(check_type, t)).collect();
                    Some(results.iter().any(|&r| r))
                }
                _ => Some(check_type.is_assignable_to(extends_type)),
            },
        }
    }

    /// 推断类型参数
    pub fn infer_type_params(&self, target: &TsValue) -> InferenceResult {
        // 快速路径：基本类型直接比较
        match (self, target) {
            (TsValue::Boolean(_), TsValue::Boolean(_))
            | (TsValue::Number(_), TsValue::Number(_))
            | (TsValue::String(_), TsValue::String(_)) => {
                return InferenceResult::success(HashMap::new());
            }
            _ if self == target => {
                return InferenceResult::success(HashMap::new());
            }
            _ => {}
        }

        match (self, target) {
            // 类型推断模式
            (TsValue::Infer { type_param, constraint }, target) => {
                if let Some(constraint_ty) = constraint {
                    if !target.is_assignable_to(constraint_ty) {
                        return InferenceResult::failure();
                    }
                }
                let mut inferred = HashMap::new();
                inferred.insert(type_param.clone(), target.clone());
                InferenceResult::success(inferred)
            }
            // 泛型类型
            (TsValue::Generic(name1, args1), TsValue::Generic(name2, args2)) => {
                if name1 != name2 || args1.len() != args2.len() {
                    return InferenceResult::failure();
                }
                let mut result = InferenceResult::success(HashMap::new());
                for (a1, a2) in args1.iter().zip(args2.iter()) {
                    let sub_result = a1.infer_type_params(a2);
                    result = result.merge(sub_result);
                }
                result
            }
            // 联合类型
            (TsValue::Union(types1), TsValue::Union(types2)) => {
                if types1.len() != types2.len() {
                    return InferenceResult::failure();
                }
                let mut result = InferenceResult::success(HashMap::new());
                for (t1, t2) in types1.iter().zip(types2.iter()) {
                    let sub_result = t1.infer_type_params(t2);
                    result = result.merge(sub_result);
                }
                result
            }
            // 元组类型
            (TsValue::Tuple(elements1), TsValue::Tuple(elements2)) => {
                if elements1.len() != elements2.len() {
                    return InferenceResult::failure();
                }
                let mut result = InferenceResult::success(HashMap::new());
                for (e1, e2) in elements1.iter().zip(elements2.iter()) {
                    let sub_result = e1.infer_type_params(e2);
                    result = result.merge(sub_result);
                }
                result
            }
            // 对象类型
            (TsValue::Object(props1), TsValue::Object(props2)) => {
                let mut result = InferenceResult::success(HashMap::new());
                for (key, value1) in props1 {
                    if let Some(value2) = props2.get(key) {
                        let sub_result = value1.infer_type_params(value2);
                        result = result.merge(sub_result);
                    }
                }
                result
            }
            // 函数类型
            (
                TsValue::FunctionType { params: params1, return_type: ret1 },
                TsValue::FunctionType { params: params2, return_type: ret2 },
            ) => {
                if params1.len() != params2.len() {
                    return InferenceResult::failure();
                }
                let mut result = ret1.infer_type_params(ret2);
                for ((_, ty1), (_, ty2)) in params1.iter().zip(params2.iter()) {
                    let sub_result = ty1.infer_type_params(ty2);
                    result = result.merge(sub_result);
                }
                result
            }
            // 包装类型
            (TsValue::Nullable(inner1), TsValue::Nullable(inner2)) => inner1.infer_type_params(inner2),
            (TsValue::NonNullable(inner1), TsValue::NonNullable(inner2)) => inner1.infer_type_params(inner2),
            (TsValue::Readonly(inner1), TsValue::Readonly(inner2)) => inner1.infer_type_params(inner2),
            (TsValue::Promise(inner1), TsValue::Promise(inner2)) => inner1.infer_type_params(inner2),
            // 数组类型
            (TsValue::Array(elements1), TsValue::Array(elements2)) => {
                if elements1.len() != elements2.len() {
                    return InferenceResult::failure();
                }
                let mut result = InferenceResult::success(HashMap::new());
                for (e1, e2) in elements1.iter().zip(elements2.iter()) {
                    let sub_result = e1.infer_type_params(e2);
                    result = result.merge(sub_result);
                }
                result
            }
            // 其他情况
            _ => InferenceResult::failure(),
        }
    }

    /// 替换类型参数
    pub fn substitute_type_params(&self, substitutions: &HashMap<String, TsValue>) -> TsValue {
        match self {
            TsValue::Generic(name, args) => {
                if let Some(substituted) = substitutions.get(name) {
                    substituted.clone()
                }
                else {
                    TsValue::Generic(name.clone(), args.iter().map(|a| a.substitute_type_params(substitutions)).collect())
                }
            }
            TsValue::Union(types) => TsValue::Union(types.iter().map(|t| t.substitute_type_params(substitutions)).collect()),
            TsValue::Tuple(elements) => {
                TsValue::Tuple(elements.iter().map(|e| e.substitute_type_params(substitutions)).collect())
            }
            TsValue::Array(elements) => {
                TsValue::Array(elements.iter().map(|e| e.substitute_type_params(substitutions)).collect())
            }
            TsValue::Object(props) => {
                TsValue::Object(props.iter().map(|(k, v)| (k.clone(), v.substitute_type_params(substitutions))).collect())
            }
            TsValue::Nullable(inner) => TsValue::Nullable(Box::new(inner.substitute_type_params(substitutions))),
            TsValue::NonNullable(inner) => TsValue::NonNullable(Box::new(inner.substitute_type_params(substitutions))),
            TsValue::Readonly(inner) => TsValue::Readonly(Box::new(inner.substitute_type_params(substitutions))),
            TsValue::Promise(inner) => TsValue::Promise(Box::new(inner.substitute_type_params(substitutions))),
            TsValue::KeyOf(inner) => TsValue::KeyOf(Box::new(inner.substitute_type_params(substitutions))),
            TsValue::TypeOf(inner) => TsValue::TypeOf(Box::new(inner.substitute_type_params(substitutions))),
            TsValue::Conditional(cond) => TsValue::Conditional(Conditional {
                check_type: Box::new(cond.check_type.substitute_type_params(substitutions)),
                extends_type: Box::new(cond.extends_type.substitute_type_params(substitutions)),
                true_type: Box::new(cond.true_type.substitute_type_params(substitutions)),
                false_type: Box::new(cond.false_type.substitute_type_params(substitutions)),
                modifier: cond.modifier.clone(),
            }),
            TsValue::Mapped(mapped) => TsValue::Mapped(Mapped {
                type_param: mapped.type_param.clone(),
                constraint: Box::new(mapped.constraint.substitute_type_params(substitutions)),
                value_type: Box::new(mapped.value_type.substitute_type_params(substitutions)),
                key_modifier: mapped.key_modifier.clone(),
                constraint_type: mapped.constraint_type.clone(),
            }),
            TsValue::TemplateLiteral(tpl) => TsValue::TemplateLiteral(TemplateLiteral {
                parts: tpl
                    .parts
                    .iter()
                    .map(|p| match p {
                        TemplateLiteralPart::String(s) => TemplateLiteralPart::String(s.clone()),
                        TemplateLiteralPart::Type(ty) => {
                            TemplateLiteralPart::Type(Box::new(ty.substitute_type_params(substitutions)))
                        }
                        TemplateLiteralPart::Number => TemplateLiteralPart::Number,
                        TemplateLiteralPart::StringType => TemplateLiteralPart::StringType,
                        TemplateLiteralPart::BigInt => TemplateLiteralPart::BigInt,
                        TemplateLiteralPart::Union(types) => {
                            TemplateLiteralPart::Union(types.iter().map(|t| t.substitute_type_params(substitutions)).collect())
                        }
                    })
                    .collect(),
            }),
            TsValue::IndexedAccess { object_type, index_type } => TsValue::IndexedAccess {
                object_type: Box::new(object_type.substitute_type_params(substitutions)),
                index_type: Box::new(index_type.substitute_type_params(substitutions)),
            },
            TsValue::FunctionType { params, return_type } => TsValue::FunctionType {
                params: params.iter().map(|(name, ty)| (name.clone(), ty.substitute_type_params(substitutions))).collect(),
                return_type: Box::new(return_type.substitute_type_params(substitutions)),
            },
            TsValue::ConstructorType { params, return_type } => TsValue::ConstructorType {
                params: params.iter().map(|(name, ty)| (name.clone(), ty.substitute_type_params(substitutions))).collect(),
                return_type: Box::new(return_type.substitute_type_params(substitutions)),
            },
            TsValue::Infer { type_param, constraint } => TsValue::Infer {
                type_param: type_param.clone(),
                constraint: constraint.as_ref().map(|c| Box::new(c.substitute_type_params(substitutions))),
            },
            _ => self.clone(),
        }
    }

    /// 计算两个类型的交集
    ///
    /// # 参数
    /// - `other`: 另一个类型
    ///
    /// # 返回
    /// 两个类型的交集
    pub fn intersection_with(&self, other: &TsValue) -> TsValue {
        match (self, other) {
            // 基本类型交集
            (TsValue::Any, _) | (_, TsValue::Any) => other.clone(),
            (TsValue::Unknown, _) | (_, TsValue::Unknown) => TsValue::Unknown,
            (TsValue::Never, _) | (_, TsValue::Never) => TsValue::Never,
            (TsValue::Void, _) | (_, TsValue::Void) => TsValue::Void,
            (a, b) if a == b => a.clone(),

            // 联合类型交集
            (TsValue::Union(types1), TsValue::Union(types2)) => {
                let mut result = Vec::new();
                for t1 in types1 {
                    for t2 in types2 {
                        let intersection = t1.intersection_with(t2);
                        if !intersection.is_never() {
                            result.push(intersection);
                        }
                    }
                }
                if result.is_empty() {
                    TsValue::Never
                }
                else if result.len() == 1 {
                    result[0].clone()
                }
                else {
                    TsValue::Union(result)
                }
            }
            (TsValue::Union(types), other) => {
                let mut result = Vec::new();
                for t in types {
                    let intersection = t.intersection_with(other);
                    if !intersection.is_never() {
                        result.push(intersection);
                    }
                }
                if result.is_empty() {
                    TsValue::Never
                }
                else if result.len() == 1 {
                    result[0].clone()
                }
                else {
                    TsValue::Union(result)
                }
            }
            (other, TsValue::Union(types)) => {
                let mut result = Vec::new();
                for t in types {
                    let intersection = other.intersection_with(t);
                    if !intersection.is_never() {
                        result.push(intersection);
                    }
                }
                if result.is_empty() {
                    TsValue::Never
                }
                else if result.len() == 1 {
                    result[0].clone()
                }
                else {
                    TsValue::Union(result)
                }
            }

            // 对象类型交集
            (TsValue::Object(props1), TsValue::Object(props2)) => {
                let mut result = HashMap::new();
                for (key, value1) in props1 {
                    if let Some(value2) = props2.get(key) {
                        let intersection = value1.intersection_with(value2);
                        if !intersection.is_never() {
                            result.insert(key.clone(), intersection);
                        }
                    }
                }
                TsValue::Object(result)
            }

            // 其他情况返回 never
            _ => TsValue::Never,
        }
    }

    /// 计算两个类型的差集
    ///
    /// # 参数
    /// - `other`: 要减去的类型
    ///
    /// # 返回
    /// 类型差集
    pub fn difference_with(&self, other: &TsValue) -> TsValue {
        match (self, other) {
            // 基本类型差集
            (TsValue::Any, TsValue::Any) => TsValue::Never,
            (TsValue::Any, _) => TsValue::Any,
            (_, TsValue::Any) => TsValue::Never,
            (TsValue::Unknown, _) => TsValue::Unknown,
            (_, TsValue::Unknown) => TsValue::Never,
            (TsValue::Never, _) => TsValue::Never,
            (_, TsValue::Never) => self.clone(),
            (TsValue::Void, _) => TsValue::Void,
            (_, TsValue::Void) => self.clone(),
            (a, b) if a == b => TsValue::Never,

            // 联合类型差集
            (TsValue::Union(types), other) => {
                let mut result = Vec::new();
                for t in types {
                    let difference = t.difference_with(other);
                    if !difference.is_never() {
                        result.push(difference);
                    }
                }
                if result.is_empty() {
                    TsValue::Never
                }
                else if result.len() == 1 {
                    result[0].clone()
                }
                else {
                    TsValue::Union(result)
                }
            }

            // 其他情况返回原类型
            _ => self.clone(),
        }
    }

    /// 应用映射类型到对象
    ///
    /// # 参数
    /// - `mapped_type`: 映射类型
    ///
    /// # 返回
    /// 应用映射后的类型
    pub fn apply_mapped_type(&self, mapped_type: &Mapped) -> TsValue {
        match self {
            TsValue::Object(props) => {
                let mut result = HashMap::new();
                for (key, _value) in props {
                    // 替换类型参数
                    let mut substitutions = HashMap::new();
                    substitutions.insert(mapped_type.type_param.clone(), TsValue::String(key.clone()));
                    let mapped_value = mapped_type.value_type.substitute_type_params(&substitutions);
                    result.insert(key.clone(), mapped_value);
                }
                TsValue::Object(result)
            }
            _ => TsValue::Never,
        }
    }

    /// 检查类型是否为原始类型
    ///
    /// # 返回
    /// 如果是原始类型返回 true，否则返回 false
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            TsValue::Undefined
                | TsValue::Null
                | TsValue::Boolean(_)
                | TsValue::Number(_)
                | TsValue::String(_)
                | TsValue::Symbol(_)
                | TsValue::BigInt(_)
        )
    }

    /// 检查类型是否为复合类型
    ///
    /// # 返回
    /// 如果是复合类型返回 true，否则返回 false
    pub fn is_complex(&self) -> bool {
        matches!(
            self,
            TsValue::Object(_)
                | TsValue::Array(_)
                | TsValue::Function(_)
                | TsValue::Union(_)
                | TsValue::Tuple(_)
                | TsValue::Map(_)
                | TsValue::Set(_)
                | TsValue::Promise(_)
        )
    }

    /// 获取类型的字符串表示
    ///
    /// # 返回
    /// 类型的字符串表示
    pub fn type_name(&self) -> String {
        match self {
            TsValue::Undefined => "undefined".to_string(),
            TsValue::Null => "null".to_string(),
            TsValue::Boolean(_) => "boolean".to_string(),
            TsValue::Number(_) => "number".to_string(),
            TsValue::String(_) => "string".to_string(),
            TsValue::Object(_) => "object".to_string(),
            TsValue::Array(_) => "array".to_string(),
            TsValue::Function(_) => "function".to_string(),
            TsValue::Error(_) => "error".to_string(),
            TsValue::Union(_) => "union".to_string(),
            TsValue::Generic(name, _) => name.clone(),
            TsValue::Symbol(_) => "symbol".to_string(),
            TsValue::BigInt(_) => "bigint".to_string(),
            TsValue::Date(_) => "date".to_string(),
            TsValue::RegExp(_) => "regexp".to_string(),
            TsValue::Map(_) => "map".to_string(),
            TsValue::Set(_) => "set".to_string(),
            TsValue::Promise(_) => "promise".to_string(),
            TsValue::Iterable(_) => "iterable".to_string(),
            TsValue::Conditional(_) => "conditional".to_string(),
            TsValue::Mapped(_) => "mapped".to_string(),
            TsValue::TemplateLiteral(_) => "template_literal".to_string(),
            TsValue::KeyOf(_) => "keyof".to_string(),
            TsValue::TypeOf(_) => "typeof".to_string(),
            TsValue::IndexedAccess { .. } => "indexed_access".to_string(),
            TsValue::Tuple(_) => "tuple".to_string(),
            TsValue::Readonly(_) => "readonly".to_string(),
            TsValue::Nullable(_) => "nullable".to_string(),
            TsValue::NonNullable(_) => "non_nullable".to_string(),
            TsValue::Infer { type_param, .. } => format!("infer {}", type_param),
            TsValue::FunctionType { .. } => "function_type".to_string(),
            TsValue::ConstructorType { .. } => "constructor_type".to_string(),
            TsValue::ThisType => "this".to_string(),
            TsValue::Never => "never".to_string(),
            TsValue::Unknown => "unknown".to_string(),
            TsValue::Any => "any".to_string(),
            TsValue::Void => "void".to_string(),
        }
    }

    /// 简化类型
    ///
    /// # 返回
    /// 简化后的类型
    pub fn simplify(&self) -> TsValue {
        match self {
            TsValue::Union(types) => {
                // 过滤 never 类型
                let filtered: Vec<TsValue> = types.iter().map(|t| t.simplify()).filter(|t| !t.is_never()).collect();

                // 去重
                let mut unique = Vec::new();
                for t in filtered {
                    if !unique.iter().any(|u| u == &t) {
                        unique.push(t);
                    }
                }

                if unique.is_empty() {
                    TsValue::Never
                }
                else if unique.len() == 1 {
                    unique[0].clone()
                }
                else {
                    TsValue::Union(unique)
                }
            }
            TsValue::Nullable(inner) => {
                let simplified = inner.simplify();
                if simplified.is_never() { TsValue::Never } else { TsValue::Nullable(Box::new(simplified)) }
            }
            TsValue::NonNullable(inner) => {
                let simplified = inner.simplify();
                if simplified.is_never() { TsValue::Never } else { TsValue::NonNullable(Box::new(simplified)) }
            }
            TsValue::Readonly(inner) => {
                let simplified = inner.simplify();
                TsValue::Readonly(Box::new(simplified))
            }
            TsValue::Promise(inner) => {
                let simplified = inner.simplify();
                TsValue::Promise(Box::new(simplified))
            }
            TsValue::Array(elements) => {
                let simplified: Vec<TsValue> = elements.iter().map(|e| e.simplify()).collect();
                TsValue::Array(simplified)
            }
            TsValue::Object(props) => {
                let simplified: HashMap<String, TsValue> = props.iter().map(|(k, v)| (k.clone(), v.simplify())).collect();
                TsValue::Object(simplified)
            }
            TsValue::Tuple(elements) => {
                let simplified: Vec<TsValue> = elements.iter().map(|e| e.simplify()).collect();
                TsValue::Tuple(simplified)
            }
            TsValue::Conditional(cond) => TsValue::Conditional(Conditional {
                check_type: Box::new(cond.check_type.simplify()),
                extends_type: Box::new(cond.extends_type.simplify()),
                true_type: Box::new(cond.true_type.simplify()),
                false_type: Box::new(cond.false_type.simplify()),
                modifier: cond.modifier.clone(),
            }),
            TsValue::Mapped(mapped) => TsValue::Mapped(Mapped {
                type_param: mapped.type_param.clone(),
                constraint: Box::new(mapped.constraint.simplify()),
                value_type: Box::new(mapped.value_type.simplify()),
                key_modifier: mapped.key_modifier.clone(),
                constraint_type: mapped.constraint_type.clone(),
            }),
            TsValue::TemplateLiteral(tpl) => TsValue::TemplateLiteral(TemplateLiteral {
                parts: tpl
                    .parts
                    .iter()
                    .map(|p| match p {
                        TemplateLiteralPart::String(s) => TemplateLiteralPart::String(s.clone()),
                        TemplateLiteralPart::Type(ty) => TemplateLiteralPart::Type(Box::new(ty.simplify())),
                        TemplateLiteralPart::Number => TemplateLiteralPart::Number,
                        TemplateLiteralPart::StringType => TemplateLiteralPart::StringType,
                        TemplateLiteralPart::BigInt => TemplateLiteralPart::BigInt,
                        TemplateLiteralPart::Union(types) => {
                            TemplateLiteralPart::Union(types.iter().map(|t| t.simplify()).collect())
                        }
                    })
                    .collect(),
            }),
            TsValue::IndexedAccess { object_type, index_type } => TsValue::IndexedAccess {
                object_type: Box::new(object_type.simplify()),
                index_type: Box::new(index_type.simplify()),
            },
            TsValue::FunctionType { params, return_type } => TsValue::FunctionType {
                params: params.iter().map(|(name, ty)| (name.clone(), ty.simplify())).collect(),
                return_type: Box::new(return_type.simplify()),
            },
            TsValue::ConstructorType { params, return_type } => TsValue::ConstructorType {
                params: params.iter().map(|(name, ty)| (name.clone(), ty.simplify())).collect(),
                return_type: Box::new(return_type.simplify()),
            },
            _ => self.clone(),
        }
    }

    /// 计算类型交集
    pub fn intersect(&self, other: &TsValue) -> TsValue {
        match (self, other) {
            (TsValue::Any, _) | (_, TsValue::Any) => TsValue::Any,
            (TsValue::Never, _) | (_, TsValue::Never) => TsValue::Never,
            (TsValue::Object(props1), TsValue::Object(props2)) => {
                let mut merged = props1.clone();
                for (key, value) in props2 {
                    merged.insert(key.clone(), value.clone());
                }
                TsValue::Object(merged)
            }
            (TsValue::Union(types1), TsValue::Union(types2)) => {
                let mut result = Vec::new();
                for t1 in types1 {
                    for t2 in types2 {
                        result.push(t1.intersect(t2));
                    }
                }
                TsValue::Union(result)
            }
            (TsValue::Union(types), other) => {
                let result: Vec<TsValue> = types.iter().map(|t| t.intersect(other)).collect();
                TsValue::Union(result)
            }
            (other, TsValue::Union(types)) => {
                let result: Vec<TsValue> = types.iter().map(|t| other.intersect(t)).collect();
                TsValue::Union(result)
            }
            _ => {
                if self.is_assignable_to(other) {
                    self.clone()
                }
                else if other.is_assignable_to(self) {
                    other.clone()
                }
                else {
                    TsValue::Never
                }
            }
        }
    }

    /// 计算类型差集
    pub fn difference(&self, other: &TsValue) -> TsValue {
        match (self, other) {
            (TsValue::Any, TsValue::Any) => TsValue::Never,
            (TsValue::Any, _) => TsValue::Any,
            (_, TsValue::Any) => TsValue::Never,
            (TsValue::Never, _) => TsValue::Never,
            (_, TsValue::Never) => self.clone(),
            (TsValue::Object(props1), TsValue::Object(props2)) => {
                let mut result = props1.clone();
                for key in props2.keys() {
                    result.remove(key);
                }
                TsValue::Object(result)
            }
            (TsValue::Union(types), other) => {
                let result: Vec<TsValue> =
                    types.iter().map(|t| t.difference(other)).filter(|t| !matches!(t, TsValue::Never)).collect();
                if result.is_empty() {
                    TsValue::Never
                }
                else if result.len() == 1 {
                    result[0].clone()
                }
                else {
                    TsValue::Union(result)
                }
            }
            _ => {
                if self.is_assignable_to(other) {
                    TsValue::Never
                }
                else {
                    self.clone()
                }
            }
        }
    }

    /// 检查类型是否为字面量类型
    pub fn is_literal(&self) -> bool {
        match self {
            TsValue::Boolean(_) | TsValue::Number(_) | TsValue::String(_) | TsValue::BigInt(_) => true,
            _ => false,
        }
    }

    /// 获取类型的基础类型
    pub fn get_base_type(&self) -> TsValue {
        match self {
            TsValue::Nullable(inner) | TsValue::NonNullable(inner) | TsValue::Readonly(inner) => inner.get_base_type(),
            TsValue::Promise(inner) => TsValue::Promise(Box::new(inner.get_base_type())),
            TsValue::Array(elements) => {
                if elements.is_empty() {
                    TsValue::Array(Vec::new())
                }
                else {
                    TsValue::Array(vec![elements[0].get_base_type()])
                }
            }
            _ => self.clone(),
        }
    }

    /// 检查类型是否为联合类型的一部分
    pub fn is_part_of_union(&self, union: &TsValue) -> bool {
        match union {
            TsValue::Union(types) => types.iter().any(|t| self.is_assignable_to(t) || t.is_assignable_to(self)),
            _ => self.is_assignable_to(union) || union.is_assignable_to(self),
        }
    }

    /// 解析 keyof 类型
    pub fn resolve_keyof(&self) -> TsValue {
        match self {
            TsValue::Object(props) => {
                let keys: Vec<TsValue> = props.keys().map(|k| TsValue::String(k.clone())).collect();
                if keys.is_empty() {
                    TsValue::Never
                }
                else if keys.len() == 1 {
                    keys[0].clone()
                }
                else {
                    TsValue::Union(keys)
                }
            }
            TsValue::Array(_) => TsValue::Union(vec![
                TsValue::String("length".to_string()),
                TsValue::String("push".to_string()),
                TsValue::String("pop".to_string()),
                TsValue::String("shift".to_string()),
                TsValue::String("unshift".to_string()),
                TsValue::String("slice".to_string()),
                TsValue::String("splice".to_string()),
                TsValue::String("forEach".to_string()),
                TsValue::String("map".to_string()),
                TsValue::String("filter".to_string()),
                TsValue::String("reduce".to_string()),
            ]),
            TsValue::Tuple(elements) => {
                let mut keys = Vec::new();
                for (i, _) in elements.iter().enumerate() {
                    keys.push(TsValue::String(i.to_string()));
                }
                keys.push(TsValue::String("length".to_string()));
                TsValue::Union(keys)
            }
            TsValue::Union(types) => {
                let mut all_keys = Vec::new();
                for ty in types {
                    match ty.resolve_keyof() {
                        TsValue::Union(keys) => all_keys.extend(keys),
                        key => all_keys.push(key),
                    }
                }
                if all_keys.is_empty() {
                    TsValue::Never
                }
                else if all_keys.len() == 1 {
                    all_keys[0].clone()
                }
                else {
                    TsValue::Union(all_keys)
                }
            }
            TsValue::Nullable(inner) => inner.resolve_keyof(),
            TsValue::NonNullable(inner) => inner.resolve_keyof(),
            TsValue::Readonly(inner) => inner.resolve_keyof(),
            TsValue::Promise(_inner) => TsValue::Union(vec![
                TsValue::String("then".to_string()),
                TsValue::String("catch".to_string()),
                TsValue::String("finally".to_string()),
            ]),
            TsValue::KeyOf(inner) => inner.resolve_keyof(),
            _ => TsValue::Never,
        }
    }

    /// 解析索引访问类型
    pub fn resolve_indexed_access(&self) -> TsValue {
        match self {
            TsValue::IndexedAccess { object_type, index_type } => {
                let obj = object_type.resolve_indexed_access();
                let idx = index_type.resolve_indexed_access();
                match &idx {
                    TsValue::String(key) => obj.get_property_type(key).unwrap_or(TsValue::Never),
                    TsValue::Union(keys) => {
                        let results: Vec<TsValue> = keys
                            .iter()
                            .filter_map(|k| if let TsValue::String(key) = k { obj.get_property_type(key) } else { None })
                            .collect();
                        if results.is_empty() {
                            TsValue::Never
                        }
                        else if results.len() == 1 {
                            results.into_iter().next().unwrap()
                        }
                        else {
                            TsValue::Union(results)
                        }
                    }
                    TsValue::KeyOf(inner) => {
                        let keys = inner.resolve_keyof();
                        TsValue::IndexedAccess { object_type: Box::new(obj), index_type: Box::new(keys) }
                            .resolve_indexed_access()
                    }
                    _ => TsValue::Never,
                }
            }
            TsValue::Union(types) => {
                let resolved: Vec<TsValue> = types.iter().map(|t| t.resolve_indexed_access()).collect();
                TsValue::Union(resolved)
            }
            TsValue::Tuple(elements) => {
                let resolved: Vec<TsValue> = elements.iter().map(|e| e.resolve_indexed_access()).collect();
                TsValue::Tuple(resolved)
            }
            TsValue::Array(elements) => {
                let resolved: Vec<TsValue> = elements.iter().map(|e| e.resolve_indexed_access()).collect();
                TsValue::Array(resolved)
            }
            TsValue::Object(props) => {
                let resolved: HashMap<String, TsValue> =
                    props.iter().map(|(k, v)| (k.clone(), v.resolve_indexed_access())).collect();
                TsValue::Object(resolved)
            }
            TsValue::Nullable(inner) => TsValue::Nullable(Box::new(inner.resolve_indexed_access())),
            TsValue::NonNullable(inner) => TsValue::NonNullable(Box::new(inner.resolve_indexed_access())),
            TsValue::Readonly(inner) => TsValue::Readonly(Box::new(inner.resolve_indexed_access())),
            TsValue::Promise(inner) => TsValue::Promise(Box::new(inner.resolve_indexed_access())),
            other => other.clone(),
        }
    }

    /// 解析映射类型
    pub fn resolve_mapped(&self) -> TsValue {
        match self {
            TsValue::Mapped(mapped) => {
                let keys_type = mapped.constraint.resolve_keyof();
                let keys = match keys_type {
                    TsValue::Union(types) => types,
                    TsValue::String(s) => vec![TsValue::String(s)],
                    TsValue::Never => return TsValue::Object(HashMap::new()),
                    _ => return self.clone(),
                };
                let mut props = HashMap::new();
                for key in keys {
                    if let TsValue::String(key_str) = key {
                        let mut substitutions = HashMap::new();
                        substitutions.insert(mapped.type_param.clone(), TsValue::String(key_str.clone()));
                        let value_type = mapped.value_type.substitute_type_params(&substitutions);
                        props.insert(key_str, value_type);
                    }
                }
                TsValue::Object(props)
            }
            TsValue::Union(types) => {
                let resolved: Vec<TsValue> = types.iter().map(|t| t.resolve_mapped()).collect();
                TsValue::Union(resolved)
            }
            TsValue::Tuple(elements) => {
                let resolved: Vec<TsValue> = elements.iter().map(|e| e.resolve_mapped()).collect();
                TsValue::Tuple(resolved)
            }
            TsValue::Array(elements) => {
                let resolved: Vec<TsValue> = elements.iter().map(|e| e.resolve_mapped()).collect();
                TsValue::Array(resolved)
            }
            TsValue::Object(props) => {
                let resolved: HashMap<String, TsValue> = props.iter().map(|(k, v)| (k.clone(), v.resolve_mapped())).collect();
                TsValue::Object(resolved)
            }
            TsValue::Nullable(inner) => TsValue::Nullable(Box::new(inner.resolve_mapped())),
            TsValue::NonNullable(inner) => TsValue::NonNullable(Box::new(inner.resolve_mapped())),
            TsValue::Readonly(inner) => TsValue::Readonly(Box::new(inner.resolve_mapped())),
            TsValue::Promise(inner) => TsValue::Promise(Box::new(inner.resolve_mapped())),
            other => other.clone(),
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
        let mut map = HashMap::new();
        for (k, v) in self {
            map.insert(k.to_string(), v.to_ts_value());
        }
        TsValue::Object(map)
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
        TsValue::Object(self.clone())
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

unsafe impl Send for TsValue {}

unsafe impl Sync for TsValue {}


