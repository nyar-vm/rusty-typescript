//! WASM 加载器
//! 
//! 负责加载 WASM 模块，支持多种加载方式和错误恢复机制

import { WasmLoadError, WasmLoadErrorType } from "./errors";

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
                reject(WasmLoadError.timeoutError(`Load timeout (${ms}ms)`, this.options.url));
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
        try {
            const timeoutMs = this.options.timeout ?? WasmLoader.DEFAULT_TIMEOUT;

            if (this.supportsStreamingCompilation()) {
                return await this.loadWithStreaming(url, timeoutMs);
            }

            return await this.loadWithFetch(url, timeoutMs);
        } catch (error) {
            if (error instanceof WasmLoadError) {
                if (error.canRetry() && error.loadErrorType !== "CompileError") {
                    await this.delay(this.options.retryDelay ?? WasmLoader.DEFAULT_RETRY_DELAY);
                    return this.loadFromUrl(url, retryCount + 1);
                }
                throw error;
            }

            const wasmError = WasmLoadError.networkError(
                `Failed to load WASM module: ${error instanceof Error ? error.message : String(error)}`,
                url,
                error instanceof Error ? error : new Error(String(error)),
            );

            if (retryCount < (this.options.maxRetries ?? WasmLoader.DEFAULT_MAX_RETRIES)) {
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
        const fetchPromise = fetch(url, {
            headers: this.options.headers,
        });

        const response = await Promise.race([
            fetchPromise,
            this.createTimeoutPromise<Response>(timeoutMs),
        ]);

        if (!response.ok) {
            throw WasmLoadError.networkError(
                `HTTP error: ${response.status} ${response.statusText}`,
                url,
            );
        }

        const modulePromise = WebAssembly.compileStreaming(response);

        const module = await Promise.race([
            modulePromise,
            this.createTimeoutPromise<WebAssembly.Module>(timeoutMs),
        ]);

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
        const fetchPromise = fetch(url, {
            headers: this.options.headers,
        });

        const response = await Promise.race([
            fetchPromise,
            this.createTimeoutPromise<Response>(timeoutMs),
        ]);

        if (!response.ok) {
            throw WasmLoadError.networkError(
                `HTTP error: ${response.status} ${response.statusText}`,
                url,
            );
        }

        const reader = response.body?.getReader();
        if (!reader) {
            const arrayBuffer = await response.arrayBuffer();
            const module = await WebAssembly.compile(arrayBuffer);
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
        }

        const arrayBuffer = new Uint8Array(loadedBytes);
        let offset = 0;
        for (const chunk of chunks) {
            arrayBuffer.set(chunk, offset);
            offset += chunk.length;
        }

        try {
            const module = await WebAssembly.compile(arrayBuffer.buffer);
            return module;
        } catch (error) {
            throw WasmLoadError.compileError(
                `Failed to compile WASM module: ${error instanceof Error ? error.message : String(error)}`,
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
        const arrayBuffer = buffer instanceof Buffer ? new Uint8Array(buffer).buffer : buffer;

        try {
            const module = await WebAssembly.compile(arrayBuffer as BufferSource);
            return module;
        } catch (error) {
            throw WasmLoadError.compileError(
                `Failed to compile WASM module: ${error instanceof Error ? error.message : String(error)}`,
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
        try {
            const instance = await WebAssembly.instantiate(module, imports);
            return instance;
        } catch (error) {
            throw WasmLoadError.instantiateError(
                `Failed to instantiate WASM module: ${error instanceof Error ? error.message : String(error)}`,
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

        try {
            let module: WebAssembly.Module;

            if (this.options.buffer) {
                module = await this.loadFromBuffer(this.options.buffer);
            } else if (this.options.url) {
                module = await this.loadFromUrl(this.options.url);
            } else {
                throw new WasmLoadError("No WASM module source specified (URL or Buffer)", "NetworkError", {
                    suggestions: ["Please provide wasmUrl or wasmModule option"],
                });
            }

            this._module = module;

            const instance = await this.instantiateModule(module, this.options.imports);
            this._instance = instance;

            return instance;
        } catch (error) {
            throw error;
        }
    }

    /**
     * 重置加载器状态
     */
    public reset(): void {
        this._module = null;
        this._instance = null;
    }
}
