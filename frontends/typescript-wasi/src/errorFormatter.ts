//! 错误格式化工具
//!
//! 提供错误消息的格式化和用户友好的错误展示功能

import { RustyTypeScriptError, ErrorType, ErrorLocation } from "./errors";

/**
 * 错误格式化选项
 */
export interface FormatOptions {
    /**
     * 是否包含颜色代码（ANSI）
     */
    colorize?: boolean;

    /**
     * 是否包含堆栈信息
     */
    includeStack?: boolean;

    /**
     * 是否包含上下文信息
     */
    includeContext?: boolean;

    /**
     * 是否包含修复建议
     */
    includeSuggestions?: boolean;

    /**
     * 代码上下文的行数
     */
    contextLines?: number;

    /**
     * 最大消息长度
     */
    maxMessageLength?: number;
}

/**
 * 默认格式化选项
 */
const DEFAULT_OPTIONS: FormatOptions = {
    colorize: false,
    includeStack: true,
    includeContext: true,
    includeSuggestions: true,
    contextLines: 3,
    maxMessageLength: 1000,
};

/**
 * ANSI 颜色代码
 */
const ANSI_COLORS = {
    reset: "\x1b[0m",
    red: "\x1b[31m",
    yellow: "\x1b[33m",
    blue: "\x1b[34m",
    cyan: "\x1b[36m",
    gray: "\x1b[90m",
    bold: "\x1b[1m",
};

/**
 * 错误类型对应的颜色
 */
const ERROR_TYPE_COLORS: Record<ErrorType, string> = {
    syntax: ANSI_COLORS.red,
    type: ANSI_COLORS.yellow,
    runtime: ANSI_COLORS.red,
    memory: ANSI_COLORS.cyan,
    module: ANSI_COLORS.blue,
    ffi: ANSI_COLORS.yellow,
};

/**
 * Error type names in English
 */
const ERROR_TYPE_NAMES: Record<ErrorType, string> = {
    syntax: "Syntax Error",
    type: "Type Error",
    runtime: "Runtime Error",
    memory: "Memory Error",
    module: "Module Error",
    ffi: "FFI Error",
};

/**
 * 格式化错误消息
 *
 * 将 RustyTypeScriptError 转换为用户友好的错误消息字符串
 *
 * @param error 要格式化的错误对象
 * @param options 格式化选项
 * @returns 格式化后的错误消息字符串
 */
export function formatErrorMessage(
    error: RustyTypeScriptError,
    options: FormatOptions = {},
): string {
    const opts = { ...DEFAULT_OPTIONS, ...options };
    const lines: string[] = [];

    const typeColor = opts.colorize ? ERROR_TYPE_COLORS[error.type] : "";
    const reset = opts.colorize ? ANSI_COLORS.reset : "";
    const bold = opts.colorize ? ANSI_COLORS.bold : "";

    const typeName = ERROR_TYPE_NAMES[error.type];

    if (error.code) {
        lines.push(`${bold}${typeColor}[${error.code}] ${typeName}${reset}`);
    } else {
        lines.push(`${bold}${typeColor}${typeName}${reset}`);
    }

    let message = error.message;
    if (opts.maxMessageLength && message.length > opts.maxMessageLength) {
        message = message.substring(0, opts.maxMessageLength) + "...";
    }
    lines.push(`  Message: ${message}`);

    if (error.location) {
        lines.push(formatLocation(error.location, opts));
    }

    if (opts.includeContext && error.context) {
        const contextStr = formatContext(error.context, opts);
        if (contextStr) {
            lines.push(contextStr);
        }
    }

    if (opts.includeSuggestions && error.suggestions.length > 0) {
        lines.push(formatSuggestions(error.suggestions, opts));
    }

    if (opts.includeStack && error.errorStack) {
        lines.push("");
        lines.push(formatErrorStack(error, opts));
    }

    return lines.join("\n");
}

/**
 * 格式化错误位置
 *
 * @param location 错误位置
 * @param options 格式化选项
 * @returns 格式化后的位置字符串
 */
export function formatLocation(location: ErrorLocation, options: FormatOptions = {}): string {
    const opts = { ...DEFAULT_OPTIONS, ...options };
    const cyan = opts.colorize ? ANSI_COLORS.cyan : "";
    const reset = opts.colorize ? ANSI_COLORS.reset : "";

    let locationStr = "  Location: ";

    if (location.file) {
        locationStr += `${cyan}${location.file}:${location.line}:${location.column}${reset}`;
    } else {
        locationStr += `Line ${location.line}, Column ${location.column}`;
    }

    if (location.snippet) {
        locationStr += `\n  Code: ${location.snippet}`;
    }

    return locationStr;
}

/**
 * 格式化错误堆栈
 *
 * @param error 错误对象
 * @param options 格式化选项
 * @returns 格式化后的堆栈字符串
 */
export function formatErrorStack(error: RustyTypeScriptError, options: FormatOptions = {}): string {
    const opts = { ...DEFAULT_OPTIONS, ...options };
    const gray = opts.colorize ? ANSI_COLORS.gray : "";
    const reset = opts.colorize ? ANSI_COLORS.reset : "";

    if (!error.errorStack) {
        return "";
    }

    const lines = error.errorStack.split("\n");
    const formattedLines: string[] = [];

    formattedLines.push("  Stack trace:");

    for (const line of lines) {
        const trimmed = line.trim();
        if (trimmed) {
            formattedLines.push(`    ${gray}${trimmed}${reset}`);
        }
    }

    return formattedLines.join("\n");
}

/**
 * 格式化错误上下文
 *
 * @param context 错误上下文
 * @param options 格式化选项
 * @returns 格式化后的上下文字符串
 */
function formatContext(
    context: NonNullable<RustyTypeScriptError["context"]>,
    options: FormatOptions,
): string {
    const opts = { ...DEFAULT_OPTIONS, ...options };
    const lines: string[] = [];
    const blue = opts.colorize ? ANSI_COLORS.blue : "";
    const reset = opts.colorize ? ANSI_COLORS.reset : "";

    if (context.code) {
        lines.push(`  Code snippet:`);
        const codeLines = context.code.split("\n");
        for (let i = 0; i < Math.min(codeLines.length, opts.contextLines ?? 3); i++) {
            lines.push(`    ${codeLines[i]}`);
        }
    }

    if (context.variables && Object.keys(context.variables).length > 0) {
        lines.push(`  Variable state:`);
        for (const [name, value] of Object.entries(context.variables)) {
            const valueStr = typeof value === "object" ? JSON.stringify(value) : String(value);
            lines.push(`    ${blue}${name}${reset} = ${valueStr}`);
        }
    }

    if (context.callStack && context.callStack.length > 0) {
        lines.push(`  Call stack:`);
        for (let i = 0; i < context.callStack.length; i++) {
            lines.push(`    ${i + 1}. ${context.callStack[i]}`);
        }
    }

    if (context.metadata && Object.keys(context.metadata).length > 0) {
        lines.push(`  Metadata:`);
        for (const [key, value] of Object.entries(context.metadata)) {
            lines.push(`    ${key}: ${JSON.stringify(value)}`);
        }
    }

    return lines.join("\n");
}

/**
 * 格式化修复建议
 *
 * @param suggestions 修复建议列表
 * @param options 格式化选项
 * @returns 格式化后的建议字符串
 */
function formatSuggestions(suggestions: string[], options: FormatOptions): string {
    const opts = { ...DEFAULT_OPTIONS, ...options };
    const green = opts.colorize ? "\x1b[32m" : "";
    const reset = opts.colorize ? ANSI_COLORS.reset : "";

    const lines: string[] = [];
    lines.push(`  ${green}Suggestions:${reset}`);

    for (let i = 0; i < suggestions.length; i++) {
        lines.push(`    ${i + 1}. ${suggestions[i]}`);
    }

    return lines.join("\n");
}

/**
 * 创建简洁的错误摘要
 *
 * @param error 错误对象
 * @returns 简洁的错误摘要字符串
 */
export function createErrorSummary(error: RustyTypeScriptError): string {
    const typeName = ERROR_TYPE_NAMES[error.type];
    let summary = `[${typeName}] ${error.message}`;

    if (error.location) {
        const loc = error.location.file
            ? `${error.location.file}:${error.location.line}:${error.location.column}`
            : `行 ${error.location.line}`;
        summary += ` (${loc})`;
    }

    return summary;
}

/**
 * 创建详细的错误报告
 *
 * @param error 错误对象
 * @param sourceCode 源代码（可选）
 * @returns 详细的错误报告字符串
 */
export function createDetailedReport(error: RustyTypeScriptError, sourceCode?: string): string {
    const sections: string[] = [];

    sections.push("=".repeat(60));
    sections.push("错误报告");
    sections.push("=".repeat(60));
    sections.push("");

    sections.push(formatErrorMessage(error, { colorize: false }));

    if (sourceCode && error.location) {
        sections.push("");
        sections.push("-".repeat(60));
        sections.push("源代码上下文");
        sections.push("-".repeat(60));
        sections.push(formatSourceContext(sourceCode, error.location));
    }

    sections.push("");
    sections.push("=".repeat(60));

    return sections.join("\n");
}

/**
 * 格式化源代码上下文
 *
 * @param sourceCode 源代码
 * @param location 错误位置
 * @param contextLines 上下文行数
 * @returns 格式化后的源代码字符串
 */
function formatSourceContext(
    sourceCode: string,
    location: ErrorLocation,
    contextLines: number = 3,
): string {
    const lines = sourceCode.split("\n");
    const startLine = Math.max(0, location.line - contextLines - 1);
    const endLine = Math.min(lines.length, location.line + contextLines);

    const result: string[] = [];
    const lineNumWidth = String(endLine).length;

    for (let i = startLine; i < endLine; i++) {
        const lineNum = String(i + 1).padStart(lineNumWidth, " ");
        const lineContent = lines[i];

        if (i === location.line - 1) {
            result.push(`>>> ${lineNum} | ${lineContent}`);
            const pointer = " ".repeat(location.column + 6 + lineNumWidth);
            result.push(`${pointer}^`);
        } else {
            result.push(`    ${lineNum} | ${lineContent}`);
        }
    }

    return result.join("\n");
}

/**
 * 将错误转换为 JSON 格式
 *
 * @param error 错误对象
 * @returns JSON 格式的错误信息
 */
export function errorToJson(error: RustyTypeScriptError): Record<string, unknown> {
    return {
        name: error.name,
        type: error.type,
        message: error.message,
        code: error.code,
        location: error.location,
        suggestions: error.suggestions,
        context: error.context,
        stack: error.errorStack,
    };
}

/**
 * 从 JSON 创建错误对象
 *
 * @param json JSON 对象
 * @returns RustyTypeScriptError 实例
 */
export function errorFromJson(json: Record<string, unknown>): RustyTypeScriptError {
    return new RustyTypeScriptError(String(json.message ?? ""), json.type as ErrorType, {
        code: json.code ? String(json.code) : undefined,
        location: json.location as ErrorLocation | undefined,
        suggestions: Array.isArray(json.suggestions) ? (json.suggestions as string[]) : [],
        context: json.context as RustyTypeScriptError["context"],
        stack: json.stack ? String(json.stack) : undefined,
    });
}

/**
 * 批量格式化错误列表
 *
 * @param errors 错误列表
 * @param options 格式化选项
 * @returns 格式化后的错误列表字符串
 */
export function formatErrorList(
    errors: RustyTypeScriptError[],
    options: FormatOptions = {},
): string {
    if (errors.length === 0) {
        return "没有错误";
    }

    const opts = { ...DEFAULT_OPTIONS, ...options };
    const lines: string[] = [];

    lines.push(`发现 ${errors.length} 个错误:\n`);

    for (let i = 0; i < errors.length; i++) {
        lines.push(`错误 ${i + 1}:`);
        lines.push(formatErrorMessage(errors[i], { ...opts, includeStack: false }));
        lines.push("");
    }

    return lines.join("\n");
}
