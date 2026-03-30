//! 错误相关类型和类
//! 
//! 定义了所有错误类型、错误类和相关接口

/**
 * 错误类型枚举
 *
 * 定义了所有可能的错误类型
 */
export type ErrorType = "syntax" | "type" | "runtime" | "memory" | "module" | "ffi";

/**
 * 错误位置接口
 *
 * 表示错误在源代码中的位置
 */
export interface ErrorLocation {
    /**
     * 行号（从 1 开始）
     */
    line: number;

    /**
     * 列号（从 1 开始）
     */
    column: number;

    /**
     * 文件路径（可选）
     */
    file?: string;

    /**
     * 源代码片段（可选）
     */
    snippet?: string;
}

/**
 * 错误上下文接口
 *
 * 包含错误的额外上下文信息
 */
export interface ErrorContext {
    /**
     * 相关代码片段
     */
    code?: string;

    /**
     * 错误发生时的变量状态
     */
    variables?: Record<string, unknown>;

    /**
     * 调用堆栈
     */
    callStack?: string[];

    /**
     * 额外的元数据
     */
    metadata?: Record<string, unknown>;
}

/**
 * Rusty TypeScript 错误类
 *
 * 表示 TypeScript 编译或执行过程中发生的错误
 */
export class RustyTypeScriptError extends Error {
    /**
     * 错误类型
     */
    public readonly type: ErrorType;

    /**
     * 错误位置
     */
    public readonly location?: ErrorLocation;

    /**
     * 错误堆栈
     */
    public readonly errorStack?: string;

    /**
     * 修复建议列表
     */
    public readonly suggestions: string[];

    /**
     * 原始错误对象
     */
    public readonly cause?: Error;

    /**
     * 错误上下文信息
     */
    public readonly context?: ErrorContext;

    /**
     * 错误代码（用于国际化或错误查找）
     */
    public readonly code?: string;

    /**
     * 构造函数
     *
     * @param message 错误消息
     * @param type 错误类型
     * @param options 错误选项
     */
    constructor(
        message: string,
        type: ErrorType,
        options?: {
            location?: ErrorLocation;
            stack?: string;
            suggestions?: string[];
            cause?: Error;
            context?: ErrorContext;
            code?: string;
        },
    ) {
        super(message);
        this.name = "RustyTypeScriptError";
        this.type = type;
        this.location = options?.location;
        this.errorStack = options?.stack;
        this.suggestions = options?.suggestions ?? [];
        this.cause = options?.cause;
        this.context = options?.context;
        this.code = options?.code;

        if (this.cause && this.cause.stack) {
            this.errorStack = this.cause.stack;
        }
    }

    /**
     * 从原始错误创建 RustyTypeScriptError
     *
     * @param error 原始错误
     * @param type 错误类型
     * @param message 自定义错误消息（可选）
     * @returns RustyTypeScriptError 实例
     */
    public static fromError(error: Error, type: ErrorType, message?: string): RustyTypeScriptError {
        return new RustyTypeScriptError(message ?? error.message, type, {
            cause: error,
            stack: error.stack,
        });
    }

    /**
     * 添加修复建议
     *
     * @param suggestion 修复建议
     * @returns 当前错误实例（支持链式调用）
     */
    public addSuggestion(suggestion: string): this {
        this.suggestions.push(suggestion);
        return this;
    }

    /**
     * 设置错误上下文
     *
     * @param context 错误上下文
     * @returns 当前错误实例（支持链式调用）
     */
    public setContext(context: ErrorContext): this {
        (this as { context?: ErrorContext }).context = context;
        return this;
    }

    /**
     * 获取完整的错误信息（包含位置和建议）
     *
     * @returns 格式化的错误信息
     */
    public getFullMessage(): string {
        let fullMessage = `[${this.type.toUpperCase()}] ${this.message}`;

        if (this.code) {
            fullMessage = `[${this.code}] ${fullMessage}`;
        }

        if (this.location) {
            const locationStr = this.location.file
                ? `${this.location.file}:${this.location.line}:${this.location.column}`
                : `Line ${this.location.line}, Column ${this.location.column}`;
            fullMessage += `\n  Location: ${locationStr}`;

            if (this.location.snippet) {
                fullMessage += `\n  Code: ${this.location.snippet}`;
            }
        }

        if (this.suggestions.length > 0) {
            fullMessage += "\n  Suggestions:";
            this.suggestions.forEach((s, i) => {
                fullMessage += `\n    ${i + 1}. ${s}`;
            });
        }

        return fullMessage;
    }
}

/**
 * WASM 加载错误类型枚举
 */
export type WasmLoadErrorType = 
    | "NetworkError"
    | "CompileError"
    | "InstantiateError"
    | "TimeoutError";

/**
 * WASM 加载错误类
 *
 * 表示 WASM 模块加载过程中发生的错误
 */
export class WasmLoadError extends RustyTypeScriptError {
    /**
     * WASM 加载错误类型
     */
    public readonly loadErrorType: WasmLoadErrorType;

    /**
     * 加载 URL（如果有）
     */
    public readonly url?: string;

    /**
     * 重试次数
     */
    public readonly retryCount: number;

    /**
     * 最大重试次数
     */
    public readonly maxRetries: number;

    /**
     * 构造函数
     *
     * @param message 错误消息
     * @param loadErrorType 加载错误类型
     * @param options 错误选项
     */
    constructor(
        message: string,
        loadErrorType: WasmLoadErrorType,
        options?: {
            url?: string;
            retryCount?: number;
            maxRetries?: number;
            cause?: Error;
            suggestions?: string[];
        },
    ) {
        const errorTypeMap: Record<WasmLoadErrorType, ErrorType> = {
            NetworkError: "runtime",
            CompileError: "runtime",
            InstantiateError: "runtime",
            TimeoutError: "runtime",
        };

        const defaultSuggestions: Record<WasmLoadErrorType, string[]> = {
            NetworkError: [
                "Check network connection",
                "Verify WASM file URL",
                "Ensure server is running",
                "Try refreshing the page",
            ],
            CompileError: [
                "Check if WASM file is corrupted",
                "Ensure WASM file format is correct",
                "Verify browser support for WASM features",
            ],
            InstantiateError: [
                "Check if WASM import object is correct",
                "Ensure WASM export functions exist",
                "Check if memory limit is sufficient",
            ],
            TimeoutError: ["Check network connection speed", "Consider increasing timeout", "Try using a smaller WASM file"],
        };

        super(message, errorTypeMap[loadErrorType], {
            cause: options?.cause,
            code: `WASM_${loadErrorType.toUpperCase()}`,
            suggestions: options?.suggestions ?? defaultSuggestions[loadErrorType],
        });

        this.name = "WasmLoadError";
        this.loadErrorType = loadErrorType;
        this.url = options?.url;
        this.retryCount = options?.retryCount ?? 0;
        this.maxRetries = options?.maxRetries ?? 3;
    }

    /**
     * 创建网络错误
     *
     * @param message 错误消息
     * @param url 请求 URL
     * @param cause 原始错误
     * @returns WasmLoadError 实例
     */
    public static networkError(message: string, url?: string, cause?: Error): WasmLoadError {
        return new WasmLoadError(message, "NetworkError", { url, cause });
    }

    /**
     * 创建编译错误
     *
     * @param message 错误消息
     * @param cause 原始错误
     * @returns WasmLoadError 实例
     */
    public static compileError(message: string, cause?: Error): WasmLoadError {
        return new WasmLoadError(message, "CompileError", { cause });
    }

    /**
     * 创建实例化错误
     *
     * @param message 错误消息
     * @param cause 原始错误
     * @returns WasmLoadError 实例
     */
    public static instantiateError(message: string, cause?: Error): WasmLoadError {
        return new WasmLoadError(message, "InstantiateError", { cause });
    }

    /**
     * 创建超时错误
     *
     * @param message 错误消息
     * @param url 请求 URL
     * @returns WasmLoadError 实例
     */
    public static timeoutError(message: string, url?: string): WasmLoadError {
        return new WasmLoadError(message, "TimeoutError", { url });
    }

    /**
     * 是否可以重试
     *
     * @returns 是否可以继续重试
     */
    public canRetry(): boolean {
        return this.retryCount < this.maxRetries;
    }

    /**
     * 创建重试错误
     *
     * @param newRetryCount 新的重试次数
     * @returns 新的 WasmLoadError 实例
     */
    public withRetry(newRetryCount: number): WasmLoadError {
        return new WasmLoadError(this.message, this.loadErrorType, {
            url: this.url,
            retryCount: newRetryCount,
            maxRetries: this.maxRetries,
            cause: this.cause,
            suggestions: this.suggestions,
        });
    }

    /**
     * 获取完整的错误信息
     *
     * @returns 格式化的错误信息
     */
    public getFullMessage(): string {
        let fullMessage = `[${this.loadErrorType}] ${this.message}`;

        if (this.url) {
            fullMessage += `\n  URL: ${this.url}`;
        }

        if (this.retryCount > 0) {
            fullMessage += `\n  Retry count: ${this.retryCount}/${this.maxRetries}`;
        }

        if (this.suggestions.length > 0) {
            fullMessage += "\n  Suggestions:";
            this.suggestions.forEach((s, i) => {
                fullMessage += `\n    ${i + 1}. ${s}`;
            });
        }

        return fullMessage;
    }
}
