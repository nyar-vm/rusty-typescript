# TypeScript Type System API Documentation

## 📋 Overview

The TypeScript type system is one of the core components of the Rusty TypeScript project, providing complete TypeScript type representation and manipulation functionality. This documentation details the type system's API interfaces, helping developers understand and use the various features of the type system.

## 🔍 Core Types

### TsValue Enum

`TsValue` is the core type of the type system, representing all possible values and types in TypeScript.

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

## 🛠️ Type Operation API

### Type Checking Methods

| Method Name | Description | Return Type |
|------------|-------------|------------|
| `is_undefined()` | Check if value is undefined type | `bool` |
| `is_null()` | Check if value is null type | `bool` |
| `is_boolean()` | Check if value is boolean type | `bool` |
| `is_number()` | Check if value is number type | `bool` |
| `is_string()` | Check if value is string type | `bool` |
| `is_object()` | Check if value is object type | `bool` |
| `is_array()` | Check if value is array type | `bool` |
| `is_function()` | Check if value is function type | `bool` |
| `is_error()` | Check if value is error type | `bool` |
| `is_union()` | Check if value is union type | `bool` |
| `is_generic()` | Check if value is generic type | `bool` |
| `is_symbol()` | Check if value is symbol type | `bool` |
| `is_bigint()` | Check if value is bigint type | `bool` |
| `is_date()` | Check if value is date type | `bool` |
| `is_regexp()` | Check if value is regexp type | `bool` |
| `is_map()` | Check if value is map type | `bool` |
| `is_set()` | Check if value is set type | `bool` |
| `is_promise()` | Check if value is promise type | `bool` |
| `is_iterable()` | Check if value is iterable type | `bool` |
| `is_conditional()` | Check if value is conditional type | `bool` |
| `is_mapped()` | Check if value is mapped type | `bool` |
| `is_template_literal()` | Check if value is template literal type | `bool` |
| `is_keyof()` | Check if value is keyof type | `bool` |
| `is_typeof()` | Check if value is typeof type | `bool` |
| `is_indexed_access()` | Check if value is indexed access type | `bool` |
| `is_tuple()` | Check if value is tuple type | `bool` |
| `is_readonly()` | Check if value is readonly type | `bool` |
| `is_nullable()` | Check if value is nullable type | `bool` |
| `is_non_nullable()` | Check if value is non-nullable type | `bool` |
| `is_infer()` | Check if value is infer type | `bool` |
| `is_function_type()` | Check if value is function type | `bool` |
| `is_constructor_type()` | Check if value is constructor type | `bool` |
| `is_this_type()` | Check if value is this type | `bool` |
| `is_never()` | Check if value is never type | `bool` |
| `is_unknown()` | Check if value is unknown type | `bool` |
| `is_any()` | Check if value is any type | `bool` |
| `is_void()` | Check if value is void type | `bool` |
| `is_primitive()` | Check if value is primitive type | `bool` |
| `is_complex()` | Check if value is complex type | `bool` |
| `is_literal()` | Check if value is literal type | `bool` |

### Type Conversion Methods

| Method Name | Description | Return Type |
|------------|-------------|------------|
| `to_boolean()` | Convert to boolean type | `bool` |
| `to_number()` | Convert to number type | `f64` |
| `to_string()` | Convert to string type | `String` |
| `type_name()` | Get string representation of type | `String` |

### Type Operation Methods

| Method Name | Description | Parameters | Return Type |
|------------|-------------|------------|------------|
| `is_assignable_to()` | Check if type is assignable to target type | `target: &TsValue` | `bool` |
| `get_property_keys()` | Get all property keys of type | None | `Vec<String>` |
| `get_property_type()` | Get type of specified property | `key: &str` | `Option<TsValue>` |
| `evaluate_conditional()` | Evaluate conditional type | `check_type: &TsValue, extends_type: &TsValue` | `Option<bool>` |
| `infer_type_params()` | Infer type parameters | `target: &TsValue` | `InferenceResult` |
| `substitute_type_params()` | Substitute type parameters | `substitutions: &HashMap<String, TsValue>` | `TsValue` |
| `intersection_with()` | Calculate intersection of two types | `other: &TsValue` | `TsValue` |
| `difference_with()` | Calculate difference of two types | `other: &TsValue` | `TsValue` |
| `apply_mapped_type()` | Apply mapped type to object | `mapped_type: &Mapped` | `TsValue` |
| `simplify()` | Simplify type | None | `TsValue` |
| `get_base_type()` | Get base type of type | None | `TsValue` |

## 📦 Helper Types

### Conditional Type

Represents conditional type `T extends U ? X : Y`.

```rust
pub struct Conditional {
    pub check_type: Box<TsValue>,
    pub extends_type: Box<TsValue>,
    pub true_type: Box<TsValue>,
    pub false_type: Box<TsValue>,
    pub modifier: ConditionalModifier,
}
```

### Mapped Type

Represents mapped type `{ [K in keyof T]: V }`.

```rust
pub struct Mapped {
    pub type_param: String,
    pub constraint: Box<TsValue>,
    pub value_type: Box<TsValue>,
    pub key_modifier: MappedKeyModifier,
    pub constraint_type: MappedConstraint,
}
```

### TemplateLiteral Type

Represents template literal type `` `${string}` ``.

```rust
pub struct TemplateLiteral {
    pub parts: Vec<TemplateLiteralPart>,
}
```

## 🚀 Usage Examples

### Creating and Manipulating Types

```rust
use typescript_types::{TsValue, Conditional, Mapped};

// Create basic types
let number_type = TsValue::Number(42.0);
let string_type = TsValue::String("hello".to_string());
let boolean_type = TsValue::Boolean(true);

// Create object type
let mut props = std::collections::HashMap::new();
props.insert("name".to_string(), TsValue::String("TypeScript".to_string()));
props.insert("version".to_string(), TsValue::Number(5.0));
let object_type = TsValue::Object(props);

// Create union type
let union_type = TsValue::Union(vec![
    TsValue::Number(1.0),
    TsValue::String("text".to_string()),
    TsValue::Boolean(true),
]);

// Check types
assert!(number_type.is_number());
assert!(string_type.is_string());
assert!(object_type.is_object());
assert!(union_type.is_union());

// Type operations
let is_assignable = number_type.is_assignable_to(&TsValue::Any);
assert!(is_assignable);

let property_keys = object_type.get_property_keys();
assert!(property_keys.contains(&"name".to_string()));
assert!(property_keys.contains(&"version".to_string()));
```

### Using Conditional Types

```rust
use typescript_types::{TsValue, Conditional};

// Create conditional type: T extends U ? X : Y
let conditional_type = Conditional::new(
    TsValue::Number(1.0),  // T
    TsValue::Number(0.0),  // U
    TsValue::String("true".to_string()),  // X
    TsValue::String("false".to_string()),  // Y
);

let ts_conditional = TsValue::Conditional(conditional_type);
```

### Using Mapped Types

```rust
use typescript_types::{TsValue, Mapped};

// Create mapped type: { [K in keyof T]: V }
let mapped_type = Mapped::new(
    "K".to_string(),  // Type parameter
    TsValue::KeyOf(Box::new(TsValue::Object(std::collections::HashMap::new()))),  // keyof T
    TsValue::String("mapped".to_string()),  // V
);

let ts_mapped = TsValue::Mapped(mapped_type);
```

## ⚡ Performance Optimization

- **Type Caching**: For frequently used types, use caching to avoid duplicate creation
- **Lazy Evaluation**: For complex type operations, consider using lazy evaluation to improve performance
- **Batch Operations**: For multiple type operations, use batch processing to reduce overhead

## 🛡️ Error Handling

The type system provides an `Error` type to represent error situations:

```rust
let error_type = TsValue::Error("Type error".to_string());
assert!(error_type.is_error());
```

## 📝 Summary

The TypeScript Type System API provides rich type manipulation functionality, supporting various TypeScript type features. Through these APIs, developers can:

1. Represent and manipulate various TypeScript types
2. Perform type checking and conversion
3. Handle complex type relationships
4. Implement type inference and substitution
5. Support advanced type features like conditional types and mapped types

These APIs provide a solid type system foundation for the Rusty TypeScript compiler, enabling the compiler to correctly handle TypeScript's type system features.
