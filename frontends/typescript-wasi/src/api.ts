//! TypeScript WASI API
//!
//! 提供对 TypeScript WASM 功能的 TypeScript 接口

import { WasmLoader, WasmLoaderOptions } from "./wasmLoader";
import { RustyTypeScriptError } from "./errors";

/**
 * 编译结果接口
 */
export interface CompileResult {
    /**
     * 编译是否成功
     */
    success: boolean;
    /**
     * 编译后的 JavaScript 代码
     */
    output: string;
    /**
     * 错误信息列表
     */
    errors: string[];
    /**
     * 警告信息列表
     */
    warnings: string[];
}

/**
 * 执行结果接口
 */
export interface ExecutionResult {
    /**
     * 执行是否成功
     */
    success: boolean;
    /**
     * 执行结果值
     */
    result: string;
    /**
     * 标准输出
     */
    stdout: string;
    /**
     * 标准错误
     */
    stderr: string;
    /**
     * 错误信息
     */
    error: string | null;
    /**
     * 执行时间（毫秒）
     */
    time?: number;
}

/**
 * 求值结果接口
 */
export interface EvaluationResult {
    /**
     * 求值是否成功
     */
    success: boolean;
    /**
     * 求值结果值
     */
    value: string;
    /**
     * 结果类型
     */
    type: string;
    /**
     * 错误信息
     */
    error: string | null;
}

/**
 * TypeScript WASI 主类
 *
 * 提供对 TypeScript WASM 功能的完整接口
 */
export class RustyTypeScript {
    /**
     * WASM 加载器实例
     */
    private readonly loader: WasmLoader;
    /**
     * WASM 实例
     */
    private instance: WebAssembly.Instance | null = null;

    /**
     * 构造函数
     *
     * @param options 加载器配置选项
     */
    constructor(options: WasmLoaderOptions = {}) {
        this.loader = new WasmLoader(options);
    }

    /**
     * 加载 WASM 模块
     *
     * @returns Promise<void>
     */
    public async load(): Promise<void> {
        this.instance = await this.loader.load();
    }

    /**
     * 编译 TypeScript 代码
     *
     * @param code TypeScript 代码字符串
     * @returns 编译结果
     */
    public compileTypeScript(code: string): CompileResult {
        if (!this.instance) {
            throw new RustyTypeScriptError("WASM module not loaded", "runtime");
        }

        const compile = this.instance.exports.compile as (code: number, codeLen: number) => number;
        const freeMemory = this.instance.exports.free_memory as (ptr: number) => void;

        if (!compile || !freeMemory) {
            throw new RustyTypeScriptError("Required WASM exports not found", "runtime");
        }

        const encoder = new TextEncoder();
        const buffer = encoder.encode(code);
        const memory = this.instance.exports.memory as WebAssembly.Memory;

        if (!memory) {
            throw new RustyTypeScriptError("WASM memory not found", "runtime");
        }

        const ptr = this.allocateMemory(memory, buffer.length);
        this.writeMemory(memory, ptr, buffer);

        const resultPtr = compile(ptr, buffer.length);
        const resultString = this.readString(memory, resultPtr);

        freeMemory(ptr);
        freeMemory(resultPtr);

        try {
            return JSON.parse(resultString) as CompileResult;
        } catch (error) {
            throw new RustyTypeScriptError(`Failed to parse compile result: ${error}`, "runtime");
        }
    }

    /**
     * 执行 JavaScript 代码
     *
     * @param code JavaScript 代码字符串
     * @returns 执行结果
     */
    public executeTypeScript(code: string): ExecutionResult {
        if (!this.instance) {
            throw new RustyTypeScriptError("WASM module not loaded", "runtime");
        }

        const execute = this.instance.exports.execute as (code: number, codeLen: number) => number;
        const freeMemory = this.instance.exports.free_memory as (ptr: number) => void;

        if (!execute || !freeMemory) {
            throw new RustyTypeScriptError("Required WASM exports not found", "runtime");
        }

        const encoder = new TextEncoder();
        const buffer = encoder.encode(code);
        const memory = this.instance.exports.memory as WebAssembly.Memory;

        if (!memory) {
            throw new RustyTypeScriptError("WASM memory not found", "runtime");
        }

        const ptr = this.allocateMemory(memory, buffer.length);
        this.writeMemory(memory, ptr, buffer);

        const resultPtr = execute(ptr, buffer.length);
        const resultString = this.readString(memory, resultPtr);

        freeMemory(ptr);
        freeMemory(resultPtr);

        try {
            return JSON.parse(resultString) as ExecutionResult;
        } catch (error) {
            throw new RustyTypeScriptError(`Failed to parse execution result: ${error}`, "runtime");
        }
    }

    /**
     * 获取编译错误
     *
     * @param code TypeScript 代码字符串
     * @returns 错误信息列表
     */
    public getCompilationErrors(code: string): string[] {
        if (!this.instance) {
            throw new RustyTypeScriptError("WASM module not loaded", "runtime");
        }

        const getCompilationErrors = this.instance.exports.get_compilation_errors as (
            code: number,
            codeLen: number,
        ) => number;
        const freeMemory = this.instance.exports.free_memory as (ptr: number) => void;

        if (!getCompilationErrors || !freeMemory) {
            throw new RustyTypeScriptError("Required WASM exports not found", "runtime");
        }

        const encoder = new TextEncoder();
        const buffer = encoder.encode(code);
        const memory = this.instance.exports.memory as WebAssembly.Memory;

        if (!memory) {
            throw new RustyTypeScriptError("WASM memory not found", "runtime");
        }

        const ptr = this.allocateMemory(memory, buffer.length);
        this.writeMemory(memory, ptr, buffer);

        const resultPtr = getCompilationErrors(ptr, buffer.length);
        const resultString = this.readString(memory, resultPtr);

        freeMemory(ptr);
        freeMemory(resultPtr);

        try {
            return JSON.parse(resultString) as string[];
        } catch (error) {
            throw new RustyTypeScriptError(`Failed to parse errors result: ${error}`, "runtime");
        }
    }

    /**
     * 分配内存
     *
     * @param memory WASM 内存
     * @param size 大小
     * @returns 内存指针
     */
    private allocateMemory(memory: WebAssembly.Memory, size: number): number {
        const realloc = this.instance!.exports.realloc as (
            ptr: number,
            oldSize: number,
            align: number,
            newSize: number,
        ) => number;
        if (!realloc) {
            throw new RustyTypeScriptError("realloc export not found", "runtime");
        }
        return realloc(0, 0, 1, size);
    }

    /**
     * 写入内存
     *
     * @param memory WASM 内存
     * @param ptr 指针
     * @param data 数据
     */
    private writeMemory(memory: WebAssembly.Memory, ptr: number, data: Uint8Array): void {
        const view = new Uint8Array(memory.buffer);
        view.set(data, ptr);
    }

    /**
     * 读取字符串
     *
     * @param memory WASM 内存
     * @param ptr 指针
     * @returns 字符串
     */
    private readString(memory: WebAssembly.Memory, ptr: number): string {
        const view = new Uint8Array(memory.buffer);
        let length = 0;

        while (view[ptr + length] !== 0) {
            length++;
        }

        const buffer = view.slice(ptr, ptr + length);
        return new TextDecoder().decode(buffer);
    }

    /**
     * 重置加载器状态
     */
    public reset(): void {
        this.loader.reset();
        this.instance = null;
    }
}

/**
 * 编译 TypeScript 代码
 *
 * @param code TypeScript 代码字符串
 * @param options 加载器配置选项
 * @returns 编译结果
 */
export async function compileTypeScript(
    code: string,
    options: WasmLoaderOptions = {},
): Promise<CompileResult> {
    const ts = new RustyTypeScript(options);
    await ts.load();
    const result = ts.compileTypeScript(code);
    ts.reset();
    return result;
}

/**
 * 执行 TypeScript 代码
 *
 * @param code TypeScript 代码字符串
 * @param options 加载器配置选项
 * @returns 执行结果
 */
export async function executeTypeScript(
    code: string,
    options: WasmLoaderOptions = {},
): Promise<ExecutionResult> {
    const ts = new RustyTypeScript(options);
    await ts.load();
    const result = ts.executeTypeScript(code);
    ts.reset();
    return result;
}

/**
 * 获取编译错误
 *
 * @param code TypeScript 代码字符串
 * @param options 加载器配置选项
 * @returns 错误信息列表
 */
export async function getCompilationErrors(
    code: string,
    options: WasmLoaderOptions = {},
): Promise<string[]> {
    const ts = new RustyTypeScript(options);
    await ts.load();
    const result = ts.getCompilationErrors(code);
    ts.reset();
    return result;
}
