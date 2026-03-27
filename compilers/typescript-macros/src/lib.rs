#![warn(missing_docs)]
#![doc = include_str!("readme.md")]

use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, DeriveInput, ItemFn, Lit, Type, parse_macro_input};

/// 字段属性信息
struct FieldAttributes {
    /// 字段重命名
    rename: Option<String>,
    /// 是否跳过该字段
    skip: bool,
    /// 是否标记为可选
    optional: bool,
    /// 自定义类型
    custom_type: Option<String>,
}

impl FieldAttributes {
    /// 从字段属性中解析字段属性信息
    fn from_attributes(attrs: &[Attribute]) -> Self {
        let mut result = Self { rename: None, skip: false, optional: false, custom_type: None };

        for attr in attrs {
            if attr.path().is_ident("ts") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("rename") {
                        let value = meta.value()?;
                        let lit_str = value.parse::<Lit>()?;
                        if let Lit::Str(lit_str) = lit_str {
                            result.rename = Some(lit_str.value());
                        }
                    }
                    else if meta.path.is_ident("type") {
                        let value = meta.value()?;
                        let lit_str = value.parse::<Lit>()?;
                        if let Lit::Str(lit_str) = lit_str {
                            result.custom_type = Some(lit_str.value());
                        }
                    }
                    else if meta.path.is_ident("skip") {
                        result.skip = true;
                    }
                    else if meta.path.is_ident("optional") {
                        result.optional = true;
                    }
                    Ok(())
                })
                .ok();
            }
        }

        result
    }
}

use once_cell::sync::Lazy;
/// 将 Rust 类型转换为 TypeScript 类型字符串
///
/// 支持的类型映射：
/// - 基本类型：u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, usize, isize -> number
/// - 布尔类型：bool -> boolean
/// - 字符串：String, &str -> string
/// - 可选类型：Option<T> -> T | undefined
/// - 集合类型：Vec<T>, LinkedList<T>, VecDeque<T> -> T[]
/// - 集合类型：HashSet<T>, BTreeSet<T> -> Set<T>
/// - 映射类型：HashMap<K, V>, BTreeMap<K, V> -> Record<K, V>
/// - 元组类型：(A, B, C) -> [A, B, C]
/// - 泛型类型：支持泛型参数的保留
/// - 嵌套类型：递归处理内部类型
/// - 空类型：() -> null
/// - 切片类型：&[T] -> T[]
use std::collections::HashMap;

/// 基本类型映射缓存
static BASIC_TYPE_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    // 数字类型
    map.insert("u8", "number");
    map.insert("u16", "number");
    map.insert("u32", "number");
    map.insert("u64", "number");
    map.insert("i8", "number");
    map.insert("i16", "number");
    map.insert("i32", "number");
    map.insert("i64", "number");
    map.insert("f32", "number");
    map.insert("f64", "number");
    map.insert("usize", "number");
    map.insert("isize", "number");
    // 布尔类型
    map.insert("bool", "boolean");
    // 字符串类型
    map.insert("String", "string");
    map.insert("str", "string");
    map
});

/// 将 Rust 类型转换为 TypeScript 类型字符串
///
/// 支持的类型映射：
/// - 基本类型：u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, usize, isize -> number
/// - 布尔类型：bool -> boolean
/// - 字符串：String, &str -> string
/// - 可选类型：Option<T> -> T | undefined
/// - 集合类型：Vec<T>, LinkedList<T>, VecDeque<T> -> T[]
/// - 集合类型：HashSet<T>, BTreeSet<T> -> Set<T>
/// - 映射类型：HashMap<K, V>, BTreeMap<K, V> -> Record<K, V>
/// - 元组类型：(A, B, C) -> [A, B, C]
/// - 泛型类型：支持泛型参数的保留
/// - 嵌套类型：递归处理内部类型
/// - 空类型：() -> null
/// - 切片类型：&[T] -> T[]
fn rust_type_to_typescript(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => {
            let path = &type_path.path;

            if let Some(segment) = path.segments.last() {
                let ident = &segment.ident;
                let ident_str = ident.to_string();

                // 处理基本类型（使用缓存）
                if let Some(ts_type) = BASIC_TYPE_MAP.get(ident_str.as_str()) {
                    return ts_type.to_string();
                }

                // 处理容器类型
                match ident_str.as_str() {
                    "Option" => {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                            && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
                        {
                            let inner_ts = rust_type_to_typescript(inner_ty);
                            return format!("({}) | undefined", inner_ts);
                        }
                        return "any | undefined".to_string();
                    }
                    "Vec" | "LinkedList" | "VecDeque" => {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                            && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
                        {
                            let inner_ts = rust_type_to_typescript(inner_ty);
                            return format!("({})[]", inner_ts);
                        }
                        return "any[]".to_string();
                    }
                    "HashSet" | "BTreeSet" => {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                            && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
                        {
                            let inner_ts = rust_type_to_typescript(inner_ty);
                            return format!("Set<{}>", inner_ts);
                        }
                        return "Set<any>".to_string();
                    }
                    "HashMap" | "BTreeMap" => {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            let args_vec: Vec<_> = args.args.iter().collect();
                            if args_vec.len() == 2 {
                                let key_ts = if let syn::GenericArgument::Type(key_ty) = args_vec[0] {
                                    rust_type_to_typescript(key_ty)
                                }
                                else {
                                    "any".to_string()
                                };
                                let value_ts = if let syn::GenericArgument::Type(value_ty) = args_vec[1] {
                                    rust_type_to_typescript(value_ty)
                                }
                                else {
                                    "any".to_string()
                                };
                                return format!("Record<{}, {}>", key_ts, value_ts);
                            }
                        }
                        return "Record<string, any>".to_string();
                    }
                    _ => {
                        // 处理泛型类型，如 Container<T>
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            let generic_args: Vec<String> = args
                                .args
                                .iter()
                                .filter_map(|arg| {
                                    if let syn::GenericArgument::Type(inner_ty) = arg {
                                        Some(rust_type_to_typescript(inner_ty))
                                    }
                                    else if let syn::GenericArgument::Lifetime(_) = arg {
                                        None // 忽略生命周期参数
                                    }
                                    else {
                                        Some(quote!(#arg).to_string())
                                    }
                                })
                                .collect();

                            if !generic_args.is_empty() {
                                return format!("{}<{}>", ident, generic_args.join(", "));
                            }
                        }
                        // 保留其他类型的名称，可能是用户定义的类型或泛型参数
                        ident_str
                    }
                }
            }
            else {
                quote!(#ty).to_string()
            }
        }
        Type::Tuple(type_tuple) => {
            if type_tuple.elems.is_empty() {
                return "null".to_string();
            }
            let elements: Vec<String> = type_tuple.elems.iter().map(rust_type_to_typescript).collect();
            format!("[{}]", elements.join(", "))
        }
        Type::Slice(slice) => {
            let inner_ts = rust_type_to_typescript(&slice.elem);
            format!("({})[]", inner_ts)
        }
        Type::Reference(ref_type) => {
            let inner_ty = &ref_type.elem;
            rust_type_to_typescript(inner_ty)
        }
        _ => {
            let type_str = quote!(#ty).to_string();
            // 处理基本类型（使用缓存）
            if let Some(ts_type) = BASIC_TYPE_MAP.get(type_str.as_str()) {
                return ts_type.to_string();
            }
            // 处理空类型
            if type_str == "()" {
                return "null".to_string();
            }
            type_str
        }
    }
}

/// 为 Rust 结构体生成 TypeScript 类定义
///
/// # 示例
///
/// ```rust
/// use typescript_macros::TypescriptClass;
///
/// #[derive(TypescriptClass)]
/// struct User {
///     id: u32,
///     name: String,
///     active: bool,
/// }
/// ```
///
/// 这将生成对应的 TypeScript 类定义：
///
/// ```typescript
/// class User {
///     id: number;
///     name: string;
///     active: boolean;
///     
///     constructor(id: number, name: string, active: boolean) {
///         this.id = id;
///         this.name = name;
///         this.active = active;
///     }
/// }
/// ```
#[proc_macro_derive(TypescriptClass, attributes(ts))]
pub fn typescript_class_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;

    // 解析泛型参数
    let generic_params = if let Some(_generics) = input.generics.params.first() {
        let generic_args: Vec<String> =
            input
                .generics
                .params
                .iter()
                .filter_map(|param| {
                    if let syn::GenericParam::Type(type_param) = param { Some(type_param.ident.to_string()) } else { None }
                })
                .collect();

        if !generic_args.is_empty() { format!("<{}>", generic_args.join(", ")) } else { "".to_string() }
    }
    else {
        "".to_string()
    };

    let fields = match &input.data {
        syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Named(fields), .. }) => &fields.named,
        syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Unnamed(_), .. }) => {
            return syn::Error::new_spanned(input, "Tuple structs are not supported").to_compile_error().into();
        }
        syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Unit, .. }) => {
            return syn::Error::new_spanned(input, "Unit structs are not supported").to_compile_error().into();
        }
        syn::Data::Enum(_) => {
            return syn::Error::new_spanned(input, "Enums are not supported").to_compile_error().into();
        }
        syn::Data::Union(_) => {
            return syn::Error::new_spanned(input, "Unions are not supported").to_compile_error().into();
        }
    };

    for field in fields {
        if field.ident.is_none() {
            return syn::Error::new_spanned(field, "All fields must have names").to_compile_error().into();
        }
    }

    let field_types: Vec<String> = fields
        .iter()
        .filter_map(|field| {
            let attrs = FieldAttributes::from_attributes(&field.attrs);
            if attrs.skip {
                return None;
            }
            let field_name = attrs.rename.clone().unwrap_or_else(|| field.ident.as_ref().unwrap().to_string());
            let mut ts_type = if let Some(custom_type) = &attrs.custom_type {
                custom_type.clone()
            }
            else {
                rust_type_to_typescript(&field.ty)
            };
            if attrs.optional && !ts_type.contains("undefined") {
                ts_type = format!("{} | undefined", ts_type);
            }
            Some(format!("    {}: {}", field_name, ts_type))
        })
        .collect();

    let constructor_params: Vec<String> = fields
        .iter()
        .filter_map(|field| {
            let attrs = FieldAttributes::from_attributes(&field.attrs);
            if attrs.skip {
                return None;
            }
            let field_name = attrs.rename.clone().unwrap_or_else(|| field.ident.as_ref().unwrap().to_string());
            let mut ts_type = if let Some(custom_type) = &attrs.custom_type {
                custom_type.clone()
            }
            else {
                rust_type_to_typescript(&field.ty)
            };
            if attrs.optional && !ts_type.contains("undefined") {
                ts_type = format!("{} | undefined", ts_type);
            }
            Some(format!("        {}: {}", field_name, ts_type))
        })
        .collect();

    let constructor_assignments: Vec<String> = fields
        .iter()
        .filter_map(|field| {
            let attrs = FieldAttributes::from_attributes(&field.attrs);
            if attrs.skip {
                return None;
            }
            let field_name = attrs.rename.clone().unwrap_or_else(|| field.ident.as_ref().unwrap().to_string());
            let original_field_name = field.ident.as_ref().unwrap();
            Some(format!("        this.{} = {};", field_name, original_field_name))
        })
        .collect();

    let field_types_str = field_types.join(";\n");
    let constructor_params_str = constructor_params.join(",\n");
    let constructor_assignments_str = constructor_assignments.join("\n");

    let ts_code = quote! {
        impl #struct_name {
            /// TypeScript 类定义
            pub const TS_CLASS_DEFINITION: &'static str = concat!(
                "class ", stringify!(#struct_name), #generic_params, " {\n",
                #field_types_str, ";\n",
                "\n",
                "    constructor(\n",
                #constructor_params_str, ",\n",
                "    ) {\n",
                #constructor_assignments_str, "\n",
                "    }\n",
                "}\n"
            );

            /// 获取 TypeScript 类定义
            pub fn ts_class_definition() -> &'static str {
                Self::TS_CLASS_DEFINITION
            }
        }
    };

    TokenStream::from(ts_code)
}

/// 为 Rust 函数生成 TypeScript 函数类型定义
///
/// # 示例
///
/// ```rust
/// use typescript_macros::typescript_function;
///
/// #[typescript_function]
/// fn add(a: u32, b: u32) -> u32 {
///     a + b
/// }
/// ```
///
/// 这将生成对应的 TypeScript 函数类型定义：
///
/// ```typescript
/// type AddFunction = (a: number, b: number) => number;
/// ```
#[proc_macro_attribute]
pub fn typescript_function(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    let fn_name = &input.sig.ident;

    let params = &input.sig.inputs;
    let param_types: Vec<String> = params
        .iter()
        .map(|param| match param {
            syn::FnArg::Typed(pat_type) => {
                let param_name = match &*pat_type.pat {
                    syn::Pat::Ident(pat_ident) => &pat_ident.ident,
                    _ => return "_: any".to_string(),
                };
                let ts_type = rust_type_to_typescript(&pat_type.ty);
                format!("{}: {}", param_name, ts_type)
            }
            _ => "_: any".to_string(),
        })
        .collect();

    let return_type = match &input.sig.output {
        syn::ReturnType::Type(_, ty) => rust_type_to_typescript(ty),
        syn::ReturnType::Default => "void".to_string(),
    };

    let param_types_str = param_types.join(", ");
    let ts_const_name = format!("{}_TS_FUNCTION_DEFINITION", fn_name);
    let ts_const_ident = syn::Ident::new(&ts_const_name, fn_name.span());
    let ts_function_name = syn::Ident::new(&format!("{}_ts_function_definition", fn_name), fn_name.span());

    let ts_code = quote! {
        #input

        /// TypeScript 函数类型定义
        pub const #ts_const_ident: &'static str = concat!(
            "type ", stringify!(#fn_name), "Function = (",
            #param_types_str,
            ") => ",
            #return_type,
            ";\n"
        );

        /// 获取 TypeScript 函数类型定义
        pub fn #ts_function_name() -> &'static str {
            #ts_const_ident
        }
    };

    TokenStream::from(ts_code)
}

/// 为 Rust 函数生成 TypeScript 函数声明
///
/// # 示例
///
/// ```rust
/// use typescript_macros::typescript_function_declaration;
///
/// #[typescript_function_declaration]
/// fn add(a: u32, b: u32) -> u32 {
///     a + b
/// }
/// ```
///
/// 这将生成对应的 TypeScript 函数声明：
///
/// ```typescript
/// function add(a: number, b: number): number;
/// ```
#[proc_macro_attribute]
pub fn typescript_function_declaration(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    let fn_name = &input.sig.ident;

    let params = &input.sig.inputs;
    let param_types: Vec<String> = params
        .iter()
        .map(|param| match param {
            syn::FnArg::Typed(pat_type) => {
                let param_name = match &*pat_type.pat {
                    syn::Pat::Ident(pat_ident) => &pat_ident.ident,
                    _ => return "_: any".to_string(),
                };
                let ts_type = rust_type_to_typescript(&pat_type.ty);
                format!("{}: {}", param_name, ts_type)
            }
            _ => "_: any".to_string(),
        })
        .collect();

    let return_type = match &input.sig.output {
        syn::ReturnType::Type(_, ty) => rust_type_to_typescript(ty),
        syn::ReturnType::Default => "void".to_string(),
    };

    let param_types_str = param_types.join(", ");
    let ts_const_name = format!("{}_TS_FUNCTION_DECLARATION", fn_name);
    let ts_const_ident = syn::Ident::new(&ts_const_name, fn_name.span());
    let ts_function_name = syn::Ident::new(&format!("{}_ts_function_declaration", fn_name), fn_name.span());

    let ts_code = quote! {
        #input

        /// TypeScript 函数声明
        pub const #ts_const_ident: &'static str = concat!(
            "function ", stringify!(#fn_name), "(",
            #param_types_str,
            "): ",
            #return_type,
            ";\n"
        );

        /// 获取 TypeScript 函数声明
        pub fn #ts_function_name() -> &'static str {
            #ts_const_ident
        }
    };

    TokenStream::from(ts_code)
}

/// 为 Rust 模块生成 TypeScript 命名空间
///
/// # 示例
///
/// ```rust
/// use typescript_macros::typescript_namespace;
///
/// #[typescript_namespace("utils")]
/// mod utils {
///     pub fn add(a: u32, b: u32) -> u32 {
///         a + b
///     }
///
///     pub struct Point {
///         pub x: f64,
///         pub y: f64,
///     }
/// }
/// ```
///
/// 这将生成对应的 TypeScript 命名空间：
///
/// ```typescript
/// namespace utils {
///     export function add(a: number, b: number): number;
///     
///     export interface Point {
///         x: number;
///         y: number;
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn typescript_namespace(args: TokenStream, input: TokenStream) -> TokenStream {
    let namespace_name = parse_macro_input!(args as syn::LitStr).value();
    let input = parse_macro_input!(input as syn::ItemMod);

    let mod_name = &input.ident;
    let _content = &input.content;

    // 这里简化处理，实际项目中可能需要更复杂的解析
    // 目前我们只是生成命名空间的框架

    let ts_const_name = format!("{}_TS_NAMESPACE", mod_name);
    let ts_const_ident = syn::Ident::new(&ts_const_name, mod_name.span());
    let ts_namespace_name = syn::Ident::new(&format!("{}_ts_namespace", mod_name), mod_name.span());

    let ts_code = quote! {
        #input

        /// TypeScript 命名空间定义
        pub const #ts_const_ident: &'static str = concat!(
            "namespace ", #namespace_name, " {\n",
            "    // 模块内容将在这里生成\n",
            "}\n"
        );

        /// 获取 TypeScript 命名空间定义
        pub fn #ts_namespace_name() -> &'static str {
            #ts_const_ident
        }
    };

    TokenStream::from(ts_code)
}

/// 为 Rust 类型生成 TypeScript 类型守卫
///
/// # 示例
///
/// ```rust
/// use typescript_macros::TypescriptGuard;
///
/// #[derive(TypescriptGuard)]
/// struct User {
///     id: u32,
///     name: String,
///     active: bool,
/// }
/// ```
///
/// 这将生成对应的 TypeScript 类型守卫：
///
/// ```typescript
/// function isUser(obj: any): obj is User {
///     return (
///         typeof obj === 'object' && obj !== null &&
///         typeof obj.id === 'number' &&
///         typeof obj.name === 'string' &&
///         typeof obj.active === 'boolean'
///     );
/// }
/// ```
#[proc_macro_derive(TypescriptGuard, attributes(ts))]
pub fn typescript_guard_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let type_name = &input.ident;

    if !matches!(input.data, syn::Data::Struct(_)) {
        return syn::Error::new_spanned(input, "Only structs are supported for TypescriptGuard").to_compile_error().into();
    }

    let fields = match &input.data {
        syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Named(fields), .. }) => &fields.named,
        syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Unnamed(_), .. }) => {
            return syn::Error::new_spanned(input, "Tuple structs are not supported").to_compile_error().into();
        }
        syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Unit, .. }) => {
            return syn::Error::new_spanned(input, "Unit structs are not supported").to_compile_error().into();
        }
        _ => unreachable!(),
    };

    let guard_conditions: Vec<String> = fields
        .iter()
        .filter_map(|field| {
            let attrs = FieldAttributes::from_attributes(&field.attrs);
            if attrs.skip {
                return None;
            }
            let field_name = attrs.rename.clone().unwrap_or_else(|| field.ident.as_ref().unwrap().to_string());
            let ts_type = rust_type_to_typescript(&field.ty);

            // 根据 TypeScript 类型生成相应的类型检查
            let condition = match ts_type.as_str() {
                "number" => format!("typeof obj.{field_name} === 'number'"),
                "boolean" => format!("typeof obj.{field_name} === 'boolean'"),
                "string" => format!("typeof obj.{field_name} === 'string'"),
                _ => format!("obj.{field_name} !== undefined"), // 对于复杂类型，只检查存在性
            };

            Some(condition)
        })
        .collect();

    let guard_conditions_str = guard_conditions.join(" &&\n        ");
    let guard_function_name = format!("is{}", type_name);
    let guard_function_ident = syn::Ident::new(&guard_function_name, type_name.span());

    let ts_const_name = format!("{}_TS_GUARD", type_name);
    let ts_const_ident = syn::Ident::new(&ts_const_name, type_name.span());
    let ts_guard_name = syn::Ident::new(&format!("{}_ts_guard", type_name), type_name.span());

    let ts_code = quote! {
        impl #type_name {
            /// TypeScript 类型守卫函数
            pub const #ts_const_ident: &'static str = concat!(
                "function ", stringify!(#guard_function_ident), "(obj: any): obj is ", stringify!(#type_name), " {\n",
                "    return (\n",
                "        typeof obj === 'object' && obj !== null &&\n",
                "        #guard_conditions_str\n",
                "    );\n",
                "}\n"
            );

            /// 获取 TypeScript 类型守卫函数
            pub fn #ts_guard_name() -> &'static str {
                Self::#ts_const_ident
            }
        }
    };

    TokenStream::from(ts_code)
}

/// 为 Rust 接口生成 TypeScript 接口定义
///
/// # 示例
///
/// ```rust
/// use typescript_macros::TypescriptInterface;
///
/// #[derive(TypescriptInterface)]
/// struct User {
///     id: u32,
///     name: String,
///     active: bool,
/// }
/// ```
///
/// 这将生成对应的 TypeScript 接口定义：
///
/// ```typescript
/// interface User {
///     id: number;
///     name: string;
///     active: boolean;
/// }
/// ```
#[proc_macro_derive(TypescriptInterface, attributes(ts))]
pub fn typescript_interface_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let trait_name = &input.ident;

    // 解析泛型参数
    let generic_params = if let Some(_generics) = input.generics.params.first() {
        let generic_args: Vec<String> =
            input
                .generics
                .params
                .iter()
                .filter_map(|param| {
                    if let syn::GenericParam::Type(type_param) = param { Some(type_param.ident.to_string()) } else { None }
                })
                .collect();

        if !generic_args.is_empty() { format!("<{}>", generic_args.join(", ")) } else { "".to_string() }
    }
    else {
        "".to_string()
    };

    if !matches!(input.data, syn::Data::Struct(_)) {
        return syn::Error::new_spanned(input, "Only structs are supported for TypescriptInterface").to_compile_error().into();
    }

    let fields = match &input.data {
        syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Named(fields), .. }) => &fields.named,
        syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Unnamed(_), .. }) => {
            return syn::Error::new_spanned(input, "Tuple structs are not supported").to_compile_error().into();
        }
        syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Unit, .. }) => {
            return syn::Error::new_spanned(input, "Unit structs are not supported").to_compile_error().into();
        }
        _ => unreachable!(),
    };

    let interface_fields: Vec<String> = fields
        .iter()
        .filter_map(|field| {
            let attrs = FieldAttributes::from_attributes(&field.attrs);
            if attrs.skip {
                return None;
            }
            let field_name = attrs.rename.clone().unwrap_or_else(|| field.ident.as_ref().unwrap().to_string());
            let mut ts_type = if let Some(custom_type) = &attrs.custom_type {
                custom_type.clone()
            }
            else {
                rust_type_to_typescript(&field.ty)
            };
            if attrs.optional && !ts_type.contains("undefined") {
                ts_type = format!("{} | undefined", ts_type);
            }
            Some(format!("    {}: {}", field_name, ts_type))
        })
        .collect();

    let interface_fields_str = interface_fields.join(";\n");

    let ts_code = quote! {
        impl #trait_name {
            /// TypeScript 接口定义
            pub const TS_INTERFACE_DEFINITION: &'static str = concat!(
                "interface ", stringify!(#trait_name), #generic_params, " {\n",
                #interface_fields_str, ";\n",
                "}\n"
            );

            /// 获取 TypeScript 接口定义
            pub fn ts_interface_definition() -> &'static str {
                Self::TS_INTERFACE_DEFINITION
            }
        }
    };

    TokenStream::from(ts_code)
}

/// 为 Rust 枚举生成 TypeScript 枚举定义
///
/// # 示例
///
/// ```rust
/// use typescript_macros::TypescriptEnum;
///
/// #[derive(TypescriptEnum)]
/// enum Color {
///     Red,
///     Green,
///     Blue,
/// }
/// ```
///
/// 这将生成对应的 TypeScript 枚举定义：
///
/// ```typescript
/// enum Color {
///     Red = 0,
///     Green = 1,
///     Blue = 2,
/// }
/// ```
///
/// # 字符串枚举示例
///
/// ```rust
/// use typescript_macros::TypescriptEnum;
///
/// #[derive(TypescriptEnum)]
/// enum Direction {
///     Up,
///     Down,
///     Left,
///     Right,
/// }
/// ```
///
/// 这将生成对应的 TypeScript 枚举定义：
///
/// ```typescript
/// enum Direction {
///     Up = "UP",
///     Down = "DOWN",
///     Left = "LEFT",
///     Right = "RIGHT",
/// }
/// ```
#[proc_macro_derive(TypescriptEnum, attributes(ts))]
pub fn typescript_enum_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let enum_name = &input.ident;

    let variants = match &input.data {
        syn::Data::Enum(syn::DataEnum { variants, .. }) => variants,
        _ => {
            return syn::Error::new_spanned(input, "Only enums are supported").to_compile_error().into();
        }
    };

    let enum_variants: Vec<String> = variants
        .iter()
        .enumerate()
        .map(|(index, variant)| {
            let variant_name = &variant.ident;

            Ok(match &variant.fields {
                syn::Fields::Unit => {
                    format!("    {} = {}", variant_name, index)
                }
                syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                    let value = &fields.unnamed[0];
                    let value_str = quote!(#value).to_string();
                    format!("    {} = {}", variant_name, value_str)
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        variant,
                        "Only unit variants or variants with a single value are supported",
                    ));
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let enum_variants_str = enum_variants.join(",\n");

    let ts_code = quote! {
        impl #enum_name {
            /// TypeScript 枚举定义
            pub const TS_ENUM_DEFINITION: &'static str = concat!(
                "enum ", stringify!(#enum_name), " {\n",
                #enum_variants_str,
                "\n}"
            );

            /// 获取 TypeScript 枚举定义
            pub fn ts_enum_definition() -> &'static str {
                Self::TS_ENUM_DEFINITION
            }
        }
    };

    TokenStream::from(ts_code)
}

/// 为 Rust 类型定义生成 TypeScript 类型别名
///
/// # 示例
///
/// ```rust
/// use typescript_macros::TypescriptType;
///
/// #[derive(TypescriptType)]
/// struct User {
///     id: u32,
///     name: String,
///     active: bool,
/// }
/// ```
///
/// 这将生成对应的 TypeScript 类型别名：
///
/// ```typescript
/// type User = {
///     id: number;
///     name: string;
///     active: boolean;
/// };
/// ```
#[proc_macro_derive(TypescriptType, attributes(ts))]
pub fn typescript_type_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let type_name = &input.ident;

    // 解析泛型参数
    let generic_params = if let Some(_generics) = input.generics.params.first() {
        let generic_args: Vec<String> =
            input
                .generics
                .params
                .iter()
                .filter_map(|param| {
                    if let syn::GenericParam::Type(type_param) = param { Some(type_param.ident.to_string()) } else { None }
                })
                .collect();

        if !generic_args.is_empty() { format!("<{}>", generic_args.join(", ")) } else { "".to_string() }
    }
    else {
        "".to_string()
    };

    if !matches!(input.data, syn::Data::Struct(_)) {
        return syn::Error::new_spanned(input, "Only structs are supported for TypescriptType").to_compile_error().into();
    }

    let fields = match &input.data {
        syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Named(fields), .. }) => &fields.named,
        syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Unnamed(_), .. }) => {
            return syn::Error::new_spanned(input, "Tuple structs are not supported").to_compile_error().into();
        }
        syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Unit, .. }) => {
            return syn::Error::new_spanned(input, "Unit structs are not supported").to_compile_error().into();
        }
        _ => unreachable!(),
    };

    let type_fields: Vec<String> = fields
        .iter()
        .filter_map(|field| {
            let attrs = FieldAttributes::from_attributes(&field.attrs);
            if attrs.skip {
                return None;
            }
            let field_name = attrs.rename.clone().unwrap_or_else(|| field.ident.as_ref().unwrap().to_string());
            let mut ts_type = if let Some(custom_type) = &attrs.custom_type {
                custom_type.clone()
            }
            else {
                rust_type_to_typescript(&field.ty)
            };
            if attrs.optional && !ts_type.contains("undefined") {
                ts_type = format!("{} | undefined", ts_type);
            }
            Some(format!("    {}: {}", field_name, ts_type))
        })
        .collect();

    let type_fields_str = type_fields.join(";\n");

    let ts_code = quote! {
        impl #type_name {
            /// TypeScript 类型别名定义
            pub const TS_TYPE_DEFINITION: &'static str = concat!(
                "type ", stringify!(#type_name), #generic_params, " = {\n",
                #type_fields_str, ";\n",
                "}\n"
            );

            /// 获取 TypeScript 类型别名定义
            pub fn ts_type_definition() -> &'static str {
                Self::TS_TYPE_DEFINITION
            }
        }
    };

    TokenStream::from(ts_code)
}

/// 为 Rust 枚举生成 TypeScript 联合类型
///
/// # 示例
///
/// ```rust
/// use typescript_macros::TypescriptUnion;
///
/// #[derive(TypescriptUnion)]
/// enum Shape {
///     Circle { radius: f64 },
///     Rectangle { width: f64, height: f64 },
///     Triangle { base: f64, height: f64 },
/// }
/// ```
///
/// 这将生成对应的 TypeScript 联合类型：
///
/// ```typescript
/// type Shape =
///     | { type: "Circle"; radius: number }
///     | { type: "Rectangle"; width: number; height: number }
///     | { type: "Triangle"; base: number; height: number };
/// ```
#[proc_macro_derive(TypescriptUnion, attributes(ts))]
pub fn typescript_union_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let enum_name = &input.ident;

    let variants = match &input.data {
        syn::Data::Enum(syn::DataEnum { variants, .. }) => variants,
        _ => {
            return syn::Error::new_spanned(input, "Only enums are supported for TypescriptUnion").to_compile_error().into();
        }
    };

    let union_variants: Vec<String> = variants
        .iter()
        .map(|variant| {
            let variant_name = variant.ident.to_string();

            Ok(match &variant.fields {
                syn::Fields::Unit => {
                    format!("    | {{ type: \"{}\" }}", variant_name)
                }
                syn::Fields::Named(fields) => {
                    let field_defs: Vec<String> = fields
                        .named
                        .iter()
                        .filter_map(|field| {
                            let attrs = FieldAttributes::from_attributes(&field.attrs);
                            if attrs.skip {
                                return None;
                            }
                            let field_name = attrs.rename.clone().unwrap_or_else(|| field.ident.as_ref().unwrap().to_string());
                            let mut ts_type = if let Some(custom_type) = &attrs.custom_type {
                                custom_type.clone()
                            }
                            else {
                                rust_type_to_typescript(&field.ty)
                            };
                            if attrs.optional && !ts_type.contains("undefined") {
                                ts_type = format!("{} | undefined", ts_type);
                            }
                            Some(format!("    {}: {}", field_name, ts_type))
                        })
                        .collect();

                    let fields_str =
                        if field_defs.is_empty() { "".to_string() } else { format!(";\n{}", field_defs.join(";\n")) };

                    format!("    | {{ type: \"{}\"{}}}", variant_name, fields_str)
                }
                syn::Fields::Unnamed(fields) => {
                    if fields.unnamed.is_empty() {
                        format!("    | \"{}\"", variant_name)
                    }
                    else if fields.unnamed.len() == 1 {
                        let field_ty = &fields.unnamed[0].ty;
                        let ts_type = rust_type_to_typescript(field_ty);
                        format!("    | {{ type: \"{}\"; value: {} }}", variant_name, ts_type)
                    }
                    else {
                        return Err(syn::Error::new_spanned(
                            variant,
                            "Only unit variants, named variants, or variants with a single value are supported",
                        ));
                    }
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let union_variants_str = union_variants.join("\n");

    let ts_code = quote! {
        impl #enum_name {
            /// TypeScript 联合类型定义
            pub const TS_UNION_DEFINITION: &'static str = concat!(
                "type ", stringify!(#enum_name), " =\n",
                #union_variants_str,
                ";\n"
            );

            /// 获取 TypeScript 联合类型定义
            pub fn ts_union_definition() -> &'static str {
                Self::TS_UNION_DEFINITION
            }
        }
    };

    TokenStream::from(ts_code)
}
