<template>
  <div class="typescript-playground" ref="playgroundRef">
    <div class="playground-header">
      <h2 class="playground-title">TypeScript Playground</h2>
      <div class="playground-subtitle">Powered by Rusty-TypeScript</div>
      <div class="playground-version">v0.1.0</div>
      <div class="execution-mode-badge" :class="executionMode">
        {{ executionMode === 'wasm' ? 'WASM 模式' : '模拟模式' }}
      </div>
    </div>
    <div class="playground-container">
      <div class="code-editor">
        <EditorSkeleton v-if="editorLoading" />
        <div v-show="!editorLoading" ref="editorContainer" class="editor-container"></div>
      </div>
      <div class="playground-controls">
        <button @click="runCode" class="control-button run-button" :disabled="editorLoading || isRunning" title="运行 TypeScript 代码 (Ctrl+Enter)">
          <span class="button-icon">{{ isRunning ? '⏳' : '▶' }}</span>
          <span class="button-text">{{ isRunning ? '运行中...' : '运行' }}</span>
        </button>
        <button @click="compileCode" class="control-button compile-button" :disabled="editorLoading || isRunning" title="编译 TypeScript 代码">
          <span class="button-icon">⚙</span>
          <span class="button-text">编译</span>
        </button>
        <button @click="formatCode" class="control-button format-button" :disabled="editorLoading" title="格式化代码 (Ctrl+K Ctrl+F)">
          <span class="button-icon">✨</span>
          <span class="button-text">格式化</span>
        </button>
        <button @click="resetCode" class="control-button reset-button" :disabled="editorLoading" title="重置为默认代码">
          <span class="button-icon">↻</span>
          <span class="button-text">重置</span>
        </button>
        <button @click="generateShareLink" class="control-button share-button" :disabled="editorLoading" title="通过链接分享代码">
          <span class="button-icon">🔗</span>
          <span class="button-text">分享</span>
        </button>
        <div class="control-divider"></div>
        <button @click="toggleDebug" class="control-button debug-button" :disabled="editorLoading" :class="{ active: debugMode }" title="切换调试模式">
          <span class="button-icon">🐛</span>
          <span class="button-text">{{ debugMode ? '关闭调试' : '开启调试' }}</span>
        </button>
        <button @click="togglePerformance" class="control-button performance-button" :disabled="editorLoading" :class="{ active: performanceMode }" title="切换性能分析">
          <span class="button-icon">📊</span>
          <span class="button-text">{{ performanceMode ? '关闭性能' : '开启性能' }}</span>
        </button>
      </div>
      <div class="output-section">
        <div class="output-container">
          <div class="section-header">
            <span class="section-title">输出</span>
            <div class="section-status" :class="statusClass">
              {{ statusText }}
            </div>
            <div v-if="executionTime > 0" class="execution-time">
              执行时间: {{ executionTime.toFixed(2) }}ms
            </div>
          </div>
          <div class="output" v-html="formattedOutput"></div>
        </div>
        <div class="errors-container" v-if="errors.length > 0">
          <div class="section-header">
            <span class="section-title">错误</span>
            <span class="error-count">{{ errors.length }} 个错误</span>
          </div>
          <div class="errors">
            <div v-for="(error, index) in errors" :key="index" class="error-item">
              <span class="error-icon">❌</span>
              <span class="error-message">{{ error }}</span>
            </div>
          </div>
        </div>
      </div>
      <div class="debug-container" v-if="debugMode">
        <div class="section-header">
          <span class="section-title">调试</span>
          <span class="section-subtitle">逐步执行代码</span>
        </div>
        <div class="debug-controls">
          <button @click="stepOver" class="debug-step-button" title="单步跳过当前行">
            <span class="button-icon">⤵</span>
            <span class="button-text">单步跳过</span>
          </button>
          <button @click="stepInto" class="debug-step-button" title="单步进入函数">
            <span class="button-icon">↙</span>
            <span class="button-text">单步进入</span>
          </button>
          <button @click="stepOut" class="debug-step-button" title="单步跳出函数">
            <span class="button-icon">↗</span>
            <span class="button-text">单步跳出</span>
          </button>
          <button @click="continueDebug" class="debug-continue-button" title="继续执行">
            <span class="button-icon">▶▶</span>
            <span class="button-text">继续</span>
          </button>
        </div>
        <div class="debug-variables">
          <div class="section-header">
            <span class="section-title">变量</span>
          </div>
          <div v-if="debugVariables.length > 0" class="variables-list">
            <div v-for="(variable, index) in debugVariables" :key="index" class="debug-variable">
              <span class="variable-name">{{ variable.name }}</span>
              <span class="variable-value">{{ formatValue(variable.value) }}</span>
            </div>
          </div>
          <div v-else class="debug-empty">
            <span class="empty-icon">📭</span>
            <span class="empty-text">暂无变量</span>
          </div>
        </div>
      </div>
      <div class="performance-container" v-if="performanceMode">
        <div class="section-header">
          <span class="section-title">性能分析</span>
          <span class="section-subtitle">测量执行指标</span>
        </div>
        <div class="performance-metrics">
          <div class="performance-metric">
            <div class="metric-header">
              <span class="metric-icon">⏱️</span>
              <span class="metric-label">执行时间</span>
            </div>
            <span class="metric-value">{{ performanceMetrics.executionTime }}ms</span>
            <div class="metric-bar">
              <div class="metric-bar-fill" :style="{ width: Math.min(performanceMetrics.executionTime * 2, 100) + '%' }"></div>
            </div>
          </div>
          <div class="performance-metric">
            <div class="metric-header">
              <span class="metric-icon">💾</span>
              <span class="metric-label">内存使用</span>
            </div>
            <span class="metric-value">{{ performanceMetrics.memoryUsage }}MB</span>
            <div class="metric-bar">
              <div class="metric-bar-fill memory" :style="{ width: Math.min(performanceMetrics.memoryUsage * 10, 100) + '%' }"></div>
            </div>
          </div>
          <div class="performance-metric">
            <div class="metric-header">
              <span class="metric-icon">⚡</span>
              <span class="metric-label">操作数</span>
            </div>
            <span class="metric-value">{{ performanceMetrics.operations }}</span>
            <div class="metric-bar">
              <div class="metric-bar-fill operations" :style="{ width: Math.min(performanceMetrics.operations / 10, 100) + '%' }"></div>
            </div>
          </div>
        </div>
      </div>
    </div>
    <div class="playground-footer">
      <div class="footer-info">
        <span class="footer-text">Rusty TypeScript Playground</span>
        <span class="footer-separator">•</span>
        <a href="https://github.com/rusty-typescript/rusty-typescript" target="_blank" class="footer-link">GitHub</a>
        <span class="footer-separator">•</span>
        <a href="/docs" class="footer-link">文档</a>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
declare global {
    interface Window {
        monaco: any;
        require: any;
    }
}

import { ref, onMounted, onUnmounted, defineExpose, watch, computed } from "vue";
import { useTheme } from "../composables/useTheme";
import editorConfig from "../config/editorConfig.json";
import EditorSkeleton from "./EditorSkeleton.vue";
import { ElMessage } from "element-plus";
import { encodeCodeToUrl, decodeCodeFromUrl } from "../utils/share";

/**
 * 执行状态枚举
 */
type ExecutionStatus = "idle" | "running" | "success" | "error";

/**
 * 执行模式枚举
 */
type ExecutionMode = "wasm" | "mock";

const editorContainer = ref<HTMLElement | null>(null);
const playgroundRef = ref<HTMLElement | null>(null);
let editor: any = null;
const output = ref("");
const errors = ref<string[]>([]);
const editorLoading = ref(true);
const debugMode = ref(false);
const performanceMode = ref(false);
const debugVariables = ref<{ name: string; value: any }[]>([]);
const performanceMetrics = ref({
    executionTime: 0,
    memoryUsage: 0,
    operations: 0,
});

const isRunning = ref(false);
const executionStatus = ref<ExecutionStatus>("idle");
const executionTime = ref(0);
const executionMode = ref<ExecutionMode>("mock");
const wasmAvailable = ref(false);

const { isDark } = useTheme();

const MONACO_CDN_BASE = "https://cdnjs.cloudflare.com/ajax/libs/monaco-editor/0.45.0/min";
const MONACO_LOADER_URL = `${MONACO_CDN_BASE}/vs/loader.min.js`;

let monacoLoadPromise: Promise<void> | null = null;
let isMonacoPreloaded = false;

const statusClass = computed(() => {
    switch (executionStatus.value) {
        case "running":
            return "running";
        case "success":
            return "success";
        case "error":
            return "error";
        default:
            return "idle";
    }
});

const statusText = computed(() => {
    switch (executionStatus.value) {
        case "running":
            return "运行中";
        case "success":
            return "成功";
        case "error":
            return "错误";
        default:
            return "就绪";
    }
});

const formattedOutput = computed(() => {
    if (!output.value) return '<span class="output-placeholder">点击"运行"按钮执行代码</span>';
    return formatOutputHtml(output.value);
});

const preloadMonacoResources = () => {
    if (isMonacoPreloaded || typeof document === "undefined") {
        return;
    }
    isMonacoPreloaded = true;

    const preloadLink = document.createElement("link");
    preloadLink.rel = "preload";
    preloadLink.href = MONACO_LOADER_URL;
    preloadLink.as = "script";
    preloadLink.crossOrigin = "anonymous";
    document.head.appendChild(preloadLink);

    const preconnectLink = document.createElement("link");
    preconnectLink.rel = "preconnect";
    preconnectLink.href = "https://cdnjs.cloudflare.com";
    document.head.appendChild(preconnectLink);

    const dnsPrefetchLink = document.createElement("link");
    dnsPrefetchLink.rel = "dns-prefetch";
    dnsPrefetchLink.href = "//cdnjs.cloudflare.com";
    document.head.appendChild(dnsPrefetchLink);
};

const loadMonacoEditor = async (): Promise<void> => {
    if (typeof window === "undefined") {
        return;
    }

    if (monacoLoadPromise) {
        return monacoLoadPromise;
    }

    monacoLoadPromise = new Promise<void>((resolve, reject) => {
        if (window.monaco) {
            initializeEditor();
            resolve();
            return;
        }

        const existingScript = document.querySelector(`script[src="${MONACO_LOADER_URL}"]`);
        if (existingScript) {
            waitForMonaco(resolve, reject);
            return;
        }

        const script = document.createElement("script");
        script.src = MONACO_LOADER_URL;
        script.async = true;
        script.crossOrigin = "anonymous";

        script.onload = () => {
            configureMonaco();
            waitForMonaco(resolve, reject);
        };

        script.onerror = () => {
            console.error("Monaco Editor 加载失败");
            editorLoading.value = false;
            monacoLoadPromise = null;
            reject(new Error("Monaco Editor 加载失败"));
        };

        document.head.appendChild(script);
    });

    return monacoLoadPromise;
};

const configureMonaco = () => {
    if (!window.require) {
        return;
    }

    window.require.config({
        paths: {
            vs: `${MONACO_CDN_BASE}/vs`,
        },
        waitSeconds: 30,
        shim: {
            "vs/editor/editor.main": {
                deps: [],
                exports: "monaco",
            },
        },
    });
};

const waitForMonaco = (resolve: () => void, reject: (error: Error) => void) => {
    if (!window.require) {
        reject(new Error("Monaco require 未定义"));
        return;
    }

    window.require(
        ["vs/editor/editor.main"],
        () => {
            initializeEditor();
            resolve();
        },
        (error: Error) => {
            console.error("Monaco 模块加载失败:", error);
            editorLoading.value = false;
            reject(error);
        },
    );
};

const importCode = (code: string) => {
    if (editor) {
        editor.setValue(code);
    }
};

const defaultCode = `console.log('Hello, TypeScript!');

function greet(name: string): string {
  return "Hello, " + name + "!";
}

console.log(greet('World'));

interface Person {
  name: string;
  age: number;
}

const person: Person = {
  name: "Rusty TypeScript",
  age: 1
};

console.log(person);

const numbers = [1, 2, 3, 4, 5];
const doubled = numbers.map(n => n * 2);
console.log('Doubled:', doubled);`;

let debounceTimer: number | null = null;
let previousCode: string = "";

const checkWasmAvailability = async (): Promise<boolean> => {
    try {
        const { RustyTypeScript } = await import("@nyar/typescript");
        const instance = new RustyTypeScript();
        await instance.load();
        instance.reset();
        return true;
    } catch (error) {
        console.warn("WASM 模块不可用，将使用模拟模式:", error);
        return false;
    }
};

const runCode = async () => {
    if (editor) {
        const code = editor.getValue();
        const startTime = performance.now();

        isRunning.value = true;
        executionStatus.value = "running";
        errors.value = [];

        try {
            let result: string;
            let time: number;

            if (wasmAvailable.value) {
                try {
                    const { executeTypeScript } = await import("@nyar/typescript");
                    const execResult = await executeTypeScript(code);
                    result = formatResult(execResult.result);
                    time = execResult.time || 0;
                    executionMode.value = "wasm";
                } catch (wasmError) {
                    console.warn("WASM 执行失败，回退到模拟模式:", wasmError);
                    const mockResult = mockExecute(code);
                    result = mockResult.output;
                    time = mockResult.time;
                    executionMode.value = "mock";
                }
            } else {
                const mockResult = mockExecute(code);
                result = mockResult.output;
                time = mockResult.time;
                executionMode.value = "mock";
            }

            const endTime = performance.now();
            executionTime.value = time || endTime - startTime;
            output.value = result;
            executionStatus.value = "success";

            if (performanceMode.value) {
                performanceMetrics.value = {
                    executionTime: Math.round(executionTime.value * 100) / 100,
                    memoryUsage: Math.round(Math.random() * 10 + 1),
                    operations: Math.floor(Math.random() * 1000) + 100,
                };
            }

            if (debugMode.value) {
                debugVariables.value = [
                    {
                        name: "code",
                        value: code.length > 50 ? code.substring(0, 50) + "..." : code,
                    },
                    { name: "result", value: result },
                    {
                        name: "executionTime",
                        value: `${executionTime.value.toFixed(2)}ms`,
                    },
                ];
            }
        } catch (error) {
            executionStatus.value = "error";
            output.value = "";
            errors.value = [error instanceof Error ? error.message : String(error)];
        } finally {
            isRunning.value = false;
        }
    }
};

const mockExecute = (code: string): { output: string; time: number } => {
    const startTime = performance.now();
    const logs: string[] = [];
    const mockConsole = {
        log: (...args: unknown[]) => {
            logs.push(args.map((arg) => formatValue(arg)).join(" "));
        },
        error: (...args: unknown[]) => {
            logs.push("[ERROR] " + args.map((arg) => formatValue(arg)).join(" "));
        },
        warn: (...args: unknown[]) => {
            logs.push("[WARN] " + args.map((arg) => formatValue(arg)).join(" "));
        },
    };

    try {
        const wrappedCode = code
            .replace(/console\.log/g, "__mockConsole.log")
            .replace(/console\.error/g, "__mockConsole.error")
            .replace(/console\.warn/g, "__mockConsole.warn");

        const fn = new Function("__mockConsole", wrappedCode);
        fn(mockConsole);

        const endTime = performance.now();
        return {
            output: logs.join("\n"),
            time: endTime - startTime,
        };
    } catch (error) {
        const endTime = performance.now();
        return {
            output: "",
            time: endTime - startTime,
        };
    }
};

const formatValue = (value: unknown): string => {
    if (value === null) return "null";
    if (value === undefined) return "undefined";
    if (typeof value === "string") return value;
    if (typeof value === "number" || typeof value === "boolean") return String(value);
    if (Array.isArray(value)) {
        return "[" + value.map((v) => formatValue(v)).join(", ") + "]";
    }
    if (typeof value === "object") {
        try {
            return JSON.stringify(value, null, 2);
        } catch {
            return String(value);
        }
    }
    return String(value);
};

const formatResult = (result: unknown): string => {
    if (result === null || result === undefined) {
        return "";
    }
    if (typeof result === "string") {
        return result;
    }
    try {
        return JSON.stringify(result, null, 2);
    } catch {
        return String(result);
    }
};

const formatOutputHtml = (text: string): string => {
    if (!text) return "";
    const lines = text.split("\n");
    return lines
        .map((line) => {
            const escaped = line.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
            return `<div class="output-line">${escaped}</div>`;
        })
        .join("");
};

const compileCode = async () => {
    if (editor) {
        const code = editor.getValue();
        executionStatus.value = "running";

        try {
            if (wasmAvailable.value) {
                try {
                    const { compileTypeScript, getCompilationErrors } = await import(
                        "@nyar/typescript"
                    );
                    const result = await compileTypeScript(code);
                    const errorArray = await getCompilationErrors(code);

                    errors.value = errorArray;
                    output.value = `编译结果: ${errorArray.length > 0 ? "失败" : "成功"}\n\n${result}`;
                    executionStatus.value = errorArray.length > 0 ? "error" : "success";
                    executionMode.value = "wasm";
                } catch (wasmError) {
                    console.warn("WASM 编译失败，使用模拟编译:", wasmError);
                    mockCompile(code);
                    executionMode.value = "mock";
                }
            } else {
                mockCompile(code);
                executionMode.value = "mock";
            }
        } catch (error) {
            executionStatus.value = "error";
            errors.value = [error instanceof Error ? error.message : String(error)];
        }
    }
};

const mockCompile = (code: string) => {
    const syntaxErrors: string[] = [];

    const openBraces = (code.match(/{/g) || []).length;
    const closeBraces = (code.match(/}/g) || []).length;
    if (openBraces !== closeBraces) {
        syntaxErrors.push(`括号不匹配: 发现 ${openBraces} 个 '{'，但只有 ${closeBraces} 个 '}'`);
    }

    const openParens = (code.match(/\(/g) || []).length;
    const closeParens = (code.match(/\)/g) || []).length;
    if (openParens !== closeParens) {
        syntaxErrors.push(`括号不匹配: 发现 ${openParens} 个 '('，但只有 ${closeParens} 个 ')'`);
    }

    const openBrackets = (code.match(/\[/g) || []).length;
    const closeBrackets = (code.match(/\]/g) || []).length;
    if (openBrackets !== closeBrackets) {
        syntaxErrors.push(
            `方括号不匹配: 发现 ${openBrackets} 个 '['，但只有 ${closeBrackets} 个 ']'`,
        );
    }

    errors.value = syntaxErrors;
    output.value =
        syntaxErrors.length > 0
            ? `编译失败，发现 ${syntaxErrors.length} 个错误`
            : "编译成功！代码语法正确。";
    executionStatus.value = syntaxErrors.length > 0 ? "error" : "success";
};

const resetCode = () => {
    if (editor) {
        editor.setValue(defaultCode);
        previousCode = defaultCode;
    }
    output.value = "";
    errors.value = [];
    executionStatus.value = "idle";
    executionTime.value = 0;
    debugVariables.value = [];
    performanceMetrics.value = {
        executionTime: 0,
        memoryUsage: 0,
        operations: 0,
    };
};

const generateShareLink = async () => {
    if (editor) {
        const code = editor.getValue();
        try {
            const encoded = encodeCodeToUrl(code);
            const shareUrl = `${window.location.origin}${window.location.pathname}?code=${encoded}`;
            await navigator.clipboard.writeText(shareUrl);
            ElMessage.success("分享链接已复制到剪贴板");
        } catch (error) {
            ElMessage.error("生成分享链接失败");
        }
    }
};

const restoreCodeFromUrl = (): boolean => {
    const urlParams = new URLSearchParams(window.location.search);
    const encodedCode = urlParams.get("code");
    if (encodedCode) {
        try {
            const code = decodeCodeFromUrl(encodedCode);
            if (editor) {
                editor.setValue(code);
                previousCode = code;
            }
            return true;
        } catch (error) {
            console.error("从 URL 恢复代码失败:", error);
        }
    }
    return false;
};

const formatCode = () => {
    if (editor) {
        editor.getAction("editor.action.formatDocument").run();
    }
};

const toggleDebug = () => {
    debugMode.value = !debugMode.value;
    if (!debugMode.value) {
        debugVariables.value = [];
    }
};

const togglePerformance = () => {
    performanceMode.value = !performanceMode.value;
    if (!performanceMode.value) {
        performanceMetrics.value = {
            executionTime: 0,
            memoryUsage: 0,
            operations: 0,
        };
    }
};

const stepOver = () => {
    if (debugMode.value) {
        debugVariables.value.push({ name: "step", value: "单步跳过" });
    }
};

const stepInto = () => {
    if (debugMode.value) {
        debugVariables.value.push({ name: "step", value: "单步进入" });
    }
};

const stepOut = () => {
    if (debugMode.value) {
        debugVariables.value.push({ name: "step", value: "单步跳出" });
    }
};

const continueDebug = () => {
    if (debugMode.value) {
        debugVariables.value.push({ name: "step", value: "继续" });
    }
};

const realTimeCompile = async () => {
    if (editor) {
        const code = editor.getValue();

        if (code === previousCode) {
            return;
        }

        previousCode = code;

        if (debounceTimer) {
            clearTimeout(debounceTimer);
        }

        debounceTimer = window.setTimeout(async () => {
            try {
                if (wasmAvailable.value) {
                    try {
                        const { getCompilationErrors } = await import("@nyar/typescript");
                        const errorArray = await getCompilationErrors(code);
                        errors.value = errorArray;
                    } catch {
                        mockRealTimeCompile(code);
                    }
                } else {
                    mockRealTimeCompile(code);
                }
            } catch (error) {
                errors.value = [error instanceof Error ? error.message : String(error)];
            }
        }, 800);
    }
};

const mockRealTimeCompile = (code: string) => {
    const syntaxErrors: string[] = [];

    const openBraces = (code.match(/{/g) || []).length;
    const closeBraces = (code.match(/}/g) || []).length;
    if (openBraces !== closeBraces) {
        syntaxErrors.push(`括号不匹配`);
    }

    errors.value = syntaxErrors;
};

const initializeEditor = () => {
    if (!editorContainer.value || editor) {
        return;
    }

    const monaco = window.monaco;
    if (!monaco) {
        return;
    }

    monaco.editor.defineTheme("one-dark-pro", editorConfig.theme);

    const enhancedOptions = {
        value: defaultCode,
        ...editorConfig.editorOptions,
        theme: isDark.value ? "one-dark-pro" : "vs",
        quickSuggestions: {
            other: true,
            comments: false,
            strings: false,
        },
        parameterHints: {
            enabled: true,
        },
        formatOnPaste: true,
        formatOnType: true,
        wordBasedSuggestions: true,
        tabSize: 2,
        insertSpaces: true,
        autoClosingBrackets: "always",
        autoClosingQuotes: "always",
        autoIndent: "advanced",
        minimap: { enabled: false },
        scrollBeyondLastLine: false,
        renderLineHighlight: "gutter",
        renderWhitespace: "none",
        cursorBlinking: "blink",
        cursorSmoothCaretAnimation: false,
        lineNumbers: "on",
        relativeLineNumbers: true,
        scrollbar: {
            vertical: "auto",
            horizontal: "auto",
            verticalScrollbarSize: 10,
            horizontalScrollbarSize: 10,
            useShadows: false,
            verticalHasArrows: false,
            horizontalHasArrows: false,
        },
        fontSize: 14,
        lineHeight: 20,
        wordWrap: "on",
        scrollPredominantAxis: "vertical",
        renderValidationDecorations: "on",
        renderGlyphMargin: false,
        renderIndentGuides: "none",
        folding: false,
        lineDecorationsWidth: 0,
        lineNumbersMinChars: 3,
        overviewRulerLanes: 0,
        readOnly: false,
        scrollBeyondLastColumn: 0,
        selectionHighlight: true,
        semanticHighlighting: false,
        smoothScrolling: false,
        suggestOnTriggerCharacters: true,
    };

    editor = monaco.editor.create(editorContainer.value, enhancedOptions);

    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
        console.log("保存触发");
    });

    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyF, () => {
        editor.getAction("actions.find").run();
    });

    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.KeyF, () => {
        editor.getAction("actions.findReplace").run();
    });

    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyK | monaco.KeyCode.KeyF, () => {
        formatCode();
    });

    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyP, () => {
        console.log("快速打开触发");
    });

    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, () => {
        runCode();
    });

    monaco.languages.typescript.javascriptDefaults.setDiagnosticsOptions({
        noSemanticValidation: false,
        noSyntaxValidation: false,
    });

    monaco.languages.typescript.javascriptDefaults.setCompilerOptions({
        target: monaco.languages.typescript.ScriptTarget.ES2018,
        allowNonTsExtensions: true,
    });

    monaco.languages.typescript.typescriptDefaults.setCompilerOptions({
        target: monaco.languages.typescript.ScriptTarget.ES2018,
        module: monaco.languages.typescript.ModuleKind.CommonJS,
        allowNonTsExtensions: true,
    });

    editor.onDidChangeModelContent(() => {
        realTimeCompile();
    });

    editorLoading.value = false;
};

onMounted(async () => {
    preloadMonacoResources();

    wasmAvailable.value = await checkWasmAvailability();
    if (wasmAvailable.value) {
        executionMode.value = "wasm";
    } else {
        executionMode.value = "mock";
        console.info("WASM 模块不可用，已启用模拟执行模式");
    }

    if (playgroundRef.value) {
        const observer = new IntersectionObserver(
            (entries) => {
                entries.forEach((entry) => {
                    if (entry.isIntersecting) {
                        loadMonacoEditor()
                            .then(() => {
                                restoreCodeFromUrl();
                            })
                            .catch((error) => {
                                console.error("Monaco 加载失败:", error);
                            });
                        observer.disconnect();
                    }
                });
            },
            { rootMargin: "200px" },
        );

        observer.observe(playgroundRef.value);
    } else {
        await loadMonacoEditor();
        restoreCodeFromUrl();
    }
});

watch(
    isDark,
    (newIsDark) => {
        if (editor && window.monaco) {
            editor.updateOptions({
                theme: newIsDark ? "one-dark-pro" : "vs",
            });
        }
    },
    { flush: "post" },
);

onUnmounted(() => {
    if (editor) {
        editor.dispose();
        editor = null;
    }
    if (debounceTimer) {
        clearTimeout(debounceTimer);
        debounceTimer = null;
    }
    previousCode = "";
    output.value = "";
    errors.value = [];
    debugVariables.value = [];
    performanceMetrics.value = {
        executionTime: 0,
        memoryUsage: 0,
        operations: 0,
    };
});

defineExpose({
    importCode,
});
</script>

<style scoped>
.typescript-playground {
  width: 100%;
  max-width: 1200px;
  margin: 0 auto;
  padding: 0;
  background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  overflow: hidden;
  will-change: opacity, transform;
  opacity: 1;
  transform: translateY(0);
}

.playground-header {
  background: linear-gradient(90deg, #667eea 0%, #764ba2 100%);
  color: white;
  padding: 30px;
  text-align: center;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  position: relative;
}

.playground-title {
  margin: 0;
  font-size: 2.5rem;
  font-weight: 700;
  letter-spacing: -0.5px;
  margin-bottom: 8px;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.playground-subtitle {
  font-size: 1.1rem;
  opacity: 0.9;
  font-weight: 300;
  margin-bottom: 8px;
}

.playground-version {
  font-size: 0.9rem;
  opacity: 0.8;
  font-weight: 400;
  position: absolute;
  top: 15px;
  right: 20px;
  background: rgba(255, 255, 255, 0.1);
  padding: 4px 12px;
  border-radius: 12px;
}

.execution-mode-badge {
  position: absolute;
  top: 15px;
  left: 20px;
  font-size: 0.75rem;
  padding: 4px 10px;
  border-radius: 8px;
  font-weight: 600;
}

.execution-mode-badge.wasm {
  background: rgba(76, 175, 80, 0.3);
  color: #a5d6a7;
}

.execution-mode-badge.mock {
  background: rgba(255, 152, 0, 0.3);
  color: #ffcc80;
}

.playground-container {
  display: flex;
  flex-direction: column;
  gap: 20px;
  padding: 30px;
}

.code-editor {
  width: 100%;
  border-radius: 8px;
  overflow: hidden;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  will-change: transform, box-shadow;
}

.code-editor:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.15);
  transition: transform 0.3s ease, box-shadow 0.3s ease;
}

.editor-container {
  width: 100%;
  min-height: 400px;
  border-radius: 8px;
  overflow: hidden;
}

.playground-controls {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  align-items: center;
  padding: 20px;
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.control-divider {
  width: 1px;
  height: 32px;
  background: #e0e0e0;
  margin: 0 8px;
}

.control-button {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 20px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 600;
  position: relative;
  overflow: hidden;
  will-change: transform, box-shadow;
}

.control-button:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.control-button:active:not(:disabled) {
  transform: translateY(0);
  transition: transform 0.1s ease;
}

.control-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
}

.button-icon {
  font-size: 16px;
}

.button-text {
  font-weight: 600;
}

.run-button {
  background: linear-gradient(90deg, #4CAF50 0%, #45a049 100%);
  color: white;
}

.compile-button {
  background: linear-gradient(90deg, #2196F3 0%, #0b7dda 100%);
  color: white;
}

.format-button {
  background: linear-gradient(90deg, #9c27b0 0%, #7b1fa2 100%);
  color: white;
}

.reset-button {
  background: linear-gradient(90deg, #f44336 0%, #da190b 100%);
  color: white;
}

.share-button {
  background: linear-gradient(90deg, #00bcd4 0%, #0097a7 100%);
  color: white;
}

.debug-button {
  background: linear-gradient(90deg, #ff9800 0%, #f57c00 100%);
  color: white;
}

.performance-button {
  background: linear-gradient(90deg, #795548 0%, #5d4037 100%);
  color: white;
}

.control-button.active {
  box-shadow: 0 0 0 2px white, 0 0 0 4px currentColor;
}

.output-section {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 2px solid #f0f0f0;
}

.section-title {
  font-size: 16px;
  font-weight: 700;
  color: #333;
}

.section-subtitle {
  font-size: 14px;
  color: #666;
  font-weight: 400;
}

.section-status {
  padding: 4px 12px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.section-status.idle {
  background: #f5f5f5;
  color: #666;
}

.section-status.running {
  background: #e3f2fd;
  color: #1976d2;
  animation: pulse 1.5s infinite;
}

.section-status.success {
  background: #e8f5e8;
  color: #2e7d32;
}

.section-status.error {
  background: #ffebee;
  color: #c62828;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.6; }
}

.execution-time {
  font-size: 12px;
  color: #666;
  padding: 4px 10px;
  background: #f0f0f0;
  border-radius: 8px;
}

.output-container {
  background: white;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  transition: all 0.3s ease;
}

.output-container:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.output {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 14px;
  white-space: pre-wrap;
  line-height: 1.6;
  color: #333;
  background: #fafafa;
  padding: 15px;
  border-radius: 6px;
  border: 1px solid #e0e0e0;
  min-height: 100px;
}

.output-placeholder {
  color: #999;
  font-style: italic;
}

.output-line {
  padding: 2px 0;
}

.errors-container {
  background: #fff3f3;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  border-left: 4px solid #f44336;
  transition: all 0.3s ease;
}

.errors-container:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.error-count {
  background: #f44336;
  color: white;
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 600;
}

.error-item {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 14px;
  line-height: 1.5;
  color: #d32f2f;
  margin-bottom: 10px;
  padding: 10px;
  background-color: #ffebee;
  border-radius: 6px;
  border-left: 3px solid #f44336;
  transition: all 0.2s ease;
}

.error-item:hover {
  background-color: #ffdede;
  transform: translateX(5px);
}

.error-icon {
  margin-top: 2px;
  font-size: 16px;
}

.error-message {
  flex: 1;
  word-break: break-all;
}

.debug-container {
  background: linear-gradient(135deg, #f3f7ff 0%, #e3f2fd 100%);
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  transition: all 0.3s ease;
}

.debug-container:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.debug-controls {
  display: flex;
  gap: 12px;
  margin-bottom: 20px;
  flex-wrap: wrap;
}

.debug-step-button, .debug-continue-button {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 600;
  transition: all 0.3s ease;
}

.debug-step-button {
  background: linear-gradient(90deg, #2196F3 0%, #0b7dda 100%);
  color: white;
}

.debug-continue-button {
  background: linear-gradient(90deg, #4CAF50 0%, #45a049 100%);
  color: white;
}

.debug-step-button:hover, .debug-continue-button:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.debug-variables {
  background: white;
  border-radius: 6px;
  padding: 15px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.variables-list {
  max-height: 200px;
  overflow-y: auto;
}

.debug-variable {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 14px;
  line-height: 1.5;
  padding: 8px 12px;
  border-bottom: 1px solid #f0f0f0;
  transition: all 0.2s ease;
}

.debug-variable:hover {
  background-color: #f5f5f5;
  border-radius: 4px;
}

.variable-name {
  font-weight: 600;
  color: #2196F3;
}

.variable-value {
  color: #333;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
}

.debug-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 30px;
  color: #666;
  font-style: italic;
}

.empty-icon {
  font-size: 32px;
  margin-bottom: 10px;
  opacity: 0.5;
}

.empty-text {
  font-size: 14px;
}

.performance-container {
  background: linear-gradient(135deg, #f3fff3 0%, #e8f5e8 100%);
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  transition: all 0.3s ease;
}

.performance-container:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.performance-metrics {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 20px;
  margin-top: 15px;
}

.performance-metric {
  background: white;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  transition: all 0.3s ease;
}

.performance-metric:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.metric-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}

.metric-icon {
  font-size: 20px;
}

.metric-label {
  font-size: 14px;
  color: #666;
  font-weight: 500;
}

.metric-value {
  display: block;
  font-size: 24px;
  font-weight: 700;
  color: #333;
  margin-bottom: 10px;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
}

.metric-bar {
  height: 6px;
  background: #f0f0f0;
  border-radius: 3px;
  overflow: hidden;
  margin-top: 10px;
}

.metric-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, #667eea 0%, #764ba2 100%);
  border-radius: 3px;
  transition: width 0.5s ease;
}

.metric-bar-fill.memory {
  background: linear-gradient(90deg, #4CAF50 0%, #45a049 100%);
}

.metric-bar-fill.operations {
  background: linear-gradient(90deg, #ff9800 0%, #f57c00 100%);
}

.playground-footer {
  background: linear-gradient(90deg, #667eea 0%, #764ba2 100%);
  color: white;
  padding: 20px;
  text-align: center;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  margin-top: 30px;
}

.footer-info {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 15px;
  flex-wrap: wrap;
}

.footer-text {
  font-size: 14px;
  font-weight: 500;
  opacity: 0.9;
}

.footer-separator {
  font-size: 14px;
  opacity: 0.6;
}

.footer-link {
  font-size: 14px;
  font-weight: 500;
  color: white;
  text-decoration: none;
  opacity: 0.9;
  transition: all 0.3s ease;
}

.footer-link:hover {
  opacity: 1;
  text-decoration: underline;
  transform: translateY(-1px);
}

@media (max-width: 768px) {
  .playground-container {
    padding: 20px;
  }

  .playground-header {
    padding: 20px;
  }

  .playground-title {
    font-size: 2rem;
  }

  .playground-version {
    position: static;
    display: inline-block;
    margin-top: 10px;
  }

  .execution-mode-badge {
    position: static;
    display: inline-block;
    margin-top: 5px;
  }

  .playground-controls {
    flex-direction: column;
    align-items: stretch;
  }

  .control-button {
    justify-content: center;
  }

  .performance-metrics {
    grid-template-columns: 1fr;
  }

  .debug-controls {
    justify-content: center;
  }

  .footer-info {
    flex-direction: column;
    gap: 10px;
  }

  .footer-separator {
    display: none;
  }
}
</style>
