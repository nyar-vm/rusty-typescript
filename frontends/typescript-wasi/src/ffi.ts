//! FFI 相关类型和类
//! 
//! 定义了 FFI 函数、类型转换器和注册表

import { RustyTypeScriptError } from "./errors";

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
                "Check function name is correct",
                "Ensure function is registered",
                "Check function name case",
            ],
            TypeMismatch: ["Check parameter types", "Verify function signature requirements", "Consider type conversion"],
            InvalidSignature: ["Check function signature is correct", "Verify parameter count and types", "Consult function documentation"],
            CallFailed: ["Check function implementation for errors", "Ensure parameter values are valid", "Check runtime environment"],
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
        return new FfiError(`FFI function not found: ${functionName}`, "FunctionNotFound", {
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
            `FFI function "${functionName}" parameter ${parameterIndex} type mismatch: expected ${expectedType}, actual ${actualType}`,
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
        return new FfiError(`FFI function "${functionName}" signature invalid: ${reason}`, "InvalidSignature", {
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
        return new FfiError(`FFI function "${functionName}" call failed`, "CallFailed", {
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
            fullMessage += `\n  Function: ${this.functionName}`;
        }

        if (this.parameterIndex !== undefined) {
            fullMessage += `\n  Parameter index: ${this.parameterIndex}`;
        }

        if (this.expectedType && this.actualType) {
            fullMessage += `\n  Expected type: ${this.expectedType}`;
            fullMessage += `\n  Actual type: ${this.actualType}`;
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
                // Ignore parse error
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
                // Ignore parse error
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
            console.warn(`FFI function "${ffiFunction.name}" already exists, will be overwritten`);
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
                    `Too many parameters: expected ${parameters.length}, actual ${args.length}`,
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
                    `Missing required parameters: expected at least ${requiredParams.length}, actual ${args.length}`,
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
