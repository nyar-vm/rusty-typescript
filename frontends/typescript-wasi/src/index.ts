//! TypeScript WASI 包装层
//!
//! 此包提供了对 TypeScript WASM 功能的 TypeScript 接口。

export * from "./errorFormatter";

/**
 * 加载状态枚举
 *
 * 表示 WASM 模块的加载状态
 */
export enum LoadingState {
    /**
     * 未加载
     */
    Unloaded = "unloaded",

    /**
     * 正在加载
     */
    Loading = "loading",

    /**
     * 已加载
     */
    Loaded = "loaded",

    /**
     * 加载错误
     */
    Error = "error",
}

/**
 * 加载进度接口
 *
 * 包含 WASM 模块加载的进度信息
 */
export interface LoadingProgress {
    /**
     * 当前加载状态
     */
    state: LoadingState;

    /**
     * 加载进度百分比（0-100）
     */
    progress: number;

    /**
     * 已加载的字节数
     */
    loadedBytes: number;

    /**
     * 总字节数（如果已知）
     */
    totalBytes?: number;

    /**
     * 当前加载阶段描述
     */
    stage: string;

    /**
     * 错误信息（如果有）
     */
    error?: Error;

    /**
     * 加载开始时间戳
     */
    startTime?: number;

    /**
     * 加载完成时间戳
     */
    endTime?: number;
}

/**
 * 加载进度回调类型
 */
export type ProgressCallback = (progress: LoadingProgress) => void;

/**
 * WASM 加载错误类型枚举
 */
export type WasmLoadErrorType =
    | "NetworkError"
    | "CompileError"
    | "InstantiateError"
    | "TimeoutError";

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
                : `行 ${this.location.line}, 列 ${this.location.column}`;
            fullMessage += `\n  位置: ${locationStr}`;

            if (this.location.snippet) {
                fullMessage += `\n  代码: ${this.location.snippet}`;
            }
        }

        if (this.suggestions.length > 0) {
            fullMessage += "\n  建议:";
            this.suggestions.forEach((s, i) => {
                fullMessage += `\n    ${i + 1}. ${s}`;
            });
        }

        return fullMessage;
    }
}

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
                "检查网络连接是否正常",
                "确认 WASM 文件 URL 是否正确",
                "检查服务器是否正常运行",
                "尝试刷新页面或重新加载",
            ],
            CompileError: [
                "检查 WASM 文件是否损坏",
                "确认 WASM 文件格式正确",
                "检查浏览器是否支持该 WASM 特性",
            ],
            InstantiateError: [
                "检查 WASM 导入对象是否正确",
                "确认 WASM 导出函数存在",
                "检查内存限制是否足够",
            ],
            TimeoutError: ["检查网络连接速度", "考虑增加超时时间", "尝试使用更小的 WASM 文件"],
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
            fullMessage += `\n  重试次数: ${this.retryCount}/${this.maxRetries}`;
        }

        if (this.suggestions.length > 0) {
            fullMessage += "\n  建议:";
            this.suggestions.forEach((s, i) => {
                fullMessage += `\n    ${i + 1}. ${s}`;
            });
        }

        return fullMessage;
    }
}

/**
 * FFI 类型枚举
 *
 * 定义 FFI 支持的所有数据类型
 */
export enum FfiType {
    /**
     * 字符串类型
     */
    String = "string",

    /**
     * 数字类型
     */
    Number = "number",

    /**
     * 布尔类型
     */
    Boolean = "boolean",

    /**
     * 空值类型
     */
    Null = "null",

    /**
     * 未定义类型
     */
    Undefined = "undefined",

    /**
     * 数组类型
     */
    Array = "array",

    /**
     * 对象类型
     */
    Object = "object",

    /**
     * 函数类型
     */
    Function = "function",

    /**
     * 任意类型
     */
    Any = "any",

    /**
     * 无返回值类型
     */
    Void = "void",
}

/**
 * FFI 函数参数接口
 *
 * 描述 FFI 函数的参数信息
 */
export interface FfiParameter {
    /**
     * 参数名称
     */
    name: string;

    /**
     * 参数类型
     */
    type: FfiType;

    /**
     * 参数是否可选
     */
    optional?: boolean;

    /**
     * 参数默认值
     */
    defaultValue?: unknown;

    /**
     * 参数描述
     */
    description?: string;
}

/**
 * FFI 函数签名接口
 *
 * 描述 FFI 函数的完整签名信息
 */
export interface FfiSignature {
    /**
     * 参数列表
     */
    parameters: FfiParameter[];

    /**
     * 返回值类型
     */
    returnType: FfiType;

    /**
     * 是否可变参数
     */
    variadic?: boolean;
}

/**
 * FFI 函数接口
 *
 * 描述 FFI 函数的完整信息
 */
export interface FfiFunction {
    /**
     * 函数名称
     */
    name: string;

    /**
     * 函数签名
     */
    signature: FfiSignature;

    /**
     * 函数实现
     */
    implementation: (...args: unknown[]) => unknown;

    /**
     * 函数描述
     */
    description?: string;

    /**
     * 函数所属模块
     */
    module?: string;

    /**
     * 是否已弃用
     */
    deprecated?: boolean;

    /**
     * 弃用说明
     */
    deprecationMessage?: string;
}

/**
 * FFI 错误类型枚举
 *
 * 定义所有可能的 FFI 错误类型
 */
export type FfiErrorType = "FunctionNotFound" | "TypeMismatch" | "InvalidSignature" | "CallFailed";

/**
 * FFI 错误类
 *
 * 表示 FFI 调用过程中发生的错误
 */
export class FfiError extends RustyTypeScriptError {
    /**
     * FFI 错误类型
     */
    public readonly ffiErrorType: FfiErrorType;

    /**
     * 相关函数名称
     */
    public readonly functionName?: string;

    /**
     * 期望的类型
     */
    public readonly expectedType?: FfiType;

    /**
     * 实际的类型
     */
    public readonly actualType?: FfiType;

    /**
     * 参数索引
     */
    public readonly parameterIndex?: number;

    /**
     * 构造函数
     *
     * @param message 错误消息
     * @param ffiErrorType FFI 错误类型
     * @param options 错误选项
     */
    constructor(
        message: string,
        ffiErrorType: FfiErrorType,
        options?: {
            functionName?: string;
            expectedType?: FfiType;
            actualType?: FfiType;
            parameterIndex?: number;
            cause?: Error;
            suggestions?: string[];
        },
    ) {
        const defaultSuggestions: Record<FfiErrorType, string[]> = {
            FunctionNotFound: [
                "检查函数名称是否正确",
                "确认函数是否已注册",
                "检查函数名称的大小写",
            ],
            TypeMismatch: ["检查参数类型是否正确", "确认函数签名要求", "考虑使用类型转换"],
            InvalidSignature: ["检查函数签名是否正确", "确认参数数量和类型", "查看函数文档"],
            CallFailed: ["检查函数实现是否有错误", "确认参数值是否有效", "检查运行时环境"],
        };

        super(message, "ffi", {
            cause: options?.cause,
            code: `FFI_${ffiErrorType.toUpperCase()}`,
            suggestions: options?.suggestions ?? defaultSuggestions[ffiErrorType],
        });

        this.name = "FfiError";
        this.ffiErrorType = ffiErrorType;
        this.functionName = options?.functionName;
        this.expectedType = options?.expectedType;
        this.actualType = options?.actualType;
        this.parameterIndex = options?.parameterIndex;
    }

    /**
     * 创建函数未找到错误
     *
     * @param functionName 函数名称
     * @returns FfiError 实例
     */
    public static functionNotFound(functionName: string): FfiError {
        return new FfiError(`FFI 函数未找到: ${functionName}`, "FunctionNotFound", {
            functionName,
        });
    }

    /**
     * 创建类型不匹配错误
     *
     * @param functionName 函数名称
     * @param parameterIndex 参数索引
     * @param expectedType 期望类型
     * @param actualType 实际类型
     * @returns FfiError 实例
     */
    public static typeMismatch(
        functionName: string,
        parameterIndex: number,
        expectedType: FfiType,
        actualType: FfiType,
    ): FfiError {
        return new FfiError(
            `FFI 函数 "${functionName}" 参数 ${parameterIndex} 类型不匹配: 期望 ${expectedType}, 实际 ${actualType}`,
            "TypeMismatch",
            {
                functionName,
                parameterIndex,
                expectedType,
                actualType,
            },
        );
    }

    /**
     * 创建无效签名错误
     *
     * @param functionName 函数名称
     * @param reason 原因
     * @returns FfiError 实例
     */
    public static invalidSignature(functionName: string, reason: string): FfiError {
        return new FfiError(`FFI 函数 "${functionName}" 签名无效: ${reason}`, "InvalidSignature", {
            functionName,
        });
    }

    /**
     * 创建调用失败错误
     *
     * @param functionName 函数名称
     * @param cause 原始错误
     * @returns FfiError 实例
     */
    public static callFailed(functionName: string, cause?: Error): FfiError {
        return new FfiError(`FFI 函数 "${functionName}" 调用失败`, "CallFailed", {
            functionName,
            cause,
        });
    }

    /**
     * 获取完整的错误信息
     *
     * @returns 格式化的错误信息
     */
    public getFullMessage(): string {
        let fullMessage = `[${this.ffiErrorType}] ${this.message}`;

        if (this.functionName) {
            fullMessage += `\n  函数: ${this.functionName}`;
        }

        if (this.parameterIndex !== undefined) {
            fullMessage += `\n  参数索引: ${this.parameterIndex}`;
        }

        if (this.expectedType && this.actualType) {
            fullMessage += `\n  期望类型: ${this.expectedType}`;
            fullMessage += `\n  实际类型: ${this.actualType}`;
        }

        if (this.suggestions.length > 0) {
            fullMessage += "\n  建议:";
            this.suggestions.forEach((s, i) => {
                fullMessage += `\n    ${i + 1}. ${s}`;
            });
        }

        return fullMessage;
    }
}

/**
 * 自定义类型转换器接口
 *
 * 定义自定义类型转换器的结构
 */
export interface FfiTypeConverterFn {
    /**
     * 将值转换为目标类型
     *
     * @param value 输入值
     * @returns 转换后的值
     */
    (value: unknown): unknown;
}

/**
 * FFI 类型转换器类
 *
 * 处理 FFI 调用时的类型转换
 */
export class FfiTypeConverter {
    /**
     * 自定义类型转换器映射
     */
    private customConverters: Map<string, FfiTypeConverterFn> = new Map();

    /**
     * 注册自定义类型转换器
     *
     * @param typeName 类型名称
     * @param converter 转换器函数
     */
    public registerCustomConverter(typeName: string, converter: FfiTypeConverterFn): void {
        this.customConverters.set(typeName, converter);
    }

    /**
     * 注销自定义类型转换器
     *
     * @param typeName 类型名称
     */
    public unregisterCustomConverter(typeName: string): void {
        this.customConverters.delete(typeName);
    }

    /**
     * 检查是否存在自定义类型转换器
     *
     * @param typeName 类型名称
     * @returns 是否存在
     */
    public hasCustomConverter(typeName: string): boolean {
        return this.customConverters.has(typeName);
    }

    /**
     * 获取值的 FFI 类型
     *
     * @param value 输入值
     * @returns FFI 类型
     */
    public getType(value: unknown): FfiType {
        if (value === null) {
            return FfiType.Null;
        }

        if (value === undefined) {
            return FfiType.Undefined;
        }

        if (typeof value === "string") {
            return FfiType.String;
        }

        if (typeof value === "number") {
            return FfiType.Number;
        }

        if (typeof value === "boolean") {
            return FfiType.Boolean;
        }

        if (typeof value === "function") {
            return FfiType.Function;
        }

        if (Array.isArray(value)) {
            return FfiType.Array;
        }

        if (typeof value === "object") {
            return FfiType.Object;
        }

        return FfiType.Any;
    }

    /**
     * 检查类型是否匹配
     *
     * @param value 输入值
     * @param expectedType 期望类型
     * @returns 是否匹配
     */
    public isTypeMatch(value: unknown, expectedType: FfiType): boolean {
        if (expectedType === FfiType.Any) {
            return true;
        }

        if (expectedType === FfiType.Void) {
            return value === undefined || value === null;
        }

        const actualType = this.getType(value);

        if (actualType === expectedType) {
            return true;
        }

        if (expectedType === FfiType.Null && value === null) {
            return true;
        }

        if (expectedType === FfiType.Undefined && value === undefined) {
            return true;
        }

        if (expectedType === FfiType.Object && actualType === FfiType.Null) {
            return true;
        }

        return false;
    }

    /**
     * 转换值为指定类型
     *
     * @param value 输入值
     * @param targetType 目标类型
     * @returns 转换后的值
     */
    public convert(value: unknown, targetType: FfiType): unknown {
        if (targetType === FfiType.Any) {
            return value;
        }

        if (value === null || value === undefined) {
            if (targetType === FfiType.Null || targetType === FfiType.Undefined) {
                return value;
            }
            return null;
        }

        switch (targetType) {
            case FfiType.String:
                return this.convertToString(value);

            case FfiType.Number:
                return this.convertToNumber(value);

            case FfiType.Boolean:
                return this.convertToBoolean(value);

            case FfiType.Array:
                return this.convertToArray(value);

            case FfiType.Object:
                return this.convertToObject(value);

            default:
                return value;
        }
    }

    /**
     * 转换为字符串类型
     *
     * @param value 输入值
     * @returns 字符串值
     */
    private convertToString(value: unknown): string {
        if (typeof value === "string") {
            return value;
        }

        if (value === null || value === undefined) {
            return "";
        }

        if (typeof value === "object") {
            return JSON.stringify(value);
        }

        return String(value);
    }

    /**
     * 转换为数字类型
     *
     * @param value 输入值
     * @returns 数字值
     */
    private convertToNumber(value: unknown): number {
        if (typeof value === "number") {
            return value;
        }

        if (typeof value === "string") {
            const num = parseFloat(value);
            return isNaN(num) ? 0 : num;
        }

        if (typeof value === "boolean") {
            return value ? 1 : 0;
        }

        return 0;
    }

    /**
     * 转换为布尔类型
     *
     * @param value 输入值
     * @returns 布尔值
     */
    private convertToBoolean(value: unknown): boolean {
        if (typeof value === "boolean") {
            return value;
        }

        if (typeof value === "string") {
            const lowerStr = value.toLowerCase().trim();
            return lowerStr === "true" || lowerStr === "1" || lowerStr === "yes";
        }

        if (typeof value === "number") {
            return value !== 0;
        }

        return Boolean(value);
    }

    /**
     * 转换为数组类型
     *
     * @param value 输入值
     * @returns 数组值
     */
    private convertToArray(value: unknown): unknown[] {
        if (Array.isArray(value)) {
            return value;
        }

        if (value === null || value === undefined) {
            return [];
        }

        if (typeof value === "string") {
            try {
                const parsed = JSON.parse(value);
                if (Array.isArray(parsed)) {
                    return parsed;
                }
            } catch {
                // 忽略解析错误
            }
        }

        return [value];
    }

    /**
     * 转换为对象类型
     *
     * @param value 输入值
     * @returns 对象值
     */
    private convertToObject(value: unknown): Record<string, unknown> {
        if (value === null || value === undefined) {
            return {};
        }

        if (typeof value === "object" && !Array.isArray(value)) {
            return value as Record<string, unknown>;
        }

        if (typeof value === "string") {
            try {
                const parsed = JSON.parse(value);
                if (typeof parsed === "object" && !Array.isArray(parsed)) {
                    return parsed;
                }
            } catch {
                // 忽略解析错误
            }
        }

        if (Array.isArray(value)) {
            const obj: Record<string, unknown> = {};
            value.forEach((item, index) => {
                obj[index.toString()] = item;
            });
            return obj;
        }

        return { value };
    }

    /**
     * 使用自定义转换器转换值
     *
     * @param value 输入值
     * @param typeName 类型名称
     * @returns 转换后的值
     */
    public convertWithCustom(value: unknown, typeName: string): unknown {
        const converter = this.customConverters.get(typeName);
        if (converter) {
            return converter(value);
        }
        return value;
    }

    /**
     * 验证参数类型
     *
     * @param functionName 函数名称
     * @param parameters 参数定义
     * @param args 实际参数
     * @returns 验证结果
     */
    public validateParameters(
        functionName: string,
        parameters: FfiParameter[],
        args: unknown[],
    ): { valid: boolean; error?: FfiError } {
        for (let i = 0; i < parameters.length; i++) {
            const param = parameters[i];
            const arg = args[i];

            if (arg === undefined) {
                if (!param.optional && param.defaultValue === undefined) {
                    return {
                        valid: false,
                        error: FfiError.typeMismatch(
                            functionName,
                            i,
                            param.type,
                            FfiType.Undefined,
                        ),
                    };
                }
                continue;
            }

            if (!this.isTypeMatch(arg, param.type)) {
                return {
                    valid: false,
                    error: FfiError.typeMismatch(functionName, i, param.type, this.getType(arg)),
                };
            }
        }

        return { valid: true };
    }

    /**
     * 转换参数类型
     *
     * @param parameters 参数定义
     * @param args 实际参数
     * @returns 转换后的参数
     */
    public convertParameters(parameters: FfiParameter[], args: unknown[]): unknown[] {
        const converted: unknown[] = [];

        for (let i = 0; i < parameters.length; i++) {
            const param = parameters[i];
            let arg = args[i];

            if (arg === undefined && param.defaultValue !== undefined) {
                arg = param.defaultValue;
            }

            if (arg !== undefined) {
                arg = this.convert(arg, param.type);
            }

            converted.push(arg);
        }

        return converted;
    }

    /**
     * 清除所有自定义转换器
     */
    public clear(): void {
        this.customConverters.clear();
    }
}

/**
 * FFI 注册表类
 *
 * 管理所有 FFI 函数的注册、注销和调用
 */
export class FfiRegistry {
    /**
     * 已注册的 FFI 函数映射
     */
    private functions: Map<string, FfiFunction> = new Map();

    /**
     * 类型转换器
     */
    private typeConverter: FfiTypeConverter;

    /**
     * 获取类型转换器
     */
    public get converter(): FfiTypeConverter {
        return this.typeConverter;
    }

    /**
     * 构造函数
     */
    constructor() {
        this.typeConverter = new FfiTypeConverter();
    }

    /**
     * 注册 FFI 函数
     *
     * @param ffiFunction FFI 函数信息
     */
    public register(ffiFunction: FfiFunction): void {
        if (this.functions.has(ffiFunction.name)) {
            console.warn(`FFI 函数 "${ffiFunction.name}" 已存在，将被覆盖`);
        }

        this.functions.set(ffiFunction.name, ffiFunction);
    }

    /**
     * 批量注册 FFI 函数
     *
     * @param ffiFunctions FFI 函数信息数组
     */
    public registerAll(ffiFunctions: FfiFunction[]): void {
        ffiFunctions.forEach((fn) => this.register(fn));
    }

    /**
     * 注销 FFI 函数
     *
     * @param name 函数名称
     * @returns 是否成功注销
     */
    public unregister(name: string): boolean {
        return this.functions.delete(name);
    }

    /**
     * 获取 FFI 函数
     *
     * @param name 函数名称
     * @returns FFI 函数信息，如果不存在则返回 undefined
     */
    public get(name: string): FfiFunction | undefined {
        return this.functions.get(name);
    }

    /**
     * 检查 FFI 函数是否已注册
     *
     * @param name 函数名称
     * @returns 是否已注册
     */
    public has(name: string): boolean {
        return this.functions.has(name);
    }

    /**
     * 获取所有已注册的函数名称
     *
     * @returns 函数名称数组
     */
    public getFunctionNames(): string[] {
        return Array.from(this.functions.keys());
    }

    /**
     * 获取所有已注册的函数
     *
     * @returns FFI 函数数组
     */
    public getAllFunctions(): FfiFunction[] {
        return Array.from(this.functions.values());
    }

    /**
     * 获取指定模块的函数
     *
     * @param moduleName 模块名称
     * @returns FFI 函数数组
     */
    public getFunctionsByModule(moduleName: string): FfiFunction[] {
        return this.getAllFunctions().filter((fn) => fn.module === moduleName);
    }

    /**
     * 验证函数签名
     *
     * @param name 函数名称
     * @param args 参数列表
     * @returns 验证结果
     */
    public validateSignature(
        name: string,
        args: unknown[],
    ): { valid: boolean; error?: FfiError; function?: FfiFunction } {
        const ffiFunction = this.functions.get(name);

        if (!ffiFunction) {
            return {
                valid: false,
                error: FfiError.functionNotFound(name),
            };
        }

        const { parameters, variadic } = ffiFunction.signature;

        if (!variadic && args.length > parameters.length) {
            return {
                valid: false,
                error: FfiError.invalidSignature(
                    name,
                    `参数数量过多: 期望 ${parameters.length}, 实际 ${args.length}`,
                ),
                function: ffiFunction,
            };
        }

        const requiredParams = parameters.filter((p) => !p.optional);
        if (args.length < requiredParams.length) {
            return {
                valid: false,
                error: FfiError.invalidSignature(
                    name,
                    `缺少必需参数: 期望至少 ${requiredParams.length} 个, 实际 ${args.length}`,
                ),
                function: ffiFunction,
            };
        }

        const validationResult = this.typeConverter.validateParameters(name, parameters, args);

        if (!validationResult.valid) {
            return {
                valid: false,
                error: validationResult.error,
                function: ffiFunction,
            };
        }

        return { valid: true, function: ffiFunction };
    }

    /**
     * 调用 FFI 函数
     *
     * @param name 函数名称
     * @param args 参数列表
     * @returns 调用结果
     */
    public call(name: string, ...args: unknown[]): unknown {
        const validation = this.validateSignature(name, args);

        if (!validation.valid) {
            throw validation.error;
        }

        const ffiFunction = validation.function!;

        try {
            const convertedArgs = this.typeConverter.convertParameters(
                ffiFunction.signature.parameters,
                args,
            );

            const result = ffiFunction.implementation(...convertedArgs);

            if (ffiFunction.signature.returnType !== FfiType.Void) {
                return this.typeConverter.convert(result, ffiFunction.signature.returnType);
            }

            return result;
        } catch (error) {
            if (error instanceof FfiError) {
                throw error;
            }

            throw FfiError.callFailed(
                name,
                error instanceof Error ? error : new Error(String(error)),
            );
        }
    }

    /**
     * 异步调用 FFI 函数
     *
     * @param name 函数名称
     * @param args 参数列表
     * @returns 调用结果的 Promise
     */
    public async callAsync(name: string, ...args: unknown[]): Promise<unknown> {
        const validation = this.validateSignature(name, args);

        if (!validation.valid) {
            throw validation.error;
        }

        const ffiFunction = validation.function!;

        try {
            const convertedArgs = this.typeConverter.convertParameters(
                ffiFunction.signature.parameters,
                args,
            );

            const result = await Promise.resolve(ffiFunction.implementation(...convertedArgs));

            if (ffiFunction.signature.returnType !== FfiType.Void) {
                return this.typeConverter.convert(result, ffiFunction.signature.returnType);
            }

            return result;
        } catch (error) {
            if (error instanceof FfiError) {
                throw error;
            }

            throw FfiError.callFailed(
                name,
                error instanceof Error ? error : new Error(String(error)),
            );
        }
    }

    /**
     * 创建函数包装器
     *
     * @param name 函数名称
     * @returns 函数包装器
     */
    public createWrapper(name: string): (...args: unknown[]) => unknown {
        return (...args: unknown[]) => this.call(name, ...args);
    }

    /**
     * 创建异步函数包装器
     *
     * @param name 函数名称
     * @returns 异步函数包装器
     */
    public createAsyncWrapper(name: string): (...args: unknown[]) => Promise<unknown> {
        return async (...args: unknown[]) => this.callAsync(name, ...args);
    }

    /**
     * 获取函数签名信息
     *
     * @param name 函数名称
     * @returns 签名信息字符串
     */
    public getSignatureString(name: string): string | null {
        const ffiFunction = this.functions.get(name);
        if (!ffiFunction) {
            return null;
        }

        const params = ffiFunction.signature.parameters
            .map((p) => {
                const optional = p.optional ? "?" : "";
                const defaultValue =
                    p.defaultValue !== undefined ? ` = ${JSON.stringify(p.defaultValue)}` : "";
                return `${p.name}${optional}: ${p.type}${defaultValue}`;
            })
            .join(", ");

        const variadic = ffiFunction.signature.variadic ? "...args: any[], " : "";

        return `${name}(${variadic}${params}): ${ffiFunction.signature.returnType}`;
    }

    /**
     * 清除所有已注册的函数
     */
    public clear(): void {
        this.functions.clear();
        this.typeConverter.clear();
    }

    /**
     * 获取已注册函数的数量
     *
     * @returns 函数数量
     */
    public get size(): number {
        return this.functions.size;
    }
}

/**
 * WASM 加载器配置选项
 */
export interface WasmLoaderOptions {
    /**
     * WASM 模块 URL（浏览器环境）
     */
    url?: string;

    /**
     * WASM 模块二进制数据（Node.js 环境）
     */
    buffer?: Buffer | ArrayBuffer;

    /**
     * 加载超时时间（毫秒）
     */
    timeout?: number;

    /**
     * 最大重试次数
     */
    maxRetries?: number;

    /**
     * 重试延迟时间（毫秒）
     */
    retryDelay?: number;

    /**
     * 是否启用流式编译
     */
    streamingCompilation?: boolean;

    /**
     * 加载进度回调
     */
    onProgress?: ProgressCallback;

    /**
     * 自定义请求头（用于 fetch）
     */
    headers?: Record<string, string>;

    /**
     * 自定义导入对象
     */
    imports?: WebAssembly.Imports;
}

/**
 * WASM 加载器类
 *
 * 负责加载 WASM 模块，支持多种加载方式和错误恢复机制
 */
export class WasmLoader {
    /**
     * 默认超时时间（毫秒）
     */
    public static readonly DEFAULT_TIMEOUT = 30000;

    /**
     * 默认最大重试次数
     */
    public static readonly DEFAULT_MAX_RETRIES = 3;

    /**
     * 默认重试延迟时间（毫秒）
     */
    public static readonly DEFAULT_RETRY_DELAY = 1000;

    /**
     * 当前加载状态
     */
    private _state: LoadingState = LoadingState.Unloaded;

    /**
     * 当前加载进度
     */
    private _progress: LoadingProgress;

    /**
     * 加载器配置
     */
    private readonly options: WasmLoaderOptions;

    /**
     * 已编译的 WASM 模块
     */
    private _module: WebAssembly.Module | null = null;

    /**
     * 已实例化的 WASM 实例
     */
    private _instance: WebAssembly.Instance | null = null;

    /**
     * 获取当前加载状态
     */
    public get state(): LoadingState {
        return this._state;
    }

    /**
     * 获取当前加载进度
     */
    public get progress(): LoadingProgress {
        return { ...this._progress };
    }

    /**
     * 获取已编译的 WASM 模块
     */
    public get module(): WebAssembly.Module | null {
        return this._module;
    }

    /**
     * 获取已实例化的 WASM 实例
     */
    public get instance(): WebAssembly.Instance | null {
        return this._instance;
    }

    /**
     * 构造函数
     *
     * @param options 加载器配置选项
     */
    constructor(options: WasmLoaderOptions = {}) {
        this.options = {
            timeout: WasmLoader.DEFAULT_TIMEOUT,
            maxRetries: WasmLoader.DEFAULT_MAX_RETRIES,
            retryDelay: WasmLoader.DEFAULT_RETRY_DELAY,
            streamingCompilation: true,
            ...options,
        };

        this._progress = {
            state: LoadingState.Unloaded,
            progress: 0,
            loadedBytes: 0,
            stage: "初始化",
        };
    }

    /**
     * 更新加载进度
     *
     * @param update 进度更新内容
     */
    private updateProgress(update: Partial<LoadingProgress>): void {
        this._progress = {
            ...this._progress,
            ...update,
        };

        if (update.state !== undefined) {
            this._state = update.state;
        }

        if (this.options.onProgress) {
            this.options.onProgress(this._progress);
        }
    }

    /**
     * 创建超时 Promise
     *
     * @param ms 超时时间（毫秒）
     * @returns Promise
     */
    private createTimeoutPromise<T>(ms: number): Promise<T> {
        return new Promise((_, reject) => {
            setTimeout(() => {
                reject(WasmLoadError.timeoutError(`加载超时（${ms}ms）`, this.options.url));
            }, ms);
        });
    }

    /**
     * 延迟执行
     *
     * @param ms 延迟时间（毫秒）
     */
    private delay(ms: number): Promise<void> {
        return new Promise((resolve) => setTimeout(resolve, ms));
    }

    /**
     * 检测是否支持流式编译
     *
     * @returns 是否支持流式编译
     */
    private supportsStreamingCompilation(): boolean {
        return (
            typeof WebAssembly.compileStreaming === "function" &&
            this.options.streamingCompilation !== false
        );
    }

    /**
     * 从 URL 加载 WASM 模块
     *
     * @param url WASM 文件 URL
     * @param retryCount 当前重试次数
     * @returns WASM 模块
     */
    private async loadFromUrl(url: string, retryCount: number = 0): Promise<WebAssembly.Module> {
        this.updateProgress({
            state: LoadingState.Loading,
            stage: "正在从 URL 加载",
            startTime: Date.now(),
        });

        try {
            const timeoutMs = this.options.timeout ?? WasmLoader.DEFAULT_TIMEOUT;

            if (this.supportsStreamingCompilation()) {
                return await this.loadWithStreaming(url, timeoutMs);
            }

            return await this.loadWithFetch(url, timeoutMs);
        } catch (error) {
            if (error instanceof WasmLoadError) {
                if (error.canRetry() && error.loadErrorType !== "CompileError") {
                    this.updateProgress({
                        stage: `加载失败，正在重试 (${retryCount + 1}/${this.options.maxRetries})`,
                        error,
                    });

                    await this.delay(this.options.retryDelay ?? WasmLoader.DEFAULT_RETRY_DELAY);
                    return this.loadFromUrl(url, retryCount + 1);
                }
                throw error;
            }

            const wasmError = WasmLoadError.networkError(
                `加载 WASM 模块失败: ${error instanceof Error ? error.message : String(error)}`,
                url,
                error instanceof Error ? error : new Error(String(error)),
            );

            if (retryCount < (this.options.maxRetries ?? WasmLoader.DEFAULT_MAX_RETRIES)) {
                this.updateProgress({
                    stage: `加载失败，正在重试 (${retryCount + 1}/${this.options.maxRetries})`,
                    error: wasmError,
                });

                await this.delay(this.options.retryDelay ?? WasmLoader.DEFAULT_RETRY_DELAY);
                return this.loadFromUrl(url, retryCount + 1);
            }

            throw wasmError.withRetry(retryCount);
        }
    }

    /**
     * 使用流式编译加载
     *
     * @param url WASM 文件 URL
     * @param timeoutMs 超时时间
     * @returns WASM 模块
     */
    private async loadWithStreaming(url: string, timeoutMs: number): Promise<WebAssembly.Module> {
        this.updateProgress({ stage: "正在流式编译" });

        const fetchPromise = fetch(url, {
            headers: this.options.headers,
        });

        const response = await Promise.race([
            fetchPromise,
            this.createTimeoutPromise<Response>(timeoutMs),
        ]);

        if (!response.ok) {
            throw WasmLoadError.networkError(
                `HTTP 错误: ${response.status} ${response.statusText}`,
                url,
            );
        }

        const contentLength = response.headers.get("content-length");
        const totalBytes = contentLength ? parseInt(contentLength, 10) : undefined;

        this.updateProgress({
            totalBytes,
            stage: "正在流式编译",
        });

        const modulePromise = WebAssembly.compileStreaming(response);

        const module = await Promise.race([
            modulePromise,
            this.createTimeoutPromise<WebAssembly.Module>(timeoutMs),
        ]);

        this.updateProgress({
            progress: 100,
            loadedBytes: totalBytes ?? 0,
            stage: "流式编译完成",
        });

        return module;
    }

    /**
     * 使用 fetch 加载
     *
     * @param url WASM 文件 URL
     * @param timeoutMs 超时时间
     * @returns WASM 模块
     */
    private async loadWithFetch(url: string, timeoutMs: number): Promise<WebAssembly.Module> {
        this.updateProgress({ stage: "正在下载" });

        const fetchPromise = fetch(url, {
            headers: this.options.headers,
        });

        const response = await Promise.race([
            fetchPromise,
            this.createTimeoutPromise<Response>(timeoutMs),
        ]);

        if (!response.ok) {
            throw WasmLoadError.networkError(
                `HTTP 错误: ${response.status} ${response.statusText}`,
                url,
            );
        }

        const contentLength = response.headers.get("content-length");
        const totalBytes = contentLength ? parseInt(contentLength, 10) : undefined;

        this.updateProgress({
            totalBytes,
            stage: "正在下载",
        });

        const reader = response.body?.getReader();
        if (!reader) {
            const arrayBuffer = await response.arrayBuffer();
            this.updateProgress({
                progress: 50,
                loadedBytes: arrayBuffer.byteLength,
                stage: "正在编译",
            });

            const module = await WebAssembly.compile(arrayBuffer);
            this.updateProgress({
                progress: 100,
                stage: "编译完成",
            });
            return module;
        }

        const chunks: Uint8Array[] = [];
        let loadedBytes = 0;

        while (true) {
            const readPromise = reader.read();
            const { done, value } = await Promise.race([
                readPromise,
                this.createTimeoutPromise<ReadableStreamReadResult<Uint8Array>>(timeoutMs),
            ]);

            if (done) break;

            chunks.push(value);
            loadedBytes += value.length;

            const progress = totalBytes ? Math.round((loadedBytes / totalBytes) * 50) : 50;
            this.updateProgress({
                progress,
                loadedBytes,
                stage: "正在下载",
            });
        }

        const arrayBuffer = new Uint8Array(loadedBytes);
        let offset = 0;
        for (const chunk of chunks) {
            arrayBuffer.set(chunk, offset);
            offset += chunk.length;
        }

        this.updateProgress({
            progress: 50,
            loadedBytes,
            stage: "正在编译",
        });

        try {
            const module = await WebAssembly.compile(arrayBuffer.buffer);
            this.updateProgress({
                progress: 100,
                stage: "编译完成",
            });
            return module;
        } catch (error) {
            throw WasmLoadError.compileError(
                `编译 WASM 模块失败: ${error instanceof Error ? error.message : String(error)}`,
                error instanceof Error ? error : new Error(String(error)),
            );
        }
    }

    /**
     * 从 Buffer 加载 WASM 模块
     *
     * @param buffer WASM 二进制数据
     * @returns WASM 模块
     */
    private async loadFromBuffer(buffer: Buffer | ArrayBuffer): Promise<WebAssembly.Module> {
        this.updateProgress({
            state: LoadingState.Loading,
            stage: "正在从 Buffer 加载",
            startTime: Date.now(),
        });

        const arrayBuffer = buffer instanceof Buffer ? new Uint8Array(buffer).buffer : buffer;

        this.updateProgress({
            progress: 50,
            loadedBytes: arrayBuffer.byteLength,
            totalBytes: arrayBuffer.byteLength,
            stage: "正在编译",
        });

        try {
            const module = await WebAssembly.compile(arrayBuffer);
            this.updateProgress({
                progress: 100,
                stage: "编译完成",
            });
            return module;
        } catch (error) {
            throw WasmLoadError.compileError(
                `编译 WASM 模块失败: ${error instanceof Error ? error.message : String(error)}`,
                error instanceof Error ? error : new Error(String(error)),
            );
        }
    }

    /**
     * 实例化 WASM 模块
     *
     * @param module WASM 模块
     * @param imports 导入对象
     * @returns WASM 实例
     */
    private async instantiateModule(
        module: WebAssembly.Module,
        imports: WebAssembly.Imports = {},
    ): Promise<WebAssembly.Instance> {
        this.updateProgress({ stage: "正在实例化" });

        try {
            const instance = await WebAssembly.instantiate(module, imports);
            this.updateProgress({
                progress: 100,
                stage: "实例化完成",
            });
            return instance;
        } catch (error) {
            throw WasmLoadError.instantiateError(
                `实例化 WASM 模块失败: ${error instanceof Error ? error.message : String(error)}`,
                error instanceof Error ? error : new Error(String(error)),
            );
        }
    }

    /**
     * 加载 WASM 模块
     *
     * @returns WASM 实例
     */
    public async load(): Promise<WebAssembly.Instance> {
        if (this._instance) {
            return this._instance;
        }

        this.updateProgress({
            state: LoadingState.Loading,
            progress: 0,
            loadedBytes: 0,
            stage: "开始加载",
            startTime: Date.now(),
            error: undefined,
        });

        try {
            let module: WebAssembly.Module;

            if (this.options.buffer) {
                module = await this.loadFromBuffer(this.options.buffer);
            } else if (this.options.url) {
                module = await this.loadFromUrl(this.options.url);
            } else {
                throw new WasmLoadError("未指定 WASM 模块来源（URL 或 Buffer）", "NetworkError", {
                    suggestions: ["请提供 wasmUrl 或 wasmModule 选项"],
                });
            }

            this._module = module;

            const instance = await this.instantiateModule(module, this.options.imports);
            this._instance = instance;

            this.updateProgress({
                state: LoadingState.Loaded,
                progress: 100,
                endTime: Date.now(),
                stage: "加载完成",
            });

            return instance;
        } catch (error) {
            this.updateProgress({
                state: LoadingState.Error,
                error: error instanceof Error ? error : new Error(String(error)),
                endTime: Date.now(),
                stage: "加载失败",
            });

            throw error;
        }
    }

    /**
     * 重置加载器状态
     */
    public reset(): void {
        this._module = null;
        this._instance = null;
        this._state = LoadingState.Unloaded;
        this._progress = {
            state: LoadingState.Unloaded,
            progress: 0,
            loadedBytes: 0,
            stage: "初始化",
        };
    }

    /**
     * 检查是否已加载
     *
     * @returns 是否已加载
     */
    public isLoaded(): boolean {
        return this._state === LoadingState.Loaded && this._instance !== null;
    }

    /**
     * 检查是否正在加载
     *
     * @returns 是否正在加载
     */
    public isLoading(): boolean {
        return this._state === LoadingState.Loading;
    }

    /**
     * 获取加载耗时
     *
     * @returns 加载耗时（毫秒），如果未完成则返回 null
     */
    public getLoadDuration(): number | null {
        if (this._progress.startTime && this._progress.endTime) {
            return this._progress.endTime - this._progress.startTime;
        }
        return null;
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

    /**
     * 加载超时时间（毫秒）
     */
    timeout?: number;

    /**
     * 最大重试次数
     */
    maxRetries?: number;

    /**
     * 重试延迟时间（毫秒）
     */
    retryDelay?: number;

    /**
     * 是否启用流式编译
     */
    streamingCompilation?: boolean;

    /**
     * 加载进度回调
     */
    onProgress?: ProgressCallback;

    /**
     * 自定义请求头
     */
    headers?: Record<string, string>;
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
    /**
     * WASM 模块实例
     */
    private wasmModule: WasmModule | null = null;

    /**
     * 自定义控制台
     */
    private console: Console;

    /**
     * WASM 加载器
     */
    private loader: WasmLoader | null = null;

    /**
     * 加载进度
     */
    private _loadingProgress: LoadingProgress | null = null;

    /**
     * FFI 注册表
     */
    private ffiRegistry: FfiRegistry;

    /**
     * 获取当前加载进度
     */
    public get loadingProgress(): LoadingProgress | null {
        return this._loadingProgress ? { ...this._loadingProgress } : null;
    }

    /**
     * 获取加载状态
     */
    public get loadingState(): LoadingState {
        return this.loader?.state ?? LoadingState.Unloaded;
    }

    /**
     * 获取 FFI 注册表
     */
    public get ffi(): FfiRegistry {
        return this.ffiRegistry;
    }

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

        this.ffiRegistry = new FfiRegistry();
    }

    /**
     * 创建 WASM 导入对象
     *
     * @returns WebAssembly 导入对象
     */
    private createImports(): WebAssembly.Imports {
        return {
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
    }

    /**
     * 加载 WASM 模块
     *
     * @param options 初始化选项
     */
    private async loadWasmModule(options: InitOptions): Promise<void> {
        const loaderOptions: WasmLoaderOptions = {
            url: options.wasmUrl,
            buffer: options.wasmModule,
            timeout: options.timeout,
            maxRetries: options.maxRetries,
            retryDelay: options.retryDelay,
            streamingCompilation: options.streamingCompilation,
            headers: options.headers,
            imports: this.createImports(),
            onProgress: (progress: LoadingProgress) => {
                this._loadingProgress = progress;
                if (options.onProgress) {
                    options.onProgress(progress);
                }
            },
        };

        this.loader = new WasmLoader(loaderOptions);

        try {
            const instance = await this.loader.load();
            this.wasmModule = instance.exports as unknown as WasmModule;

            if (this.wasmModule.initialize) {
                this.wasmModule.initialize();
            }
        } catch (error) {
            if (error instanceof WasmLoadError) {
                throw error;
            }

            throw new RustyTypeScriptError("无法加载 TypeScript WASI 模块", "runtime", {
                cause: error instanceof Error ? error : new Error(String(error)),
                code: "WASM_LOAD_FAILED",
                suggestions: [
                    "检查 WASM 文件路径是否正确",
                    "确保浏览器支持 WebAssembly",
                    "检查网络连接是否正常",
                ],
            });
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
            throw new RustyTypeScriptError("WASM 模块未初始化", "runtime", {
                code: "WASM_NOT_INITIALIZED",
                suggestions: [
                    "请先调用 RustyTypeScript.init() 初始化模块",
                    "检查初始化过程中是否有错误",
                ],
            });
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
            throw new RustyTypeScriptError("WASM 模块未初始化", "runtime", {
                code: "WASM_NOT_INITIALIZED",
                suggestions: [
                    "请先调用 RustyTypeScript.init() 初始化模块",
                    "检查初始化过程中是否有错误",
                ],
            });
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
            throw new RustyTypeScriptError("WASM 模块未初始化", "runtime", {
                code: "WASM_NOT_INITIALIZED",
                suggestions: [
                    "请先调用 RustyTypeScript.init() 初始化模块",
                    "检查初始化过程中是否有错误",
                ],
            });
        }

        if (!code || code.trim().length === 0) {
            throw new RustyTypeScriptError("代码不能为空", "syntax", {
                code: "EMPTY_CODE",
                suggestions: ["请提供有效的 TypeScript 代码"],
            });
        }

        try {
            const ptr = this.stringToPtr(code);
            const resultPtr = this.wasmModule.compile_typescript(ptr);
            const result = this.getString(resultPtr);
            this.wasmModule.free_memory(resultPtr);

            if (result.startsWith("error:") || result.startsWith("Error:")) {
                throw new RustyTypeScriptError("编译失败", "syntax", {
                    code: "COMPILE_FAILED",
                    context: { code: code.substring(0, 500) },
                    suggestions: [
                        "检查代码语法是否正确",
                        "确保所有类型定义完整",
                        "检查是否有未闭合的括号或大括号",
                    ],
                });
            }

            return result;
        } catch (error) {
            if (error instanceof RustyTypeScriptError) {
                throw error;
            }
            throw new RustyTypeScriptError("编译过程中发生错误", "syntax", {
                cause: error instanceof Error ? error : new Error(String(error)),
                code: "COMPILE_ERROR",
                context: { code: code.substring(0, 500) },
                suggestions: ["检查代码语法是否正确", "确保 WASM 模块正常工作"],
            });
        }
    }

    /**
     * 执行 TypeScript 代码
     *
     * @param code TypeScript 代码字符串
     * @returns 执行结果
     */
    public async execute(code: string): Promise<ExecuteResult> {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM 模块未初始化", "runtime", {
                code: "WASM_NOT_INITIALIZED",
                suggestions: [
                    "请先调用 RustyTypeScript.init() 初始化模块",
                    "检查初始化过程中是否有错误",
                ],
            });
        }

        if (!code || code.trim().length === 0) {
            throw new RustyTypeScriptError("代码不能为空", "syntax", {
                code: "EMPTY_CODE",
                suggestions: ["请提供有效的 TypeScript 代码"],
            });
        }

        const startTime = performance.now();

        try {
            const ptr = this.stringToPtr(code);
            const resultPtr = this.wasmModule.execute_typescript(ptr);
            const resultStr = this.getString(resultPtr);
            const endTime = performance.now();

            this.wasmModule.free_memory(resultPtr);

            if (resultStr.startsWith("error:") || resultStr.startsWith("Error:")) {
                throw new RustyTypeScriptError("执行失败: " + resultStr, "runtime", {
                    code: "EXECUTE_FAILED",
                    context: { code: code.substring(0, 500) },
                    suggestions: [
                        "检查代码逻辑是否正确",
                        "确保所有变量已定义",
                        "检查是否有运行时错误",
                    ],
                });
            }

            try {
                const result = JSON.parse(resultStr);
                if (result.error) {
                    throw new RustyTypeScriptError(result.error, "runtime", {
                        code: "RUNTIME_ERROR",
                        context: { code: code.substring(0, 500) },
                        suggestions: ["检查代码逻辑是否正确", "确保所有变量已定义"],
                    });
                }
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
        } catch (error) {
            if (error instanceof RustyTypeScriptError) {
                throw error;
            }
            throw new RustyTypeScriptError("执行过程中发生错误", "runtime", {
                cause: error instanceof Error ? error : new Error(String(error)),
                code: "EXECUTE_ERROR",
                context: { code: code.substring(0, 500) },
                suggestions: ["检查代码逻辑是否正确", "确保 WASM 模块正常工作"],
            });
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
            throw new RustyTypeScriptError("WASM 模块未初始化", "runtime", {
                code: "WASM_NOT_INITIALIZED",
                suggestions: [
                    "请先调用 RustyTypeScript.init() 初始化模块",
                    "检查初始化过程中是否有错误",
                ],
            });
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
            throw new RustyTypeScriptError("WASM 模块未初始化", "runtime", {
                code: "WASM_NOT_INITIALIZED",
                suggestions: [
                    "请先调用 RustyTypeScript.init() 初始化模块",
                    "检查初始化过程中是否有错误",
                ],
            });
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
            throw new RustyTypeScriptError("WASM 模块未初始化", "runtime", {
                code: "WASM_NOT_INITIALIZED",
                suggestions: [
                    "请先调用 RustyTypeScript.init() 初始化模块",
                    "检查初始化过程中是否有错误",
                ],
            });
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
            throw new RustyTypeScriptError("WASM 模块未初始化", "runtime", {
                code: "WASM_NOT_INITIALIZED",
                suggestions: [
                    "请先调用 RustyTypeScript.init() 初始化模块",
                    "检查初始化过程中是否有错误",
                ],
            });
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
            throw new RustyTypeScriptError("WASM 模块未初始化", "runtime", {
                code: "WASM_NOT_INITIALIZED",
                suggestions: [
                    "请先调用 RustyTypeScript.init() 初始化模块",
                    "检查初始化过程中是否有错误",
                ],
            });
        }

        const namePtr = this.stringToPtr(name);
        const aliasPtr = alias ? this.stringToPtr(alias) : 0;
        const resultPtr = this.wasmModule.import_module(namePtr, aliasPtr);
        const result = this.getString(resultPtr);
        this.wasmModule.free_memory(resultPtr);

        if (result !== "ok") {
            throw new RustyTypeScriptError(`导入模块失败: ${result}`, "module", {
                code: "MODULE_IMPORT_FAILED",
                context: { metadata: { moduleName: name, alias } },
                suggestions: ["检查模块名称是否正确", "确保模块已正确安装", "检查模块路径是否有效"],
            });
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
            throw new RustyTypeScriptError("WASM 模块未初始化", "runtime", {
                code: "WASM_NOT_INITIALIZED",
                suggestions: [
                    "请先调用 RustyTypeScript.init() 初始化模块",
                    "检查初始化过程中是否有错误",
                ],
            });
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
            throw new RustyTypeScriptError("WASM 模块未初始化", "runtime", {
                code: "WASM_NOT_INITIALIZED",
                suggestions: [
                    "请先调用 RustyTypeScript.init() 初始化模块",
                    "检查初始化过程中是否有错误",
                ],
            });
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
            throw new RustyTypeScriptError("WASM 模块未初始化", "runtime", {
                code: "WASM_NOT_INITIALIZED",
                suggestions: [
                    "请先调用 RustyTypeScript.init() 初始化模块",
                    "检查初始化过程中是否有错误",
                ],
            });
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
            throw new RustyTypeScriptError("WASM 模块未初始化", "runtime", {
                code: "WASM_NOT_INITIALIZED",
                suggestions: [
                    "请先调用 RustyTypeScript.init() 初始化模块",
                    "检查初始化过程中是否有错误",
                ],
            });
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
            throw new RustyTypeScriptError("WASM 模块未初始化", "runtime", {
                code: "WASM_NOT_INITIALIZED",
                suggestions: [
                    "请先调用 RustyTypeScript.init() 初始化模块",
                    "检查初始化过程中是否有错误",
                ],
            });
        }

        this.wasmModule.enable_performance_monitoring();
    }

    /**
     * 禁用性能监控
     */
    public async disablePerformanceMonitoring(): Promise<void> {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM 模块未初始化", "runtime", {
                code: "WASM_NOT_INITIALIZED",
                suggestions: [
                    "请先调用 RustyTypeScript.init() 初始化模块",
                    "检查初始化过程中是否有错误",
                ],
            });
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
            throw new RustyTypeScriptError("WASM 模块未初始化", "runtime", {
                code: "WASM_NOT_INITIALIZED",
                suggestions: [
                    "请先调用 RustyTypeScript.init() 初始化模块",
                    "检查初始化过程中是否有错误",
                ],
            });
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
            throw new RustyTypeScriptError("WASM 模块未初始化", "runtime", {
                code: "WASM_NOT_INITIALIZED",
                suggestions: [
                    "请先调用 RustyTypeScript.init() 初始化模块",
                    "检查初始化过程中是否有错误",
                ],
            });
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
            throw new RustyTypeScriptError("WASM 模块未初始化", "runtime", {
                code: "WASM_NOT_INITIALIZED",
                suggestions: [
                    "请先调用 RustyTypeScript.init() 初始化模块",
                    "检查初始化过程中是否有错误",
                ],
            });
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
            throw new RustyTypeScriptError("WASM 模块未初始化", "runtime", {
                code: "WASM_NOT_INITIALIZED",
                suggestions: [
                    "请先调用 RustyTypeScript.init() 初始化模块",
                    "检查初始化过程中是否有错误",
                ],
            });
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
            throw new RustyTypeScriptError("WASM 模块未初始化", "runtime", {
                code: "WASM_NOT_INITIALIZED",
                suggestions: [
                    "请先调用 RustyTypeScript.init() 初始化模块",
                    "检查初始化过程中是否有错误",
                ],
            });
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
            throw new RustyTypeScriptError("WASM 模块未初始化", "runtime", {
                code: "WASM_NOT_INITIALIZED",
                suggestions: [
                    "请先调用 RustyTypeScript.init() 初始化模块",
                    "检查初始化过程中是否有错误",
                ],
            });
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
            throw new RustyTypeScriptError("WASM 模块未初始化", "runtime", {
                code: "WASM_NOT_INITIALIZED",
                suggestions: [
                    "请先调用 RustyTypeScript.init() 初始化模块",
                    "检查初始化过程中是否有错误",
                ],
            });
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
     * @param signature 函数签名（可选）
     * @param description 函数描述（可选）
     */
    public async registerFfiFunction(
        name: string,
        func: (...args: unknown[]) => unknown,
        signature?: FfiSignature,
        description?: string,
    ): Promise<void> {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM 模块未初始化", "runtime", {
                code: "WASM_NOT_INITIALIZED",
                suggestions: [
                    "请先调用 RustyTypeScript.init() 初始化模块",
                    "检查初始化过程中是否有错误",
                ],
            });
        }

        try {
            const ffiFunction: FfiFunction = {
                name,
                signature: signature ?? {
                    parameters: [],
                    returnType: FfiType.Any,
                    variadic: true,
                },
                implementation: func,
                description,
            };

            this.ffiRegistry.register(ffiFunction);

            const namePtr = this.stringToPtr(name);
            const funcPtr = this.stringToPtr(JSON.stringify({}));
            this.wasmModule.register_ffi_function(namePtr, funcPtr);
        } catch (error) {
            if (error instanceof FfiError) {
                throw error;
            }
            throw new RustyTypeScriptError(`注册 FFI 函数失败: ${name}`, "ffi", {
                cause: error instanceof Error ? error : new Error(String(error)),
                code: "FFI_REGISTER_FAILED",
                context: { metadata: { functionName: name } },
                suggestions: [
                    "检查函数名称是否有效",
                    "确保函数签名正确",
                    "检查 WASM 模块是否支持 FFI",
                ],
            });
        }
    }

    /**
     * 注销 FFI 函数
     *
     * @param name 函数名称
     * @returns 是否成功注销
     */
    public unregisterFfiFunction(name: string): boolean {
        return this.ffiRegistry.unregister(name);
    }

    /**
     * 检查 FFI 函数是否已注册
     *
     * @param name 函数名称
     * @returns 是否已注册
     */
    public hasFfiFunction(name: string): boolean {
        return this.ffiRegistry.has(name);
    }

    /**
     * 获取 FFI 函数信息
     *
     * @param name 函数名称
     * @returns FFI 函数信息
     */
    public getFfiFunction(name: string): FfiFunction | undefined {
        return this.ffiRegistry.get(name);
    }

    /**
     * 调用 FFI 函数
     *
     * @param name 函数名称
     * @param args 参数列表
     * @returns 调用结果
     */
    public callFfiFunction(name: string, ...args: unknown[]): unknown {
        return this.ffiRegistry.call(name, ...args);
    }

    /**
     * 异步调用 FFI 函数
     *
     * @param name 函数名称
     * @param args 参数列表
     * @returns 调用结果
     */
    public async callFfiFunctionAsync(name: string, ...args: unknown[]): Promise<unknown> {
        return this.ffiRegistry.callAsync(name, ...args);
    }

    /**
     * 获取所有已注册的 FFI 函数名称
     *
     * @returns 函数名称数组
     */
    public getFfiFunctionNames(): string[] {
        return this.ffiRegistry.getFunctionNames();
    }

    /**
     * 注册自定义类型转换器
     *
     * @param typeName 类型名称
     * @param converter 转换器函数
     */
    public registerTypeConverter(typeName: string, converter: FfiTypeConverterFn): void {
        this.ffiRegistry.converter.registerCustomConverter(typeName, converter);
    }

    /**
     * 注销自定义类型转换器
     *
     * @param typeName 类型名称
     */
    public unregisterTypeConverter(typeName: string): void {
        this.ffiRegistry.converter.unregisterCustomConverter(typeName);
    }

    /**
     * 清除所有状态
     */
    public async clear(): Promise<void> {
        if (!this.wasmModule) {
            throw new RustyTypeScriptError("WASM 模块未初始化", "runtime", {
                code: "WASM_NOT_INITIALIZED",
                suggestions: [
                    "请先调用 RustyTypeScript.init() 初始化模块",
                    "检查初始化过程中是否有错误",
                ],
            });
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
            throw new RustyTypeScriptError("WASM 模块未初始化", "runtime", {
                code: "WASM_NOT_INITIALIZED",
                suggestions: [
                    "请先调用 RustyTypeScript.init() 初始化模块",
                    "检查初始化过程中是否有错误",
                ],
            });
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
        this.wasmModule = null;
        if (this.loader) {
            this.loader.reset();
        }
        this._loadingProgress = null;
        this.ffiRegistry.clear();
    }

    /**
     * 获取加载耗时
     *
     * @returns 加载耗时（毫秒），如果未完成则返回 null
     */
    public getLoadDuration(): number | null {
        return this.loader?.getLoadDuration() ?? null;
    }

    /**
     * 检查模块是否已加载
     *
     * @returns 是否已加载
     */
    public isLoaded(): boolean {
        return this.loader?.isLoaded() ?? false;
    }

    /**
     * 检查模块是否正在加载
     *
     * @returns 是否正在加载
     */
    public isLoading(): boolean {
        return this.loader?.isLoading() ?? false;
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
    signature?: FfiSignature,
    description?: string,
): Promise<void> {
    const rustyTs = await RustyTypeScript.init();
    try {
        return await rustyTs.registerFfiFunction(name, func, signature, description);
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

/**
 * FFI 便捷函数
 */

export function createFfiFunction(
    name: string,
    implementation: (...args: unknown[]) => unknown,
    options?: {
        parameters?: FfiParameter[];
        returnType?: FfiType;
        variadic?: boolean;
        description?: string;
        module?: string;
    },
): FfiFunction {
    return {
        name,
        signature: {
            parameters: options?.parameters ?? [],
            returnType: options?.returnType ?? FfiType.Any,
            variadic: options?.variadic ?? false,
        },
        implementation,
        description: options?.description,
        module: options?.module,
    };
}

export function createFfiParameter(
    name: string,
    type: FfiType,
    options?: {
        optional?: boolean;
        defaultValue?: unknown;
        description?: string;
    },
): FfiParameter {
    return {
        name,
        type,
        optional: options?.optional,
        defaultValue: options?.defaultValue,
        description: options?.description,
    };
}
