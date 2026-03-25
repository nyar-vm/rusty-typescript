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
 * Rusty TypeScript 类
 *
 * 包装了 TypeScript WASM 模块的原始接口，提供更友好的使用方式。
 */
export class RustyTypeScript {
    /**
     * 初始化 Rusty TypeScript
     *
     * @param options 初始化选项
     * @returns 初始化后的 RustyTypeScript 实例
     */
    public static async init(options: InitOptions = {}): Promise<RustyTypeScript> {
        // 这里将加载和初始化 WASM 模块
        // 实际实现将根据 WASM 模块的导出函数进行调整
        return new RustyTypeScript();
    }

    /**
     * 执行 TypeScript 代码
     *
     * @param code TypeScript 代码字符串
     * @returns 执行结果
     */
    public async execute(code: string): Promise<ExecuteResult> {
        // 调用原始接口进行执行
        // 这里需要根据实际的 WASM 导出函数进行调整
        return {
            value: undefined,
            time: 0,
        };
    }

    /**
     * 释放资源
     */
    public dispose(): void {
        // 释放 WASM 模块资源
    }
}
