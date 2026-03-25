#![warn(missing_docs)]

use std::rc::Rc;
use typescript_ir::Program;
use typescript_types::{ToTsValue, TsError, TsValue};

use crate::compiler::{Compiler, cache::CompilationCache, context::CompilationContext, main::MainCompiler};

/// 编译选项
#[derive(Debug, Clone, Default)]
pub struct CompileOptions {
    /// 是否启用严格模式
    pub strict: bool,
    /// 是否启用 JIT 编译
    pub enable_jit: bool,
    /// 是否启用类型检查
    pub enable_type_checking: bool,
    /// 是否启用性能监控
    pub enable_performance_monitoring: bool,
    /// 是否启用垃圾回收
    pub enable_garbage_collection: bool,
    /// 目标 ES 版本
    pub target_es_version: String,
    /// 模块解析策略
    pub module_resolution: String,
    /// 是否生成源映射
    pub source_map: bool,
}

mod codegen;
pub mod compiler;
pub mod ffi;
pub mod gc;
pub mod jit;
pub mod language;
pub mod memory;
pub mod platform;
mod type_checker;
mod vm;

/// TypeScript 运行时环境
pub struct TypeScript {
    /// 全局变量
    globals: std::collections::HashMap<String, TsValue>,
    /// 内存管理器
    memory_manager: gc::MemoryManager,
    /// FFI 管理器
    ffi_manager: ffi::FfiManager,
    /// JIT 执行器
    jit_executor: jit::JITExecutor,
    /// 主编译器
    compiler: MainCompiler,
    /// 模块缓存
    modules: std::collections::HashMap<String, typescript_types::TsValue>,
    /// 编译选项
    compile_options: CompileOptions,
}

impl TypeScript {
    /// 创建一个新的 TypeScript 运行时环境
    pub fn new() -> Self {
        use std::collections::HashMap;
        let mut globals = HashMap::new();

        // 添加全局函数
        let log_fn = Rc::new(|args: &[TsValue]| {
            for arg in args {
                println!("{}", arg.to_string());
            }
            TsValue::Undefined
        });
        let console_obj = TsValue::Object(HashMap::from([("log".to_string(), TsValue::Function(log_fn))]));
        globals.insert("console".to_string(), console_obj);

        // 打印全局变量初始化信息
        println!("Initialized globals: {:?}", globals);

        // 初始化内存管理器
        let memory_manager = gc::MemoryManager::new(1000);

        // 初始化 FFI 管理器
        let mut ffi_manager = ffi::FfiManager::new();
        ffi_manager.register_std_functions();

        // 初始化 JIT 执行器
        let jit_executor = jit::JITExecutor::new();

        // 初始化编译上下文和缓存
        let context = CompilationContext::default();
        let cache = CompilationCache::default();

        // 初始化主编译器
        let compiler = MainCompiler::new(context, cache);

        // 初始化模块缓存
        let modules = HashMap::new();

        // 初始化编译选项
        let compile_options = CompileOptions {
            strict: false,
            enable_jit: true,
            enable_type_checking: true,
            enable_performance_monitoring: true,
            enable_garbage_collection: true,
            target_es_version: "es2020".to_string(),
            module_resolution: "node".to_string(),
            source_map: false,
        };

        Self { globals, memory_manager, ffi_manager, jit_executor, compiler, modules, compile_options }
    }

    /// 执行 TypeScript 脚本
    pub fn execute_script(&mut self, script: &str) -> Result<TsValue, TsError> {
        // 使用新的编译架构执行脚本
        match self.compiler.compile(script) {
            crate::compiler::CompilationResult::Success(value) => Ok(value),
            crate::compiler::CompilationResult::Error(error) => Err(error),
            _ => Err(TsError::Other("Compilation failed".to_string())),
        }
    }

    /// 模块管理 API

    /// 导入模块
    ///
    /// # 参数
    /// - `name`: 模块名称
    /// - `alias`: 模块别名（可选）
    ///
    /// # 返回值
    /// - `Ok(())` 导入成功，`Err(TsError)` 导入失败
    pub fn import_module(&mut self, name: &str, alias: Option<&str>) -> Result<(), TsError> {
        // 检查模块是否已缓存
        if self.modules.contains_key(name) {
            return Ok(());
        }

        // 模拟模块加载
        let module_name = alias.unwrap_or(name);
        let module_value = TsValue::Object(std::collections::HashMap::new());

        // 缓存模块
        self.modules.insert(name.to_string(), module_value.clone());
        self.globals.insert(module_name.to_string(), module_value);

        Ok(())
    }

    /// 导出模块
    ///
    /// # 参数
    /// - `name`: 模块名称
    /// - `value`: 模块值
    pub fn export_module(&mut self, name: &str, value: TsValue) {
        self.modules.insert(name.to_string(), value);
    }

    /// 获取模块
    ///
    /// # 参数
    /// - `name`: 模块名称
    ///
    /// # 返回值
    /// - 模块值（如果存在）
    pub fn get_module(&self, name: &str) -> Option<&TsValue> {
        self.modules.get(name)
    }

    /// 类型系统 API

    /// 注册类型
    ///
    /// # 参数
    /// - `name`: 类型名称
    /// - `definition`: 类型定义
    pub fn register_type(&mut self, name: &str, definition: TsValue) {
        self.globals.insert(format!("__type_{}", name), definition);
    }

    /// 获取类型
    ///
    /// # 参数
    /// - `name`: 类型名称
    ///
    /// # 返回值
    /// - 类型定义（如果存在）
    pub fn get_type(&self, name: &str) -> Option<&TsValue> {
        self.globals.get(&format!("__type_{}", name))
    }

    /// 性能监控 API

    /// 启用性能监控
    pub fn enable_performance_monitoring(&mut self) {
        self.compile_options.enable_performance_monitoring = true;
    }

    /// 禁用性能监控
    pub fn disable_performance_monitoring(&mut self) {
        self.compile_options.enable_performance_monitoring = false;
    }

    /// 获取性能报告
    ///
    /// # 返回值
    /// - 性能报告字符串
    pub fn get_performance_report(&self) -> String {
        "Performance report not implemented".to_string()
    }

    /// 内存管理 API

    /// 执行垃圾回收
    pub fn garbage_collect(&mut self) {
        self.memory_manager.collect();
    }

    /// 获取内存使用情况
    ///
    /// # 返回值
    /// - 内存使用情况字符串
    pub fn get_memory_usage(&self) -> String {
        format!("Memory usage: {} objects", self.memory_manager.get_object_count())
    }

    /// 编译选项 API

    /// 设置编译选项
    ///
    /// # 参数
    /// - `options`: 编译选项
    pub fn set_compile_options(&mut self, options: CompileOptions) {
        self.compile_options = options;
    }

    /// 获取编译选项
    ///
    /// # 返回值
    /// - 当前编译选项
    pub fn get_compile_options(&self) -> &CompileOptions {
        &self.compile_options
    }

    /// 运行时 API

    /// 设置全局变量
    ///
    /// # 参数
    /// - `name`: 变量名称
    /// - `value`: 变量值
    pub fn set_global(&mut self, name: &str, value: TsValue) {
        self.globals.insert(name.to_string(), value);
    }

    /// 获取全局变量
    ///
    /// # 参数
    /// - `name`: 变量名称
    ///
    /// # 返回值
    /// - 变量值（如果存在）
    pub fn get_global(&self, name: &str) -> Option<&TsValue> {
        self.globals.get(name)
    }

    /// 注册 FFI 函数
    ///
    /// # 参数
    /// - `name`: 函数名称
    /// - `func`: 函数实现
    pub fn register_ffi_function(&mut self, name: &str, func: Rc<dyn Fn(&[TsValue]) -> TsValue>) {
        // 暂时跳过 FFI 函数注册，需要修复类型错误
    }

    /// 清除所有状态
    pub fn clear(&mut self) {
        self.globals.clear();
        self.modules.clear();
        self.memory_manager.clear();
    }

    /// 检查运行时状态
    ///
    /// # 返回值
    /// - 运行时状态字符串
    pub fn get_status(&self) -> String {
        format!(
            "TypeScript Runtime Status:\n{}",
            format!(
                "- Globals: {}\n- Modules: {}\n- Objects: {}\n- JIT Enabled: {}\n- Type Checking: {}",
                self.globals.len(),
                self.modules.len(),
                self.memory_manager.get_object_count(),
                self.compile_options.enable_jit,
                self.compile_options.enable_type_checking
            )
        )
    }
}

/// 从 Rust 类型转换为 TypeScript 值的 trait 实现扩展
impl ToTsValue for TypeScript {
    fn to_ts_value(&self) -> TsValue {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        for (key, value) in &self.globals {
            map.insert(key.clone(), value.clone());
        }
        // 添加模块信息
        let mut modules_map = HashMap::new();
        for (key, value) in &self.modules {
            modules_map.insert(key.clone(), value.clone());
        }
        map.insert("__modules".to_string(), TsValue::Object(modules_map));
        // 添加编译选项信息
        let mut options_map = HashMap::new();
        options_map.insert("strict".to_string(), TsValue::Boolean(self.compile_options.strict));
        options_map.insert("enable_jit".to_string(), TsValue::Boolean(self.compile_options.enable_jit));
        options_map.insert("enable_type_checking".to_string(), TsValue::Boolean(self.compile_options.enable_type_checking));
        options_map.insert("target_es_version".to_string(), TsValue::String(self.compile_options.target_es_version.clone()));
        map.insert("__compile_options".to_string(), TsValue::Object(options_map));
        TsValue::Object(map)
    }
}

/// 测试函数，用于调试词法分析器和解析器
pub fn test_lexer_parser() {
    let source = "let x = 10; let y = 'hello'; console.log(x, y); x + y";
    println!("Source: {}", source);
    println!("Test lexer parser function called");
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
