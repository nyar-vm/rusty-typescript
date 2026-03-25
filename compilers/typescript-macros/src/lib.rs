#![warn(missing_docs)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, ItemFn, parse_macro_input};

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

    // 解析结构体名称
    let struct_name = &input.ident;

    // 解析结构体字段
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

    // 检查字段是否都有名称
    for field in fields {
        if field.ident.is_none() {
            return syn::Error::new_spanned(field, "All fields must have names").to_compile_error().into();
        }
    }

    // 生成字段类型映射
    let field_types: Vec<String> = fields
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();
            let field_ty = &field.ty;

            // 简单的类型映射
            let ts_type = match quote!(#field_ty).to_string().as_str() {
                "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" | "f32" | "f64" => "number",
                "bool" => "boolean",
                "String" | "&str" => "string",
                "Option<u8>" | "Option<u16>" | "Option<u32>" | "Option<u64>" | "Option<i8>" | "Option<i16>" | "Option<i32>"
                | "Option<i64>" | "Option<f32>" | "Option<f64>" => "number | undefined",
                "Option<bool>" => "boolean | undefined",
                "Option<String>" | "Option<&str>" => "string | undefined",
                _ => "any",
            };

            format!("    {}: {}", field_name, ts_type)
        })
        .collect();

    // 生成构造函数参数
    let constructor_params: Vec<String> = fields
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();
            let field_ty = &field.ty;

            let ts_type = match quote!(#field_ty).to_string().as_str() {
                "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" | "f32" | "f64" => "number",
                "bool" => "boolean",
                "String" | "&str" => "string",
                "Option<u8>" | "Option<u16>" | "Option<u32>" | "Option<u64>" | "Option<i8>" | "Option<i16>" | "Option<i32>"
                | "Option<i64>" | "Option<f32>" | "Option<f64>" => "number | undefined",
                "Option<bool>" => "boolean | undefined",
                "Option<String>" | "Option<&str>" => "string | undefined",
                _ => "any",
            };

            format!("        {}: {}", field_name, ts_type)
        })
        .collect();

    // 生成构造函数赋值
    let constructor_assignments: Vec<String> = fields
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();
            format!("        this.{} = {};", field_name, field_name)
        })
        .collect();

    // 生成 TypeScript 代码
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
pub fn TypescriptFunction(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    // 解析函数名称
    let fn_name = &input.sig.ident;

    // 解析函数参数
    let params = &input.sig.inputs;
    let param_types: Vec<String> = params
        .iter()
        .map(|param| match param {
            syn::FnArg::Typed(pat_type) => {
                let param_name = match &*pat_type.pat {
                    syn::Pat::Ident(pat_ident) => &pat_ident.ident,
                    _ => return "_: any".to_string(),
                };
                let param_ty = &pat_type.ty;

                let ts_type = match quote!(#param_ty).to_string().as_str() {
                    "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" | "f32" | "f64" => "number",
                    "bool" => "boolean",
                    "String" | "&str" => "string",
                    "Option<u8>" | "Option<u16>" | "Option<u32>" | "Option<u64>" | "Option<i8>" | "Option<i16>"
                    | "Option<i32>" | "Option<i64>" | "Option<f32>" | "Option<f64>" => "number | undefined",
                    "Option<bool>" => "boolean | undefined",
                    "Option<String>" | "Option<&str>" => "string | undefined",
                    _ => "any",
                };

                format!("{}: {}", param_name, ts_type)
            }
            _ => "_: any".to_string(),
        })
        .collect();

    // 解析返回类型
    let return_type = match &input.sig.output {
        syn::ReturnType::Type(_, ty) => match quote!(#ty).to_string().as_str() {
            "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" | "f32" | "f64" => "number",
            "bool" => "boolean",
            "String" | "&str" => "string",
            "Option<u8>" | "Option<u16>" | "Option<u32>" | "Option<u64>" | "Option<i8>" | "Option<i16>" | "Option<i32>"
            | "Option<i64>" | "Option<f32>" | "Option<f64>" => "number | undefined",
            "Option<bool>" => "boolean | undefined",
            "Option<String>" | "Option<&str>" => "string | undefined",
            _ => "any",
        },
        syn::ReturnType::Default => "void",
    };

    // 生成 TypeScript 函数类型定义
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

    // 解析接口名称
    let trait_name = &input.ident;

    // 检查是否是 trait
    if !matches!(input.data, syn::Data::Struct(_)) {
        return syn::Error::new_spanned(input, "Only structs are supported for TypescriptInterface").to_compile_error().into();
    }

    // 对于结构体，我们将其视为接口
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

    // 生成接口字段
    let interface_fields: Vec<String> = fields
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();
            let field_ty = &field.ty;

            // 简单的类型映射
            let ts_type = match quote!(#field_ty).to_string().as_str() {
                "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" | "f32" | "f64" => "number",
                "bool" => "boolean",
                "String" | "&str" => "string",
                "Option<u8>" | "Option<u16>" | "Option<u32>" | "Option<u64>" | "Option<i8>" | "Option<i16>" | "Option<i32>"
                | "Option<i64>" | "Option<f32>" | "Option<f64>" => "number | undefined",
                "Option<bool>" => "boolean | undefined",
                "Option<String>" | "Option<&str>" => "string | undefined",
                _ => "any",
            };

            format!("    {}: {}", field_name, ts_type)
        })
        .collect();

    // 生成 TypeScript 接口定义
    let interface_fields_str = interface_fields.join(";\n");

    let ts_code = quote! {
        impl #trait_name {
            /// TypeScript 接口定义
            pub const TS_INTERFACE_DEFINITION: &'static str = concat!(
                "interface ", stringify!(#trait_name), " {\n",
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
///     Up = "UP",
///     Down = "DOWN",
///     Left = "LEFT",
///     Right = "RIGHT",
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

    // 解析枚举名称
    let enum_name = &input.ident;

    // 解析枚举变体
    let variants = match &input.data {
        syn::Data::Enum(syn::DataEnum { variants, .. }) => variants,
        _ => {
            return syn::Error::new_spanned(input, "Only enums are supported").to_compile_error().into();
        }
    };

    // 生成枚举变体
    let enum_variants: Vec<String> = variants
        .iter()
        .enumerate()
        .map(|(index, variant)| {
            let variant_name = &variant.ident;

            // 检查是否有显式值
            Ok(match &variant.fields {
                syn::Fields::Unit => {
                    // 无显式值，使用索引作为值
                    format!("    {} = {}", variant_name, index)
                }
                syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                    // 有显式值
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

    // 生成 TypeScript 枚举定义
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

    // 解析类型名称
    let type_name = &input.ident;

    // 检查是否是结构体
    if !matches!(input.data, syn::Data::Struct(_)) {
        return syn::Error::new_spanned(input, "Only structs are supported for TypescriptType").to_compile_error().into();
    }

    // 对于结构体，我们将其视为类型别名
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

    // 生成类型字段
    let type_fields: Vec<String> = fields
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();
            let field_ty = &field.ty;

            // 简单的类型映射
            let ts_type = match quote!(#field_ty).to_string().as_str() {
                "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" | "f32" | "f64" => "number",
                "bool" => "boolean",
                "String" | "&str" => "string",
                "Option<u8>" | "Option<u16>" | "Option<u32>" | "Option<u64>" | "Option<i8>" | "Option<i16>" | "Option<i32>"
                | "Option<i64>" | "Option<f32>" | "Option<f64>" => "number | undefined",
                "Option<bool>" => "boolean | undefined",
                "Option<String>" | "Option<&str>" => "string | undefined",
                _ => "any",
            };

            format!("    {}: {}", field_name, ts_type)
        })
        .collect();

    // 生成 TypeScript 类型别名定义
    let type_fields_str = type_fields.join(";\n");

    let ts_code = quote! {
        impl #type_name {
            /// TypeScript 类型别名定义
            pub const TS_TYPE_DEFINITION: &'static str = concat!(
                "type ", stringify!(#type_name), " = {\n",
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
