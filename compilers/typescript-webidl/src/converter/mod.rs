//! WebIDL 到 TypeScript 转换模块
//!
//! 本模块提供将 WebIDL 类型定义转换为 TypeScript 类型定义的功能。

use oak_idl::ast::{Attribute, IdlItem, IdlMember, IdlRoot, Interface, Operation, Param};
use std::fmt::Write;

/// 常用 TypeScript 类型静态字符串
mod static_types {
    pub const STRING: &str = "string";
    pub const BOOLEAN: &str = "boolean";
    pub const NUMBER: &str = "number";
    pub const OBJECT: &str = "object";
    pub const SYMBOL: &str = "symbol";
    pub const ANY: &str = "any";
    pub const UNDEFINED: &str = "undefined";
    pub const NULL: &str = "null";
    pub const VOID: &str = "void";
    pub const ARRAY: &str = "Array";
}

/// 将 WebIDL 类型映射到 TypeScript 类型
pub fn map_webidl_type(webidl_type: &str) -> String {
    let webidl_type = webidl_type.trim();

    // 处理复杂类型，如泛型、联合类型等
    if webidl_type.contains('<') && webidl_type.contains('>') {
        let generics_start = webidl_type.find('<').unwrap();
        let generics_end = webidl_type.rfind('>').unwrap();
        let base_type = &webidl_type[..generics_start];
        let generic_args = &webidl_type[generics_start + 1..generics_end];

        let mapped_base = map_webidl_type(base_type);
        let mapped_args: Vec<String> = generic_args.split(',').map(|arg| map_webidl_type(arg.trim())).collect();

        return format!("{}<{}", mapped_base, mapped_args.join(", "));
    }

    if webidl_type.contains('|') {
        let types: Vec<String> = webidl_type.split('|').map(|t| map_webidl_type(t.trim())).collect();
        return types.join(" | ");
    }

    match webidl_type {
        "DOMString" | "ByteString" | "USVString" | "string" => String::from(static_types::STRING),
        "boolean" => String::from(static_types::BOOLEAN),
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
        | "unsigned integer"
        | "bigint"
        | "unsigned bigint" => String::from(static_types::NUMBER),
        "object" => String::from(static_types::OBJECT),
        "symbol" => String::from(static_types::SYMBOL),
        "any" => String::from(static_types::ANY),
        "undefined" => String::from(static_types::UNDEFINED),
        "null" => String::from(static_types::NULL),
        "void" => String::from(static_types::VOID),
        "sequence" => String::from(static_types::ARRAY),
        "FrozenArray" => String::from(static_types::ARRAY),
        _ => String::from(webidl_type),
    }
}

/// 将 WebIDL AST 转换为 TypeScript 类型定义
pub fn convert(root: &IdlRoot) -> String {
    let estimated_size = root.items.len() * 300;
    let mut result = String::with_capacity(estimated_size);

    for item in root.items.iter() {
        match item {
            IdlItem::Interface(interface) => {
                convert_interface(interface, &mut result);
                result.push_str("\n");
            }
            IdlItem::Const(constant) => {
                let mapped_type = map_webidl_type(&constant.type_name);
                write!(result, "const {}: {} = {};\n", constant.name, mapped_type, constant.value).unwrap();
            }
            _ => {}
        }
    }

    result.shrink_to_fit();
    result
}

/// 转换 WebIDL 接口为 TypeScript 接口
fn convert_interface(interface: &Interface, result: &mut String) {
    write!(
        result,
        "interface {} {{
",
        interface.name
    )
    .unwrap();

    // 处理属性和操作
    for member in &interface.members {
        match member {
            IdlMember::Attribute(attr) => {
                let mapped_type = map_webidl_type(&attr.type_name);

                // 处理 getters/setters
                if attr.name.starts_with("get_") {
                    let getter_name = &attr.name[4..];
                    write!(result, "    get {}(): {};\n", getter_name, mapped_type).unwrap();
                }
                else if attr.name.starts_with("set_") {
                    let setter_name = &attr.name[4..];
                    write!(result, "    set {}({}: {});\n", setter_name, setter_name, mapped_type).unwrap();
                }
                else if attr.readonly {
                    write!(result, "    readonly {}: {};\n", attr.name, mapped_type).unwrap();
                }
                else {
                    write!(result, "    {}: {};\n", attr.name, mapped_type).unwrap();
                }
            }
            IdlMember::Operation(op) => {
                let mapped_return_type = map_webidl_type(&op.return_type);

                write!(result, "    {}(", op.name).unwrap();

                for (i, param) in op.params.iter().enumerate() {
                    if i > 0 {
                        write!(result, ", ").unwrap();
                    }
                    let mapped_param_type = map_webidl_type(&param.type_name);

                    // 处理可选参数（简单实现，假设最后一个参数是可选的）
                    if i == op.params.len() - 1 {
                        write!(result, "{}?: {}", param.name, mapped_param_type).unwrap();
                    }
                    else {
                        write!(result, "{}: {}", param.name, mapped_param_type).unwrap();
                    }
                }

                write!(result, "): {};\n", mapped_return_type).unwrap();
            }
            _ => {}
        }
    }

    write!(result, "}}\n").unwrap();
}
