//! TypeScript WASI 包装层
//!
//! 此包提供了对 TypeScript WASM 功能的 TypeScript 接口。

/**
 * Rusty TypeScript 错误类
 */
export class RustyTypeScriptError extends Error {
    /**
     * 错误类型
     */
    public type: "syntax" | "type" | "runtime";

    /**
     * 错误位置
     */
    public location?: {
        line: number;
        column: number;
    };

    /**
     * 构造函数
     *
     * @param message 错误消息
     * @param type 错误类型
     * @param location 错误位置
     */
    constructor(
        message: string,
        type: "syntax" | "type" | "runtime",
        location?: {
            line: number;
            column: number;
        },
    ) {
        super(message);
        this.name = "RustyTypeScriptError";
        this.type = type;
        this.location = location;
    }
}

/**
 * 控制台接口
 */
export interface Console {
    /**
     * 日志输出
     */
    log: (...args: unknown[]) => void;

    /**
     * 错误输出
     */
    error: (...args: unknown[]) => void;

    /**
     * 警告输出
     */
    warn: (...args: unknown[]) => void;
}

/**
 * 初始化选项
 */
export interface InitOptions {
    /**
     * WASM 模块 URL（浏览器环境）
     */
    wasmUrl?: string;

    /**
     * WASM 模块二进制数据（Node.js 环境）
     */
    wasmModule?: Buffer;

    /**
     * 自定义控制台
     */
    console?: Console;
}

/**
 * 执行结果
 */
export interface ExecuteResult {
    /**
     * 执行结果值
     */
    value: unknown;

    /**
     * 执行时间（毫秒）
     */
    time: number;
}

/**
 * 性能指标
 */
export interface PerformanceMetrics {
    /**
     * 执行时间（毫秒）
     */
    executionTime: number;

    /**
     * 内存使用（字节）
     */
    memoryUsage: number;

    /**
     * 操作数
     */
    operations: number;
}

/**
 * WASM 模块接口
 */
interface WasmModule {
    initialize: () => number;
    allocate: (size: number) => number;
    deallocate: (ptr: number, size: number) => void;
    compile_typescript: (code: number) => number;
    execute_typescript: (code: number) => number;
    get_compilation_errors: (code: number) => number;
    get_performance_metrics: () => number;
    evaluate_expression: (expr: number) => number;
    get_version: () => number;
    free_memory: (ptr: number) => void;
    // 模块管理 API
    import_module: (name: number, alias: number) => number;
    export_module: (name: number, value: number) => void;
    get_module: (name: number) => number;
    // 类型系统 API
    register_type: (name: number, definition: number) => void;
    get_type: (name: number) => number;
    // 性能监控 API
    enable_performance_monitoring: () => void;
    disable_performance_monitoring: () => void;
    get_performance_report: () => number;
    // 内存管理 API
    garbage_collect: () => void;
    get_memory_usage: () => number;
    // 编译选项 API
    set_compile_options: (options: number) => void;
    get_compile_options: () => number;
    // 运行时 API
    set_global: (name: number, value: number) => void;
    get_global: (name: number) => number;
    register_ffi_function: (name: number, func: number) => void;
    clear: () => void;
    get_status: () => number;
    memory: WebAssembly.Memory;
}

/**
 * Rusty TypeScript 类
 *
 * 包装了 TypeScript WASM 模块的原始接口，提供更友好的使用方式。
 */
export class RustyTypeScript {
    private wasmModule: WasmModule | null = null;
    private console: Console;

    /**
     * 初始化 Rusty TypeScript
     *
     * @param options 初始化选项
     * @returns 初始化后的 RustyTypeScript 实例
     */
    public static async init(options: InitOptions = {}): Promise<RustyTypeScript> {
        const instance = new RustyTypeScript(options.console);
        await instance.loadWasmModule(options);
        return instance;
    }

    /**
     * 构造函数
     *
     * @param console 自定义控制台
     */
    private constructor(console?: Console) {
        const globalConsole =
            typeof console !== "undefined"
                ? console
                : {
                      log: (...args: unknown[]) => {},
                      error: (...args: unknown[]) => {},
                      warn: (...args: unknown[]) => {},
                  };

        this.console = console || {
            log: (...args: unknown[]) => globalConsole.log(...args),
            error: (...args: unknown[]) => globalConsole.error(...args),
            warn: (...args: unknown[]) => globalConsole.warn(...args),
        };
    }

    /**
     * 加载 WASM 模块
     *
     * @param options 初始化选项
     */
    private async loadWasmModule(options: InitOptions): Promise<void> {
        try {
            let wasmModule: WebAssembly.Module;

            if (options.wasmModule) {
                // Node.js 环境
                wasmModule = new WebAssembly.Module(options.wasmModule);
            } else {
                // 浏览器环境
                const url = options.wasmUrl || "/typescript-wasi.wasm";
                const response = await fetch(url);
                const buffer = await response.arrayBuffer();
                wasmModule = await WebAssembly.compile(buffer);
            }

            const imports = {
                env: {
                    log: (ptr: number, len: number) => {
                        const message = this.getString(ptr, len);
                        this.console.log(message);
                    },
                    error: (ptr: number, len: number) => {
                        const message = this.getString(ptr, len);
                        this.console.error(message);
                    },
                    warn: (ptr: number, len: number) => {
                        const message = this.getString(ptr, len);
                        this.console.warn(message);
                    },
                },
            };

            const wasmInstance = await WebAssembly.instantiate(wasmModule, imports);
            this.wasmModule = wasmInstance.exports as unknown as WasmModule;

            // 初始化 WASM 模块
            if (this.wasmModule.initialize) {
                this.wasmModule.initialize();
            }
        } catch (error) {
            this.console.error("Failed to load WASM module:", error);
            throw new RustyTypeScriptError("Failed to load TypeScript WASI module", "runtime");
        }
    }

    /**
     * 将字符串转换为 WASM 内存中的指针
     *
     * @param str 字符串
     * @returns 指针
     */
    private stringToPtr(str: string): number {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM module not initialized", "runtime");
        }

        const bytes = new TextEncoder().encode(str + "\0");
        const memory = this.wasmModule.memory;
        const view = new Uint8Array(memory.buffer);

        // 简单的内存分配实现，实际应用中可能需要更复杂的内存管理
        let ptr = 1024; // 从 1024 开始分配内存
        while (ptr < view.length) {
            let isFree = true;
            for (let i = 0; i < bytes.length; i++) {
                if (view[ptr + i] !== 0) {
                    isFree = false;
                    break;
                }
            }
            if (isFree) {
                break;
            }
            ptr += bytes.length;
        }

        // 复制字符串到内存
        for (let i = 0; i < bytes.length; i++) {
            view[ptr + i] = bytes[i];
        }

        return ptr;
    }

    /**
     * 从 WASM 内存中获取字符串
     *
     * @param ptr 指针
     * @param len 长度（可选）
     * @returns 字符串
     */
    private getString(ptr: number, len?: number): string {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM module not initialized", "runtime");
        }

        const memory = this.wasmModule.memory;
        const view = new Uint8Array(memory.buffer);

        if (len === undefined) {
            // 查找 null 终止符
            let end = ptr;
            while (end < view.length && view[end] !== 0) {
                end++;
            }
            len = end - ptr;
        }

        const bytes = view.subarray(ptr, ptr + len);
        return new TextDecoder().decode(bytes);
    }

    /**
     * 编译 TypeScript 代码
     *
     * @param code TypeScript 代码字符串
     * @returns 编译结果
     */
    public async compile(code: string): Promise<string> {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM module not initialized", "runtime");
        }

        const ptr = this.stringToPtr(code);
        const resultPtr = this.wasmModule.compile_typescript(ptr);
        const result = this.getString(resultPtr);
        this.wasmModule.free_memory(resultPtr);
        return result;
    }

    /**
     * 执行 TypeScript 代码
     *
     * @param code TypeScript 代码字符串
     * @returns 执行结果
     */
    public async execute(code: string): Promise<ExecuteResult> {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM module not initialized", "runtime");
        }

        const startTime = performance.now();
        const ptr = this.stringToPtr(code);
        const resultPtr = this.wasmModule.execute_typescript(ptr);
        const resultStr = this.getString(resultPtr);
        const endTime = performance.now();

        this.wasmModule.free_memory(resultPtr);

        try {
            const result = JSON.parse(resultStr);
            return {
                value: result.result || result,
                time: endTime - startTime,
            };
        } catch {
            return {
                value: resultStr,
                time: endTime - startTime,
            };
        }
    }

    /**
     * 获取编译错误
     *
     * @param code TypeScript 代码字符串
     * @returns 错误列表
     */
    public async getCompilationErrors(code: string): Promise<string[]> {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM module not initialized", "runtime");
        }

        const ptr = this.stringToPtr(code);
        const resultPtr = this.wasmModule.get_compilation_errors(ptr);
        const resultStr = this.getString(resultPtr);
        this.wasmModule.free_memory(resultPtr);

        try {
            return JSON.parse(resultStr);
        } catch {
            return [];
        }
    }

    /**
     * 获取性能指标
     *
     * @returns 性能指标
     */
    public async getPerformanceMetrics(): Promise<PerformanceMetrics> {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM module not initialized", "runtime");
        }

        const resultPtr = this.wasmModule.get_performance_metrics();
        const resultStr = this.getString(resultPtr);
        this.wasmModule.free_memory(resultPtr);

        try {
            return JSON.parse(resultStr);
        } catch {
            return {
                executionTime: 0,
                memoryUsage: 0,
                operations: 0,
            };
        }
    }

    /**
     * 评估表达式
     *
     * @param expr 表达式字符串
     * @returns 评估结果
     */
    public async evaluateExpression(expr: string): Promise<unknown> {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM module not initialized", "runtime");
        }

        const ptr = this.stringToPtr(expr);
        const resultPtr = this.wasmModule.evaluate_expression(ptr);
        const resultStr = this.getString(resultPtr);
        this.wasmModule.free_memory(resultPtr);

        try {
            const result = JSON.parse(resultStr);
            return result.result || result;
        } catch {
            return resultStr;
        }
    }

    /**
     * 获取版本信息
     *
     * @returns 版本字符串
     */
    public async getVersion(): Promise<string> {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM module not initialized", "runtime");
        }

        const resultPtr = this.wasmModule.get_version();
        const result = this.getString(resultPtr);
        this.wasmModule.free_memory(resultPtr);
        return result;
    }

    /**
     * 模块管理 API
     */

    /**
     * 导入模块
     *
     * @param name 模块名称
     * @param alias 模块别名（可选）
     */
    public async importModule(name: string, alias?: string): Promise<void> {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM module not initialized", "runtime");
        }

        const namePtr = this.stringToPtr(name);
        const aliasPtr = alias ? this.stringToPtr(alias) : 0;
        const resultPtr = this.wasmModule.import_module(namePtr, aliasPtr);
        const result = this.getString(resultPtr);
        this.wasmModule.free_memory(resultPtr);

        if (result !== "ok") {
            throw new RustyTypeScriptError(result, "runtime");
        }
    }

    /**
     * 导出模块
     *
     * @param name 模块名称
     * @param value 模块值
     */
    public async exportModule(name: string, value: unknown): Promise<void> {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM module not initialized", "runtime");
        }

        const namePtr = this.stringToPtr(name);
        const valuePtr = this.stringToPtr(JSON.stringify(value));
        this.wasmModule.export_module(namePtr, valuePtr);
    }

    /**
     * 获取模块
     *
     * @param name 模块名称
     * @returns 模块值
     */
    public async getModule(name: string): Promise<unknown> {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM module not initialized", "runtime");
        }

        const namePtr = this.stringToPtr(name);
        const resultPtr = this.wasmModule.get_module(namePtr);
        const resultStr = this.getString(resultPtr);
        this.wasmModule.free_memory(resultPtr);

        try {
            return JSON.parse(resultStr);
        } catch {
            return resultStr;
        }
    }

    /**
     * 类型系统 API
     */

    /**
     * 注册类型
     *
     * @param name 类型名称
     * @param definition 类型定义
     */
    public async registerType(name: string, definition: unknown): Promise<void> {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM module not initialized", "runtime");
        }

        const namePtr = this.stringToPtr(name);
        const definitionPtr = this.stringToPtr(JSON.stringify(definition));
        this.wasmModule.register_type(namePtr, definitionPtr);
    }

    /**
     * 获取类型
     *
     * @param name 类型名称
     * @returns 类型定义
     */
    public async getType(name: string): Promise<unknown> {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM module not initialized", "runtime");
        }

        const namePtr = this.stringToPtr(name);
        const resultPtr = this.wasmModule.get_type(namePtr);
        const resultStr = this.getString(resultPtr);
        this.wasmModule.free_memory(resultPtr);

        try {
            return JSON.parse(resultStr);
        } catch {
            return resultStr;
        }
    }

    /**
     * 性能监控 API
     */

    /**
     * 启用性能监控
     */
    public async enablePerformanceMonitoring(): Promise<void> {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM module not initialized", "runtime");
        }

        this.wasmModule.enable_performance_monitoring();
    }

    /**
     * 禁用性能监控
     */
    public async disablePerformanceMonitoring(): Promise<void> {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM module not initialized", "runtime");
        }

        this.wasmModule.disable_performance_monitoring();
    }

    /**
     * 获取性能报告
     *
     * @returns 性能报告
     */
    public async getPerformanceReport(): Promise<string> {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM module not initialized", "runtime");
        }

        const resultPtr = this.wasmModule.get_performance_report();
        const result = this.getString(resultPtr);
        this.wasmModule.free_memory(resultPtr);
        return result;
    }

    /**
     * 内存管理 API
     */

    /**
     * 执行垃圾回收
     */
    public async garbageCollect(): Promise<void> {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM module not initialized", "runtime");
        }

        this.wasmModule.garbage_collect();
    }

    /**
     * 获取内存使用情况
     *
     * @returns 内存使用情况
     */
    public async getMemoryUsage(): Promise<string> {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM module not initialized", "runtime");
        }

        const resultPtr = this.wasmModule.get_memory_usage();
        const result = this.getString(resultPtr);
        this.wasmModule.free_memory(resultPtr);
        return result;
    }

    /**
     * 编译选项 API
     */

    /**
     * 设置编译选项
     *
     * @param options 编译选项
     */
    public async setCompileOptions(options: {
        strict?: boolean;
        enableJit?: boolean;
        enableTypeChecking?: boolean;
        enablePerformanceMonitoring?: boolean;
        enableGarbageCollection?: boolean;
        targetEsVersion?: string;
        moduleResolution?: string;
        sourceMap?: boolean;
    }): Promise<void> {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM module not initialized", "runtime");
        }

        const optionsPtr = this.stringToPtr(JSON.stringify(options));
        this.wasmModule.set_compile_options(optionsPtr);
    }

    /**
     * 获取编译选项
     *
     * @returns 编译选项
     */
    public async getCompileOptions(): Promise<unknown> {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM module not initialized", "runtime");
        }

        const resultPtr = this.wasmModule.get_compile_options();
        const resultStr = this.getString(resultPtr);
        this.wasmModule.free_memory(resultPtr);

        try {
            return JSON.parse(resultStr);
        } catch {
            return resultStr;
        }
    }

    /**
     * 运行时 API
     */

    /**
     * 设置全局变量
     *
     * @param name 变量名称
     * @param value 变量值
     */
    public async setGlobal(name: string, value: unknown): Promise<void> {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM module not initialized", "runtime");
        }

        const namePtr = this.stringToPtr(name);
        const valuePtr = this.stringToPtr(JSON.stringify(value));
        this.wasmModule.set_global(namePtr, valuePtr);
    }

    /**
     * 获取全局变量
     *
     * @param name 变量名称
     * @returns 变量值
     */
    public async getGlobal(name: string): Promise<unknown> {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM module not initialized", "runtime");
        }

        const namePtr = this.stringToPtr(name);
        const resultPtr = this.wasmModule.get_global(namePtr);
        const resultStr = this.getString(resultPtr);
        this.wasmModule.free_memory(resultPtr);

        try {
            return JSON.parse(resultStr);
        } catch {
            return resultStr;
        }
    }

    /**
     * 注册 FFI 函数
     *
     * @param name 函数名称
     * @param func 函数实现
     */
    public async registerFfiFunction(
        name: string,
        func: (...args: unknown[]) => unknown,
    ): Promise<void> {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM module not initialized", "runtime");
        }

        // 注意：这里只是示例，实际 FFI 函数注册需要更复杂的实现
        const namePtr = this.stringToPtr(name);
        const funcPtr = this.stringToPtr(JSON.stringify({})); // 占位符
        this.wasmModule.register_ffi_function(namePtr, funcPtr);
    }

    /**
     * 清除所有状态
     */
    public async clear(): Promise<void> {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM module not initialized", "runtime");
        }

        this.wasmModule.clear();
    }

    /**
     * 检查运行时状态
     *
     * @returns 运行时状态
     */
    public async getStatus(): Promise<string> {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM module not initialized", "runtime");
        }

        const resultPtr = this.wasmModule.get_status();
        const result = this.getString(resultPtr);
        this.wasmModule.free_memory(resultPtr);
        return result;
    }

    /**
     * 释放资源
     */
    public dispose(): void {
        // 释放 WASM 模块资源
        this.wasmModule = null;
    }
}

/**
 * 编译 TypeScript 代码
 *
 * @param code TypeScript 代码字符串
 * @returns 编译结果
 */
export async function compileTypeScript(code: string): Promise<string> {
    const rustyTs = await RustyTypeScript.init();
    try {
        return await rustyTs.compile(code);
    } finally {
        rustyTs.dispose();
    }
}

/**
 * 执行 TypeScript 代码
 *
 * @param code TypeScript 代码字符串
 * @returns 执行结果
 */
export async function executeTypeScript(code: string): Promise<{ result: any; time: number }> {
    const rustyTs = await RustyTypeScript.init();
    try {
        const result = await rustyTs.execute(code);
        return {
            result: result.value,
            time: result.time,
        };
    } finally {
        rustyTs.dispose();
    }
}

/**
 * 获取编译错误
 *
 * @param code TypeScript 代码字符串
 * @returns 错误列表
 */
export async function getCompilationErrors(code: string): Promise<string[]> {
    const rustyTs = await RustyTypeScript.init();
    try {
        return await rustyTs.getCompilationErrors(code);
    } finally {
        rustyTs.dispose();
    }
}

/**
 * 获取性能指标
 *
 * @returns 性能指标
 */
export async function getPerformanceMetrics(): Promise<PerformanceMetrics> {
    const rustyTs = await RustyTypeScript.init();
    try {
        return await rustyTs.getPerformanceMetrics();
    } finally {
        rustyTs.dispose();
    }
}

/**
 * 获取版本信息
 *
 * @returns 版本字符串
 */
export async function getVersion(): Promise<string> {
    const rustyTs = await RustyTypeScript.init();
    try {
        return await rustyTs.getVersion();
    } finally {
        rustyTs.dispose();
    }
}

/**
 * 模块管理便捷函数
 */

export async function importModule(name: string, alias?: string): Promise<void> {
    const rustyTs = await RustyTypeScript.init();
    try {
        return await rustyTs.importModule(name, alias);
    } finally {
        rustyTs.dispose();
    }
}

export async function exportModule(name: string, value: unknown): Promise<void> {
    const rustyTs = await RustyTypeScript.init();
    try {
        return await rustyTs.exportModule(name, value);
    } finally {
        rustyTs.dispose();
    }
}

export async function getModule(name: string): Promise<unknown> {
    const rustyTs = await RustyTypeScript.init();
    try {
        return await rustyTs.getModule(name);
    } finally {
        rustyTs.dispose();
    }
}

/**
 * 类型系统便捷函数
 */

export async function registerType(name: string, definition: unknown): Promise<void> {
    const rustyTs = await RustyTypeScript.init();
    try {
        return await rustyTs.registerType(name, definition);
    } finally {
        rustyTs.dispose();
    }
}

export async function getType(name: string): Promise<unknown> {
    const rustyTs = await RustyTypeScript.init();
    try {
        return await rustyTs.getType(name);
    } finally {
        rustyTs.dispose();
    }
}

/**
 * 性能监控便捷函数
 */

export async function enablePerformanceMonitoring(): Promise<void> {
    const rustyTs = await RustyTypeScript.init();
    try {
        return await rustyTs.enablePerformanceMonitoring();
    } finally {
        rustyTs.dispose();
    }
}

export async function disablePerformanceMonitoring(): Promise<void> {
    const rustyTs = await RustyTypeScript.init();
    try {
        return await rustyTs.disablePerformanceMonitoring();
    } finally {
        rustyTs.dispose();
    }
}

export async function getPerformanceReport(): Promise<string> {
    const rustyTs = await RustyTypeScript.init();
    try {
        return await rustyTs.getPerformanceReport();
    } finally {
        rustyTs.dispose();
    }
}

/**
 * 内存管理便捷函数
 */

export async function garbageCollect(): Promise<void> {
    const rustyTs = await RustyTypeScript.init();
    try {
        return await rustyTs.garbageCollect();
    } finally {
        rustyTs.dispose();
    }
}

export async function getMemoryUsage(): Promise<string> {
    const rustyTs = await RustyTypeScript.init();
    try {
        return await rustyTs.getMemoryUsage();
    } finally {
        rustyTs.dispose();
    }
}

/**
 * 编译选项便捷函数
 */

export async function setCompileOptions(options: {
    strict?: boolean;
    enableJit?: boolean;
    enableTypeChecking?: boolean;
    enablePerformanceMonitoring?: boolean;
    enableGarbageCollection?: boolean;
    targetEsVersion?: string;
    moduleResolution?: string;
    sourceMap?: boolean;
}): Promise<void> {
    const rustyTs = await RustyTypeScript.init();
    try {
        return await rustyTs.setCompileOptions(options);
    } finally {
        rustyTs.dispose();
    }
}

export async function getCompileOptions(): Promise<unknown> {
    const rustyTs = await RustyTypeScript.init();
    try {
        return await rustyTs.getCompileOptions();
    } finally {
        rustyTs.dispose();
    }
}

/**
 * 运行时便捷函数
 */

export async function setGlobal(name: string, value: unknown): Promise<void> {
    const rustyTs = await RustyTypeScript.init();
    try {
        return await rustyTs.setGlobal(name, value);
    } finally {
        rustyTs.dispose();
    }
}

export async function getGlobal(name: string): Promise<unknown> {
    const rustyTs = await RustyTypeScript.init();
    try {
        return await rustyTs.getGlobal(name);
    } finally {
        rustyTs.dispose();
    }
}

export async function registerFfiFunction(
    name: string,
    func: (...args: unknown[]) => unknown,
): Promise<void> {
    const rustyTs = await RustyTypeScript.init();
    try {
        return await rustyTs.registerFfiFunction(name, func);
    } finally {
        rustyTs.dispose();
    }
}

export async function clear(): Promise<void> {
    const rustyTs = await RustyTypeScript.init();
    try {
        return await rustyTs.clear();
    } finally {
        rustyTs.dispose();
    }
}

export async function getStatus(): Promise<string> {
    const rustyTs = await RustyTypeScript.init();
    try {
        return await rustyTs.getStatus();
    } finally {
        rustyTs.dispose();
    }
}
