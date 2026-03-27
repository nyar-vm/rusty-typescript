# TypeScript 类型系统 API 文档

## 概述

TypeScript 类型系统是 Rusty TypeScript 项目的核心组件之一，提供了完整的 TypeScript 类型表示和操作功能。本文档详细介绍了类型系统的 API 接口，帮助开发者理解和使用类型系统的各项功能。

## 核心类型

### TsValue 枚举

`TsValue` 是类型系统的核心类型，表示 TypeScript 中的所有可能值和类型。

```rust
pub enum TsValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Object(HashMap<String, TsValue>),
    Array(Vec<TsValue>),
    Function(Rc<dyn Fn(&[TsValue]) -> TsValue>),
    Error(String),
    Union(Vec<TsValue>),
    Generic(String, Vec<TsValue>),
    Symbol(String),
    BigInt(i128),
    Date(i64),
    RegExp(String),
    Map(Vec<(TsValue, TsValue)>),
    Set(Vec<TsValue>),
    Promise(Box<TsValue>),
    Iterable(Box<dyn Iterator<Item = TsValue>>),
    Conditional(Conditional),
    Mapped(Mapped),
    TemplateLiteral(TemplateLiteral),
    KeyOf(Box<TsValue>),
    TypeOf(Box<TsValue>),
    IndexedAccess { object_type: Box<TsValue>, index_type: Box<TsValue> },
    Tuple(Vec<TsValue>),
    Readonly(Box<TsValue>),
    Nullable(Box<TsValue>),
    NonNullable(Box<TsValue>),
    Infer { type_param: String, constraint: Option<Box<TsValue>> },
    FunctionType { params: Vec<(String, TsValue)>, return_type: Box<TsValue> },
    ConstructorType { params: Vec<(String, TsValue)>, return_type: Box<TsValue> },
    ThisType,
    Never,
    Unknown,
    Any,
    Void,
}
```

## 类型操作 API

### 类型检查方法

| 方法名 | 描述 | 返回类型 |
|-------|------|---------|
| `is_undefined()` | 检查是否为 undefined 类型 | `bool` |
| `is_null()` | 检查是否为 null 类型 | `bool` |
| `is_boolean()` | 检查是否为 boolean 类型 | `bool` |
| `is_number()` | 检查是否为 number 类型 | `bool` |
| `is_string()` | 检查是否为 string 类型 | `bool` |
| `is_object()` | 检查是否为 object 类型 | `bool` |
| `is_array()` | 检查是否为 array 类型 | `bool` |
| `is_function()` | 检查是否为 function 类型 | `bool` |
| `is_error()` | 检查是否为 error 类型 | `bool` |
| `is_union()` | 检查是否为 union 类型 | `bool` |
| `is_generic()` | 检查是否为 generic 类型 | `bool` |
| `is_symbol()` | 检查是否为 symbol 类型 | `bool` |
| `is_bigint()` | 检查是否为 bigint 类型 | `bool` |
| `is_date()` | 检查是否为 date 类型 | `bool` |
| `is_regexp()` | 检查是否为 regexp 类型 | `bool` |
| `is_map()` | 检查是否为 map 类型 | `bool` |
| `is_set()` | 检查是否为 set 类型 | `bool` |
| `is_promise()` | 检查是否为 promise 类型 | `bool` |
| `is_iterable()` | 检查是否为 iterable 类型 | `bool` |
| `is_conditional()` | 检查是否为 conditional 类型 | `bool` |
| `is_mapped()` | 检查是否为 mapped 类型 | `bool` |
| `is_template_literal()` | 检查是否为 template literal 类型 | `bool` |
| `is_keyof()` | 检查是否为 keyof 类型 | `bool` |
| `is_typeof()` | 检查是否为 typeof 类型 | `bool` |
| `is_indexed_access()` | 检查是否为 indexed access 类型 | `bool` |
| `is_tuple()` | 检查是否为 tuple 类型 | `bool` |
| `is_readonly()` | 检查是否为 readonly 类型 | `bool` |
| `is_nullable()` | 检查是否为 nullable 类型 | `bool` |
| `is_non_nullable()` | 检查是否为 non-nullable 类型 | `bool` |
| `is_infer()` | 检查是否为 infer 类型 | `bool` |
| `is_function_type()` | 检查是否为 function type 类型 | `bool` |
| `is_constructor_type()` | 检查是否为 constructor type 类型 | `bool` |
| `is_this_type()` | 检查是否为 this 类型 | `bool` |
| `is_never()` | 检查是否为 never 类型 | `bool` |
| `is_unknown()` | 检查是否为 unknown 类型 | `bool` |
| `is_any()` | 检查是否为 any 类型 | `bool` |
| `is_void()` | 检查是否为 void 类型 | `bool` |
| `is_primitive()` | 检查是否为原始类型 | `bool` |
| `is_complex()` | 检查是否为复合类型 | `bool` |
| `is_literal()` | 检查是否为字面量类型 | `bool` |

### 类型转换方法

| 方法名 | 描述 | 返回类型 |
|-------|------|---------|
| `to_boolean()` | 转换为 boolean 类型 | `bool` |
| `to_number()` | 转换为 number 类型 | `f64` |
| `to_string()` | 转换为 string 类型 | `String` |
| `type_name()` | 获取类型的字符串表示 | `String` |

### 类型操作方法

| 方法名 | 描述 | 参数 | 返回类型 |
|-------|------|------|---------|
| `is_assignable_to()` | 检查类型是否可赋值给目标类型 | `target: &TsValue` | `bool` |
| `get_property_keys()` | 获取类型的所有属性键 | 无 | `Vec<String>` |
| `get_property_type()` | 获取指定属性的类型 | `key: &str` | `Option<TsValue>` |
| `evaluate_conditional()` | 评估条件类型 | `check_type: &TsValue, extends_type: &TsValue` | `Option<bool>` |
| `infer_type_params()` | 推断类型参数 | `target: &TsValue` | `InferenceResult` |
| `substitute_type_params()` | 替换类型参数 | `substitutions: &HashMap<String, TsValue>` | `TsValue` |
| `intersection_with()` | 计算两个类型的交集 | `other: &TsValue` | `TsValue` |
| `difference_with()` | 计算两个类型的差集 | `other: &TsValue` | `TsValue` |
| `apply_mapped_type()` | 应用映射类型到对象 | `mapped_type: &Mapped` | `TsValue` |
| `simplify()` | 简化类型 | 无 | `TsValue` |
| `get_base_type()` | 获取类型的基础类型 | 无 | `TsValue` |

## 辅助类型

### Conditional 类型

表示条件类型 `T extends U ? X : Y`。

```rust
pub struct Conditional {
    pub check_type: Box<TsValue>,
    pub extends_type: Box<TsValue>,
    pub true_type: Box<TsValue>,
    pub false_type: Box<TsValue>,
    pub modifier: ConditionalModifier,
}
```

### Mapped 类型

表示映射类型 `{ [K in keyof T]: V }`。

```rust
pub struct Mapped {
    pub type_param: String,
    pub constraint: Box<TsValue>,
    pub value_type: Box<TsValue>,
    pub key_modifier: MappedKeyModifier,
    pub constraint_type: MappedConstraint,
}
```

### TemplateLiteral 类型

表示模板字面量类型 `` `${string}` ``。

```rust
pub struct TemplateLiteral {
    pub parts: Vec<TemplateLiteralPart>,
}
```

## 示例用法

### 创建和操作类型

```rust
use typescript_types::{TsValue, Conditional, Mapped};

// 创建基本类型
let number_type = TsValue::Number(42.0);
let string_type = TsValue::String("hello".to_string());
let boolean_type = TsValue::Boolean(true);

// 创建对象类型
let mut props = std::collections::HashMap::new();
props.insert("name".to_string(), TsValue::String("TypeScript".to_string()));
props.insert("version".to_string(), TsValue::Number(5.0));
let object_type = TsValue::Object(props);

// 创建联合类型
let union_type = TsValue::Union(vec![
    TsValue::Number(1.0),
    TsValue::String("text".to_string()),
    TsValue::Boolean(true),
]);

// 检查类型
assert!(number_type.is_number());
assert!(string_type.is_string());
assert!(object_type.is_object());
assert!(union_type.is_union());

// 类型操作
let is_assignable = number_type.is_assignable_to(&TsValue::Any);
assert!(is_assignable);

let property_keys = object_type.get_property_keys();
assert!(property_keys.contains(&"name".to_string()));
assert!(property_keys.contains(&"version".to_string()));
```

### 使用条件类型

```rust
use typescript_types::{TsValue, Conditional};

// 创建条件类型: T extends U ? X : Y
let conditional_type = Conditional::new(
    TsValue::Number(1.0),  // T
    TsValue::Number(0.0),  // U
    TsValue::String("true".to_string()),  // X
    TsValue::String("false".to_string()),  // Y
);

let ts_conditional = TsValue::Conditional(conditional_type);
```

### 使用映射类型

```rust
use typescript_types::{TsValue, Mapped};

// 创建映射类型: { [K in keyof T]: V }
let mapped_type = Mapped::new(
    "K".to_string(),  // 类型参数
    TsValue::KeyOf(Box::new(TsValue::Object(std::collections::HashMap::new()))),  // keyof T
    TsValue::String("mapped".to_string()),  // V
);

let ts_mapped = TsValue::Mapped(mapped_type);
```

## 性能优化

- **类型缓存**: 对于频繁使用的类型，可以使用缓存来避免重复创建
- **惰性求值**: 对于复杂类型操作，考虑使用惰性求值来提高性能
- **批量操作**: 对于多个类型操作，尽量使用批量处理来减少开销

## 错误处理

类型系统提供了 `Error` 类型来表示错误情况：

```rust
let error_type = TsValue::Error("Type error".to_string());
assert!(error_type.is_error());
```

## 总结

TypeScript 类型系统 API 提供了丰富的类型操作功能，支持 TypeScript 中的各种类型特性。通过这些 API，开发者可以：

1. 表示和操作各种 TypeScript 类型
2. 执行类型检查和转换
3. 处理复杂的类型关系
4. 实现类型推断和替换
5. 支持高级类型特性如条件类型和映射类型

这些 API 为 Rusty TypeScript 编译器提供了坚实的类型系统基础，使得编译器能够正确处理 TypeScript 的类型系统特性。