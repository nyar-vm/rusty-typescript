#![warn(missing_docs)]

use oak_core::builder::Builder;
use oak_typescript::{TypeScriptBuilder, TypeScriptLanguage};
use std::rc::Rc;
use typescript_ir::Program;
use typescript_types::{ToTsValue, TsError, TsValue};

mod codegen;
mod ffi;
mod gc;
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
        let mut cache = oak_core::parser::ParseSession::<oak_typescript::TypeScriptLanguage>::default();
        let edits: &[oak_core::TextEdit] = &[];
        let build_result = builder.build(script, edits, &mut cache);
        let ast = build_result.result.map_err(|e| TsError::SyntaxError(e.to_string()))?;
        println!("Step 2: AST to IR conversion");

        // 将 oak-typescript AST 转换为 IR
        let ir = typescript_ir::ast_to_ir::program(&ast);
        println!("Step 3: VM execution");

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
    let mut cache = oak_core::parser::ParseSession::<oak_typescript::TypeScriptLanguage>::default();
    let edits: &[oak_core::TextEdit] = &[];
    let build_result = builder.build(source, edits, &mut cache);

    match build_result.result {
        Ok(ast) => {
            println!("AST: {:?}", ast);

            // 转换为 IR 并打印
            let ir = typescript_ir::ast_to_ir::program(&ast);
            println!("IR: {:?}", ir);
        }
        Err(error) => {
            println!("Build error: {:?}", error);
        }
    }
}
