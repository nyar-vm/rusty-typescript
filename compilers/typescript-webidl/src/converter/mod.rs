/// WebIDL 到 TypeScript 转换模块
use oak_idl::ast::{IdlItem, IdlRoot};
use std::fmt::Write;

/// 将 WebIDL 类型映射到 TypeScript 类型
///
/// # 参数
/// - `webidl_type`: WebIDL 类型字符串
///
/// # 返回值
/// - 对应的 TypeScript 类型字符串
fn map_webidl_type(webidl_type: &str) -> String {
    // 移除空格，统一处理
    let webidl_type = webidl_type.trim();

    // 基本类型映射（使用静态字符串，避免重复分配）
    match webidl_type {
        // 字符串类型
        "DOMString" | "ByteString" | "USVString" | "string" => String::from("string"),
        // 布尔类型
        "boolean" => String::from("boolean"),
        // 数字类型
        "byte"
        | "octet"
        | "short"
        | "unsigned short"
        | "long"
        | "unsigned long"
        | "long long"
        | "unsigned long long"
        | "float"
        | "unrestricted float"
        | "double"
        | "unrestricted double"
        | "integer"
        | "unsigned integer" => String::from("number"),
        // 其他基本类型
        "any" => String::from("any"),
        "undefined" => String::from("undefined"),
        "null" => String::from("null"),
        "void" => String::from("void"),
        // 复杂类型处理
        _ => {
            // 处理序列类型 (sequence<T>)
            if webidl_type.starts_with("sequence<") && webidl_type.ends_with(">") {
                let inner_type = &webidl_type[9..webidl_type.len() - 1];
                let mut result = String::with_capacity(10 + inner_type.len());
                result.push_str("Array<");
                result.push_str(&map_webidl_type(inner_type));
                result.push('>');
                return result;
            }
            // 处理联合类型 (A or B)
            if webidl_type.contains(" or ") {
                let types: Vec<&str> = webidl_type.split(" or ").collect();
                let mut result = String::with_capacity(webidl_type.len());
                let mut first = true;
                for t in types {
                    if !first {
                        result.push_str(" | ");
                    }
                    result.push_str(&map_webidl_type(t.trim()));
                    first = false;
                }
                return result;
            }
            // 处理可选类型 (T?)
            if webidl_type.ends_with("?") {
                let base_type = &webidl_type[..webidl_type.len() - 1];
                let mut result = String::with_capacity(base_type.len() + 10);
                result.push_str(&map_webidl_type(base_type));
                result.push_str(" | undefined");
                return result;
            }
            // 处理 Promise 类型 (Promise<T>)
            if webidl_type.starts_with("Promise<") && webidl_type.ends_with(">") {
                let inner_type = &webidl_type[8..webidl_type.len() - 1];
                let mut result = String::with_capacity(12 + inner_type.len());
                result.push_str("Promise<");
                result.push_str(&map_webidl_type(inner_type));
                result.push('>');
                return result;
            }
            // 处理映射类型 (record<K, V>)
            if webidl_type.starts_with("record<") && webidl_type.ends_with(">") {
                let inner = &webidl_type[7..webidl_type.len() - 1];
                if let Some(comma_idx) = inner.find(",") {
                    let key_type = &inner[..comma_idx].trim();
                    let value_type = &inner[comma_idx + 1..].trim();
                    let mut result = String::with_capacity(20 + key_type.len() + value_type.len());
                    result.push_str("{ [key: ");
                    result.push_str(&map_webidl_type(key_type));
                    result.push_str("]: ");
                    result.push_str(&map_webidl_type(value_type));
                    result.push('}');
                    return result;
                }
            }
            // 处理数组类型 (T[])
            if webidl_type.ends_with("[]") {
                let base_type = &webidl_type[..webidl_type.len() - 2];
                let mut result = String::with_capacity(8 + base_type.len());
                result.push_str("Array<");
                result.push_str(&map_webidl_type(base_type));
                result.push('>');
                return result;
            }
            // 处理命名空间限定类型 (Namespace.Type)
            if webidl_type.contains(".") {
                // 保持命名空间限定不变
                return String::from(webidl_type);
            }
            // 处理泛型类型 (T<U>)
            if webidl_type.contains("<") && webidl_type.contains(">") {
                let parts: Vec<&str> = webidl_type.split("<").collect();
                if parts.len() == 2 {
                    let name = parts[0].trim();
                    let generic = &parts[1][..parts[1].len() - 1].trim();
                    let mut result = String::with_capacity(name.len() + 2 + generic.len());
                    result.push_str(name);
                    result.push('<');
                    result.push_str(&map_webidl_type(generic));
                    result.push('>');
                    return result;
                }
            }
            // 其他类型保持不变
            String::from(webidl_type)
        }
    }
}

/// 将 WebIDL AST 转换为 TypeScript 类型定义
///
/// # 参数
/// - `root`: WebIDL AST 根节点
///
/// # 返回值
/// - 生成的 TypeScript 类型定义字符串
pub fn convert(root: &IdlRoot) -> String {
    // 更精确的预分配字符串容量，减少内存分配
    let estimated_size = root.items.len() * 300; // 增加估计值以减少重新分配
    let mut result = String::with_capacity(estimated_size);

    // 调试信息
    println!("Items count: {}", root.items.len());
    for (i, item) in root.items.iter().enumerate() {
        println!("Item {}: {:?}", i, item);
    }

    for item in root.items.iter() {
        match item {
            IdlItem::Interface(interface) => {
                println!("处理接口: {}", interface.name);
                convert_interface(interface, &mut result);
                result.push_str("\n");
            }
            IdlItem::Struct(struct_) => {
                println!("处理结构体: {}", struct_.name);
                convert_struct(struct_, &mut result);
                result.push_str("\n");
            }
            IdlItem::Enum(enum_) => {
                println!("处理枚举: {}", enum_.name);
                convert_enum(enum_, &mut result);
                result.push_str("\n");
            }
            IdlItem::Typedef(typedef) => {
                println!("处理类型别名: {}", typedef.name);
                convert_typedef(typedef, &mut result);
                result.push_str("\n");
            }
            IdlItem::Const(const_) => {
                println!("处理常量: {}", const_.name);
                convert_const(const_, &mut result);
                result.push_str("\n");
            }
            IdlItem::Module(module) => {
                println!("处理模块: {}", module.name);
                convert_module(module, &mut result);
                result.push_str("\n");
            }
        }
    }

    // 收缩字符串容量到实际大小，减少内存使用
    result.shrink_to_fit();
    println!("转换结果长度: {}", result.len());
    result
}

/// 转换 WebIDL 模块为 TypeScript 命名空间
fn convert_module(module: &oak_idl::ast::Module, result: &mut String) {
    write!(result, "namespace {} {{\n", module.name).unwrap();

    for item in &module.items {
        match item {
            IdlItem::Interface(interface) => {
                convert_interface(interface, result);
                result.push_str("\n");
            }
            IdlItem::Struct(struct_) => {
                convert_struct(struct_, result);
                result.push_str("\n");
            }
            IdlItem::Enum(enum_) => {
                convert_enum(enum_, result);
                result.push_str("\n");
            }
            IdlItem::Typedef(typedef) => {
                convert_typedef(typedef, result);
                result.push_str("\n");
            }
            IdlItem::Const(const_) => {
                convert_const(const_, result);
                result.push_str("\n");
            }
            IdlItem::Module(submodule) => {
                convert_module(submodule, result);
                result.push_str("\n");
            }
        }
    }

    write!(result, "}}").unwrap();
}

/// 转换 WebIDL 接口为 TypeScript 接口
fn convert_interface(interface: &oak_idl::ast::Interface, result: &mut String) {
    write!(result, "interface {} {{\n", interface.name).unwrap();

    for member in &interface.members {
        match member {
            oak_idl::ast::IdlMember::Attribute(attr) => {
                let mapped_type = map_webidl_type(&attr.type_name);
                if attr.readonly {
                    write!(result, "    readonly {}: {};\n", attr.name, mapped_type).unwrap();
                }
                else {
                    write!(result, "    {}: {};\n", attr.name, mapped_type).unwrap();
                }
            }
            oak_idl::ast::IdlMember::Operation(op) => {
                write!(result, "    {}(", op.name).unwrap();
                // 优化参数格式化，减少中间分配
                let mut first = true;
                for arg in &op.params {
                    if !first {
                        write!(result, ", ").unwrap();
                    }
                    let mapped_type = map_webidl_type(&arg.type_name);
                    write!(result, "{}: {}", arg.name, mapped_type).unwrap();
                    first = false;
                }
                let mapped_return_type = map_webidl_type(&op.return_type);
                write!(result, "): {};\n", mapped_return_type).unwrap();
            }
        }
    }

    write!(result, "}}",).unwrap();
}

/// 转换 WebIDL 结构体为 TypeScript 类型
fn convert_struct(struct_: &oak_idl::ast::Struct, result: &mut String) {
    write!(result, "interface {} {{\n", struct_.name).unwrap();

    for field in &struct_.fields {
        // 处理可选属性和默认值
        let mut field_type = field.type_name.clone();
        let mut optional = false;

        // 检查是否是可选属性
        if field_type.ends_with('?') {
            optional = true;
            field_type = field_type.trim_end_matches('?').to_string();
        }

        // 检查是否有默认值
        let mut has_default = false;
        let mut default_value = String::new();
        if let Some(index) = field_type.find(" = ") {
            has_default = true;
            default_value = field_type[index + 3..].to_string();
            field_type = field_type[..index].to_string();
        }

        let mapped_type = map_webidl_type(&field_type);

        // 构建字段定义
        if optional {
            write!(result, "    {}?: {};\n", field.name, mapped_type).unwrap();
        }
        else if has_default {
            write!(result, "    {}: {} = {};\n", field.name, mapped_type, default_value).unwrap();
        }
        else {
            write!(result, "    {}: {};\n", field.name, mapped_type).unwrap();
        }
    }

    write!(result, "}}",).unwrap();
}

/// 转换 WebIDL 枚举为 TypeScript 枚举
fn convert_enum(enum_: &oak_idl::ast::Enum, result: &mut String) {
    write!(result, "enum {} {{\n", enum_.name).unwrap();

    for (i, variant) in enum_.variants.iter().enumerate() {
        if i > 0 {
            write!(result, ",\n").unwrap();
        }
        write!(result, "    {}", variant).unwrap();
    }

    write!(result, "\n}}",).unwrap();
}

/// 转换 WebIDL 类型别名为 TypeScript 类型别名
fn convert_typedef(typedef: &oak_idl::ast::Typedef, result: &mut String) {
    let mapped_type = map_webidl_type(&typedef.type_name);
    write!(result, "type {} = {};", typedef.name, mapped_type).unwrap();
}

/// 转换 WebIDL 常量为 TypeScript 常量
fn convert_const(const_: &oak_idl::ast::Const, result: &mut String) {
    let mapped_type = map_webidl_type(&const_.type_name);
    write!(result, "const {}: {} = {};", const_.name, mapped_type, const_.value).unwrap();
}
