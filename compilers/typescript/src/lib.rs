#![warn(missing_docs)]

use oak_core::builder::Builder;
use oak_typescript::{TypeScriptBuilder, TypeScriptLanguage};
use std::rc::Rc;
use typescript_ir::Program;
use typescript_types::{ToTsValue, TsError, TsValue};

use crate::language::TypeScriptLanguage as CustomTypeScriptLanguage;

mod codegen;
pub mod ffi;
mod gc;
pub mod language;
pub mod platform;
mod type_checker;
mod vm;

/// TypeScript 运行时环境
pub struct TypeScript {
    /// 全局变量
    globals: Vec<(String, TsValue)>,
    /// 内存管理器
    memory_manager: gc::MemoryManager,
    /// FFI 管理器
    ffi_manager: ffi::FfiManager,
}

impl TypeScript {
    /// 创建一个新的 TypeScript 运行时环境
    pub fn new() -> Self {
        let mut globals = vec![];

        // 添加全局函数
        let log_fn = Rc::new(|args: &[TsValue]| {
            for arg in args {
                println!("{}", arg.to_string());
            }
            TsValue::Undefined
        });
        let console_obj = TsValue::Object(vec![("log".to_string(), TsValue::Function(log_fn))]);
        globals.push(("console".to_string(), console_obj));

        // 打印全局变量初始化信息
        println!("Initialized globals: {:?}", globals);

        // 初始化内存管理器
        let memory_manager = gc::MemoryManager::new(1000);

        // 初始化 FFI 管理器
        let mut ffi_manager = ffi::FfiManager::new();
        ffi_manager.register_std_functions();

        Self { globals, memory_manager, ffi_manager }
    }

    /// 执行 TypeScript 脚本
    pub fn execute_script(&mut self, script: &str) -> Result<TsValue, TsError> {
        println!("Step 1: Lexical analysis and Syntax analysis");
        // 使用 oak-typescript 进行词法分析和语法分析
        let language = TypeScriptLanguage::new();
        let builder = TypeScriptBuilder::new(&language);
        let mut cache = oak_core::parser::ParseSession::<TypeScriptLanguage>::default();
        let edits: &[oak_core::TextEdit] = &[];
        let build_result = builder.build(script, edits, &mut cache);
        let ast = build_result.result.map_err(|e| TsError::SyntaxError(e.to_string()))?;
        println!("Step 2: AST to IR conversion");

        // 将 oak-typescript AST 转换为 IR
        let mut ir = typescript_ir::ast_to_ir::program(&ast)?;
        println!("Step 3: IR optimization");

        // 优化 IR
        ir = typescript_ir::optimize_full(&ir);
        println!("Step 4: Type checking");

        // 类型检查
        ir = type_checker::check(ir)?;
        println!("Step 5: VM execution");

        // 虚拟机执行
        let mut vm = vm::VM::new(self.globals.clone());
        vm.execute(&ir)
    }
}

/// 从 Rust 类型转换为 TypeScript 值的 trait 实现扩展
impl ToTsValue for TypeScript {
    fn to_ts_value(&self) -> TsValue {
        TsValue::Object(self.globals.clone())
    }
}

/// 测试函数，用于调试词法分析器和解析器
pub fn test_lexer_parser() {
    let source = "let x = 10; let y = 'hello'; console.log(x, y); x + y";
    println!("Source: {}", source);

    // 使用 oak-typescript 进行词法分析和语法分析
    let language = TypeScriptLanguage::new();
    let builder = TypeScriptBuilder::new(&language);
    let mut cache = oak_core::parser::ParseSession::<TypeScriptLanguage>::default();
    let edits: &[oak_core::TextEdit] = &[];
    let build_result = builder.build(source, edits, &mut cache);

    match build_result.result {
        Ok(ast) => {
            println!("AST: {:?}", ast);

            // 转换为 IR 并打印
            match typescript_ir::ast_to_ir::program(&ast) {
                Ok(ir) => println!("IR: {:?}", ir),
                Err(error) => println!("IR conversion error: {:?}", error),
            }
        }
        Err(error) => {
            println!("Build error: {:?}", error);
        }
    }
}

/// 创建 TypeScript 运行时实例
///
/// 这是一个工厂函数，用于创建并返回一个新的 TypeScript 实例。
///
/// # 返回
/// 返回一个新创建的 TypeScript 实例
pub fn create_runtime() -> TypeScript {
    TypeScript::new()
}

/// 执行 TypeScript 脚本
///
/// 这是一个便捷函数，直接在给定的运行时实例上执行脚本。
///
/// # 参数
/// - `runtime`: TypeScript 运行时实例的可变引用
/// - `script`: 要执行的 TypeScript 脚本字符串
///
/// # 返回
/// 返回脚本执行结果的字符串表示，或错误信息
pub fn run_script(runtime: &mut TypeScript, script: &str) -> Result<String, TsError> {
    match runtime.execute_script(script) {
        Ok(value) => Ok(value.to_string()),
        Err(error) => Err(error),
    }
}
