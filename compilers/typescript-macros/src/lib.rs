#![warn(missing_docs)]

//! TypeScript macros for Rust
//! 
//! This crate provides procedural macros for generating TypeScript type definitions
//! from Rust code, enabling seamless type-safe interaction between Rust and TypeScript.

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, ItemFn, Type, parse_macro_input};

/// 将 Rust 类型转换为 TypeScript 类型字符串
///
/// 支持的类型映射：
/// - 基本类型：u8, u16, u32, u64, i8, i16, i32, i64, f32, f64 -> number
/// - 布尔类型：bool -> boolean
/// - 字符串：String, &str -> string
/// - 可选类型：Option<T> -> T | undefined
/// - 集合类型：Vec<T> -> T[], HashSet<T>, BTreeSet<T> -> Set<T>
/// - 映射类型：HashMap<K, V>, BTreeMap<K, V> -> Record<K, V>
/// - 元组类型：(A, B, C) -> [A, B, C]
/// - 嵌套类型：递归处理内部类型
fn rust_type_to_typescript(ty: &Type) -> String {
    let type_str = quote!(#ty).to_string();
    
    if let Type::Path(type_path) = ty {
        let path = &type_path.path;
        
        if let Some(segment) = path.segments.last() {
            let ident = segment.ident.to_string();
            
            match ident.as_str() {
                "Option" => {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            let inner_ts = rust_type_to_typescript(inner_ty);
                            return format!("({}) | undefined", inner_ts);
                        }
                    }
                    return "any | undefined".to_string();
                }
                "Vec" => {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            let inner_ts = rust_type_to_typescript(inner_ty);
                            return format!("({})[]", inner_ts);
                        }
                    }
                    return "any[]".to_string();
                }
                "HashSet" | "BTreeSet" => {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            let inner_ts = rust_type_to_typescript(inner_ty);
                            return format!("Set<{}>", inner_ts);
                        }
                    }
                    return "Set<any>".to_string();
                }
                "HashMap" | "BTreeMap" => {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        let args_vec: Vec<_> = args.args.iter().collect();
                        if args_vec.len() == 2 {
                            let key_ts = if let syn::GenericArgument::Type(key_ty) = args_vec[0] {
                                rust_type_to_typescript(key_ty)
                            } else {
                                "any".to_string()
                            };
                            let value_ts = if let syn::GenericArgument::Type(value_ty) = args_vec[1] {
                                rust_type_to_typescript(value_ty)
                            } else {
                                "any".to_string()
                            };
                            return format!("Record<{}, {}>", key_ts, value_ts);
                        }
                    }
                    return "Record<string, any>".to_string();
                }
                _ => {}
            }
        }
    }
    
    if let Type::Tuple(type_tuple) = ty {
        if type_tuple.elems.is_empty() {
            return "null".to_string();
        }
        let elements: Vec<String> = type_tuple.elems.iter()
            .map(|elem| rust_type_to_typescript(elem))
            .collect();
        return format!("[{}]", elements.join(", "));
    }
    
    match type_str.as_str() {
        "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" | "f32" | "f64" => "number".to_string(),
        "bool" => "boolean".to_string(),
        "String" | "& str" => "string".to_string(),
        _ => "any".to_string(),
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
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();
            let ts_type = rust_type_to_typescript(&field.ty);
            format!("    {}: {}", field_name, ts_type)
        })
        .collect();

    let constructor_params: Vec<String> = fields
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();
            let ts_type = rust_type_to_typescript(&field.ty);
            format!("        {}: {}", field_name, ts_type)
        })
        .collect();

    let constructor_assignments: Vec<String> = fields
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();
            format!("        this.{} = {};", field_name, field_name)
        })
        .collect();

    let field_types_str = field_types.join(";\n");
    let constructor_params_str = constructor_params.join(",\n");
    let constructor_assignments_str = constructor_assignments.join("\n");

    let ts_code = quote! {
        impl #struct_name {
            /// TypeScript 类定义
            pub const TS_CLASS_DEFINITION: &'static str = concat!(
                "class ", stringify!(#struct_name), " {\n",
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
/// use typescript_macros::TypescriptFunction;
///
/// #[TypescriptFunction]
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

    let param_types_str =