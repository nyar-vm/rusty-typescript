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
        _ => {
            return syn::Error::new_spanned(input, "Only named fields structs are supported").to_compile_error().into();
        }
    };

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
            _ => "any",
        },
        syn::ReturnType::Default => "void",
    };

    // 生成 TypeScript 函数类型定义
    let param_types_str = param_types.join(", ");
    let ts_const_name = format!("{}_TS_FUNCTION_DEFINITION", fn_name);
    let ts_const_ident = syn::Ident::new(&ts_const_name, fn_name.span());

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
    };

    TokenStream::from(ts_code)
}
