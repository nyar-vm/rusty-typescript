use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use typescript_macros::{
    TypescriptClass, TypescriptEnum, TypescriptGuard, TypescriptInterface, TypescriptType, TypescriptUnion,
    typescript_function, typescript_function_declaration, typescript_namespace,
};

// 测试基本类
#[derive(TypescriptClass)]
struct BasicClass {
    id: u32,
    name: String,
    active: bool,
}

// 测试接口
#[derive(TypescriptInterface)]
struct BasicInterface {
    id: u32,
    name: String,
    active: bool,
}

// 测试枚举
#[derive(TypescriptEnum)]
enum BasicEnum {
    Red,
    Green,
    Blue,
}

// 测试类型别名
#[derive(TypescriptType)]
struct BasicType {
    id: u32,
    name: String,
    active: bool,
}

// 测试联合类型
#[derive(TypescriptUnion)]
enum BasicUnion {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    Triangle { base: f64, height: f64 },
}

// 测试类型守卫
#[derive(TypescriptGuard)]
struct BasicGuard {
    id: u32,
    name: String,
    active: bool,
}

// 测试函数
#[typescript_function]
fn basic_function(a: u32, b: u32) -> u32 {
    a + b
}

// 测试函数声明
#[typescript_function_declaration]
fn basic_function_declaration(name: String) -> String {
    format!("Hello, {}", name)
}

// 测试命名空间
#[typescript_namespace("utils")]
mod utils {
    pub fn add(a: u32, b: u32) -> u32 {
        a + b
    }
}

// 测试带有自定义属性的类
#[derive(TypescriptClass, TypescriptInterface, TypescriptType)]
struct ClassWithAttributes {
    id: u32,
    #[ts(rename = "fullName")]
    name: String,
    #[ts(skip)]
    password: String,
    #[ts(optional)]
    age: u32,
    #[ts(type = "Date")]
    created_at: String,
}

// 测试带有复杂类型的类
#[derive(TypescriptType)]
struct ComplexTypes {
    // 基本类型
    number: u32,
    boolean: bool,
    string: String,
    // 集合类型
    vector: Vec<String>,
    hash_set: HashSet<u32>,
    btree_set: BTreeSet<String>,
    // 映射类型
    hash_map: HashMap<String, u32>,
    btree_map: BTreeMap<String, String>,
    // 元组类型
    tuple: (String, u32, bool),
    // 可选类型
    optional: Option<String>,
    // 嵌套类型
    nested: Vec<Option<HashMap<String, Vec<u32>>>>,
}

// 测试复杂的联合类型
#[derive(TypescriptUnion)]
enum ComplexUnion {
    Unit,
    StringValue(String),
    NumberValue(u32),
    Object { id: u32, name: String, tags: Vec<String> },
}

#[test]
fn test_basic_class() {
    let class_def = BasicClass::TS_CLASS_DEFINITION;
    assert!(class_def.contains("class BasicClass"));
    assert!(class_def.contains("id: number"));
    assert!(class_def.contains("name: string"));
    assert!(class_def.contains("active: boolean"));
    assert!(class_def.contains("constructor"));
}

#[test]
fn test_basic_interface() {
    let interface_def = BasicInterface::TS_INTERFACE_DEFINITION;
    assert!(interface_def.contains("interface BasicInterface"));
    assert!(interface_def.contains("id: number"));
    assert!(interface_def.contains("name: string"));
    assert!(interface_def.contains("active: boolean"));
}

#[test]
fn test_basic_enum() {
    let enum_def = BasicEnum::TS_ENUM_DEFINITION;
    assert!(enum_def.contains("enum BasicEnum"));
    assert!(enum_def.contains("Red = 0"));
    assert!(enum_def.contains("Green = 1"));
    assert!(enum_def.contains("Blue = 2"));
}

#[test]
fn test_basic_type() {
    let type_def = BasicType::TS_TYPE_DEFINITION;
    assert!(type_def.contains("type BasicType ="));
    assert!(type_def.contains("id: number"));
    assert!(type_def.contains("name: string"));
    assert!(type_def.contains("active: boolean"));
}

#[test]
fn test_basic_union() {
    let union_def = BasicUnion::TS_UNION_DEFINITION;
    assert!(union_def.contains("type BasicUnion ="));
    assert!(union_def.contains("| { type: \"Circle\""));
    assert!(union_def.contains("radius: number"));
    assert!(union_def.contains("| { type: \"Rectangle\""));
    assert!(union_def.contains("width: number"));
    assert!(union_def.contains("height: number"));
    assert!(union_def.contains("| { type: \"Triangle\""));
    assert!(union_def.contains("base: number"));
    assert!(union_def.contains("height: number"));
}

#[test]
fn test_basic_function() {
    let function_def = basic_function_TS_FUNCTION_DEFINITION;
    assert!(function_def.contains("type basic_functionFunction"));
    assert!(function_def.contains("a: number"));
    assert!(function_def.contains("b: number"));
    assert!(function_def.contains("=> number"));
}

#[test]
fn test_basic_function_declaration() {
    let function_decl_def = basic_function_declaration_TS_FUNCTION_DECLARATION;
    assert!(function_decl_def.contains("function basic_function_declaration"));
    assert!(function_decl_def.contains("name: string"));
    assert!(function_decl_def.contains(": string"));
}

#[test]
fn test_basic_namespace() {
    let namespace_def = utils_TS_NAMESPACE;
    assert!(namespace_def.contains("namespace utils"));
}

#[test]
fn test_class_with_attributes() {
    // 测试类
    let class_def = ClassWithAttributes::TS_CLASS_DEFINITION;
    assert!(class_def.contains("class ClassWithAttributes"));
    assert!(class_def.contains("id: number"));
    assert!(class_def.contains("fullName: string")); // 重命名测试
    assert!(!class_def.contains("password")); // 跳过测试
    assert!(class_def.contains("age: number | undefined")); // 可选测试
    assert!(class_def.contains("created_at: Date")); // 自定义类型测试

    // 测试接口
    let interface_def = ClassWithAttributes::TS_INTERFACE_DEFINITION;
    assert!(interface_def.contains("interface ClassWithAttributes"));
    assert!(interface_def.contains("id: number"));
    assert!(interface_def.contains("fullName: string"));
    assert!(!interface_def.contains("password"));
    assert!(interface_def.contains("age: number | undefined"));
    assert!(interface_def.contains("created_at: Date"));

    // 测试类型
    let type_def = ClassWithAttributes::TS_TYPE_DEFINITION;
    assert!(type_def.contains("type ClassWithAttributes ="));
    assert!(type_def.contains("id: number"));
    assert!(type_def.contains("fullName: string"));
    assert!(!type_def.contains("password"));
    assert!(type_def.contains("age: number | undefined"));
    assert!(type_def.contains("created_at: Date"));
}

#[test]
fn test_complex_types() {
    let type_def = ComplexTypes::TS_TYPE_DEFINITION;
    assert!(type_def.contains("type ComplexTypes ="));
    // 基本类型
    assert!(type_def.contains("number: number"));
    assert!(type_def.contains("boolean: boolean"));
    assert!(type_def.contains("string: string"));
    // 集合类型
    assert!(type_def.contains("vector: (string)[]"));
    assert!(type_def.contains("hash_set: Set<number>"));
    assert!(type_def.contains("btree_set: Set<string>"));
    // 映射类型
    assert!(type_def.contains("hash_map: Record<string, number>"));
    assert!(type_def.contains("btree_map: Record<string, string>"));
    // 元组类型
    assert!(type_def.contains("tuple: [string, number, boolean]"));
    // 可选类型
    assert!(type_def.contains("optional: (string) | undefined"));
    // 嵌套类型
    assert!(type_def.contains("nested: ((Record<string, (number)[]>) | undefined)[]"));
}

#[test]
fn test_complex_union() {
    let union_def = ComplexUnion::TS_UNION_DEFINITION;
    assert!(union_def.contains("type ComplexUnion ="));
    assert!(union_def.contains("| { type: \"Unit\" }"));
    assert!(union_def.contains("| { type: \"StringValue\"; value: string }"));
    assert!(union_def.contains("| { type: \"NumberValue\"; value: number }"));
    assert!(union_def.contains("| { type: \"Object\""));
    assert!(union_def.contains("id: number"));
    assert!(union_def.contains("name: string"));
    assert!(union_def.contains("tags: (string)[]"));
}
