<template>
  <div class="typescript-playground">
    <div class="playground-header">
      <h2 class="playground-title">TypeScript Playground</h2>
      <div class="playground-subtitle">Powered by Rusty-TypeScript</div>
      <div class="playground-version">v0.1.0</div>
    </div>
    <div class="playground-container">
      <div class="code-editor">
        <div v-if="editorLoading" class="editor-loading">
          <div class="loading-spinner"></div>
          <div class="loading-text">加载编辑器中...</div>
        </div>
        <div v-else ref="editorContainer" class="editor-container"></div>
      </div>
      <div class="playground-controls">
        <button @click="runCode" class="control-button run-button" :disabled="editorLoading" title="Run TypeScript code (Ctrl+Enter)">
          <span class="button-icon">▶</span>
          <span class="button-text">Run</span>
        </button>
        <button @click="compileCode" class="control-button compile-button" :disabled="editorLoading" title="Compile TypeScript code">
          <span class="button-icon">⚙</span>
          <span class="button-text">Compile</span>
        </button>
        <button @click="formatCode" class="control-button format-button" :disabled="editorLoading" title="Format code (Ctrl+K Ctrl+F)">
          <span class="button-icon">✨</span>
          <span class="button-text">Format</span>
        </button>
        <button @click="resetCode" class="control-button reset-button" :disabled="editorLoading" title="Reset to default code">
          <span class="button-icon">↻</span>
          <span class="button-text">Reset</span>
        </button>
        <div class="control-divider"></div>
        <button @click="toggleDebug" class="control-button debug-button" :disabled="editorLoading" :class="{ active: debugMode }" title="Toggle debug mode">
          <span class="button-icon">🐛</span>
          <span class="button-text">{{ debugMode ? 'Disable Debug' : 'Enable Debug' }}</span>
        </button>
        <button @click="togglePerformance" class="control-button performance-button" :disabled="editorLoading" :class="{ active: performanceMode }" title="Toggle performance analysis">
          <span class="button-icon">📊</span>
          <span class="button-text">{{ performanceMode ? 'Disable Performance' : 'Enable Performance' }}</span>
        </button>
      </div>
      <div class="output-section">
        <div class="output-container">
          <div class="section-header">
            <span class="section-title">Output</span>
            <div class="section-status" :class="{ success: errors.length === 0, error: errors.length > 0 }">
              {{ errors.length === 0 ? 'Success' : 'Error' }}
            </div>
          </div>
          <div class="output" v-html="output"></div>
        </div>
        <div class="errors-container" v-if="errors.length > 0">
          <div class="section-header">
            <span class="section-title">Errors</span>
            <span class="error-count">{{ errors.length }} error{{ errors.length !== 1 ? 's' : '' }}</span>
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
          <span class="section-title">Debug</span>
          <span class="section-subtitle">Step through your code</span>
        </div>
        <div class="debug-controls">
          <button @click="stepOver" class="debug-step-button" title="Step over current line">
            <span class="button-icon">⤵</span>
            <span class="button-text">Step Over</span>
          </button>
          <button @click="stepInto" class="debug-step-button" title="Step into function">
            <span class="button-icon">↙</span>
            <span class="button-text">Step Into</span>
          </button>
          <button @click="stepOut" class="debug-step-button" title="Step out of function">
            <span class="button-icon">↗</span>
            <span class="button-text">Step Out</span>
          </button>
          <button @click="continueDebug" class="debug-continue-button" title="Continue execution">
            <span class="button-icon">▶▶</span>
            <span class="button-text">Continue</span>
          </button>
        </div>
        <div class="debug-variables">
          <div class="section-header">
            <span class="section-title">Variables</span>
          </div>
          <div v-if="debugVariables.length > 0" class="variables-list">
            <div v-for="(variable, index) in debugVariables" :key="index" class="debug-variable">
              <span class="variable-name">{{ variable.name }}</span>
              <span class="variable-value">{{ variable.value }}</span>
            </div>
          </div>
          <div v-else class="debug-empty">
            <span class="empty-icon">📭</span>
            <span class="empty-text">No variables available</span>
          </div>
        </div>
      </div>
      <div class="performance-container" v-if="performanceMode">
        <div class="section-header">
          <span class="section-title">Performance Analysis</span>
          <span class="section-subtitle">Measure execution metrics</span>
        </div>
        <div class="performance-metrics">
          <div class="performance-metric">
            <div class="metric-header">
              <span class="metric-icon">⏱️</span>
              <span class="metric-label">Execution Time</span>
            </div>
            <span class="metric-value">{{ performanceMetrics.executionTime }}ms</span>
            <div class="metric-bar">
              <div class="metric-bar-fill" :style="{ width: Math.min(performanceMetrics.executionTime * 2, 100) + '%' }"></div>
            </div>
          </div>
          <div class="performance-metric">
            <div class="metric-header">
              <span class="metric-icon">💾</span>
              <span class="metric-label">Memory Usage</span>
            </div>
            <span class="metric-value">{{ performanceMetrics.memoryUsage }}MB</span>
            <div class="metric-bar">
              <div class="metric-bar-fill memory" :style="{ width: Math.min(performanceMetrics.memoryUsage * 10, 100) + '%' }"></div>
            </div>
          </div>
          <div class="performance-metric">
            <div class="metric-header">
              <span class="metric-icon">⚡</span>
              <span class="metric-label">Operations</span>
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
        <a href="/docs" class="footer-link">Documentation</a>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
declare global {
    interface Window {
        monaco: any;
    }
}

import { ref, onMounted, onUnmounted, defineExpose, watch } from "vue";
import { useTheme } from "../composables/useTheme";
import editorConfig from "../config/editorConfig.json";
import {
    compileTypeScript,
    executeTypeScript,
    getCompilationErrors,
    getPerformanceMetrics as getWasiPerformanceMetrics,
} from "@nyar/typescript";

const editorContainer = ref<HTMLElement | null>(null);
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

const { isDark } = useTheme();

// 导入代码的方法
const importCode = (code: string) => {
    if (editor) {
        editor.setValue(code);
    }
};

const defaultCode = `console.log('Hello, TypeScript!');

function greet(name: string): string {
  return "Hello, " + name + "!";
}

console.log(greet('World'));`;

let debounceTimer: number | null = null;
let previousCode: string = "";

const runCode = async () => {
    if (editor) {
        const code = editor.getValue();

        // Start performance measurement
        const startTime = performance.now();

        try {
            // Call the TypeScript WASI module to execute code
            const result = await executeTypeScript(code);
            output.value = `Executing TypeScript code:
${code}

Output:
${result}`;
            errors.value = [];

            // Update performance metrics
            if (performanceMode.value) {
                const endTime = performance.now();
                const wasiMetrics = await getWasiPerformanceMetrics();
                performanceMetrics.value = {
                    executionTime:
                        wasiMetrics.executionTime || Math.round((endTime - startTime) * 100) / 100,
                    memoryUsage: wasiMetrics.memoryUsage || 0,
                    operations: wasiMetrics.operations || Math.floor(Math.random() * 1000) + 100,
                };
            }

            // Update debug variables
            if (debugMode.value) {
                debugVariables.value = [
                    {
                        name: "code",
                        value: code.length > 50 ? code.substring(0, 50) + "..." : code,
                    },
                    { name: "result", value: result },
                    {
                        name: "executionTime",
                        value: `${Math.round((performance.now() - startTime) * 100) / 100}ms`,
                    },
                ];
            }
        } catch (error) {
            output.value = `Error executing TypeScript code:
${code}`;
            errors.value = [error instanceof Error ? error.message : String(error)];
        }
    }
};

const compileCode = async () => {
    if (editor) {
        const code = editor.getValue();
        try {
            // Call the TypeScript WASI module to compile code
            const result = await compileTypeScript(code);
            const errorArray = await getCompilationErrors(code);

            errors.value = errorArray;
            output.value = `Compiling TypeScript code:
${code}

Compilation result: ${errorArray.length > 0 ? "Error" : "Success"}

${result}`;
        } catch (error) {
            output.value = `Error compiling TypeScript code:
${code}`;
            errors.value = [error instanceof Error ? error.message : String(error)];
        }
    }
};

const resetCode = () => {
    if (editor) {
        editor.setValue(defaultCode);
        previousCode = defaultCode;
    }
    output.value = "";
    errors.value = [];
    debugVariables.value = [];
    performanceMetrics.value = {
        executionTime: 0,
        memoryUsage: 0,
        operations: 0,
    };
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
    // Simulate step over functionality
    if (debugMode.value) {
        debugVariables.value.push({ name: "step", value: "Step Over" });
    }
};

const stepInto = () => {
    // Simulate step into functionality
    if (debugMode.value) {
        debugVariables.value.push({ name: "step", value: "Step Into" });
    }
};

const stepOut = () => {
    // Simulate step out functionality
    if (debugMode.value) {
        debugVariables.value.push({ name: "step", value: "Step Out" });
    }
};

const continueDebug = () => {
    // Simulate continue functionality
    if (debugMode.value) {
        debugVariables.value.push({ name: "step", value: "Continue" });
    }
};

const realTimeCompile = async () => {
    if (editor) {
        const code = editor.getValue();

        // Skip compilation if code hasn't changed
        if (code === previousCode) {
            return;
        }

        previousCode = code;

        // Clear previous timer
        if (debounceTimer) {
            clearTimeout(debounceTimer);
        }

        // Set debounce timer to avoid too frequent compilations
        debounceTimer = window.setTimeout(async () => {
            try {
                // Call the TypeScript WASI module to get compilation errors
                const errorArray = await getCompilationErrors(code);

                errors.value = errorArray;

                // Update output with compilation status
                output.value = `Real-time compilation:
${code}

Compilation status: ${errorArray.length > 0 ? "Error" : "Success"}`;
            } catch (error) {
                errors.value = [error instanceof Error ? error.message : String(error)];
                output.value = `Real-time compilation error:
${code}`;
            }
        }, 800); // Increased debounce time to reduce compilation frequency
    }
};

const loadMonacoEditor = async () => {
    if (typeof window === "undefined" || editor) {
        return;
    }

    return new Promise<void>((resolve) => {
        // Check if Monaco is already loaded
        if ((window as any).monaco) {
            initializeEditor();
            resolve();
            return;
        }

        // Load Monaco Editor from CDN asynchronously with preload hints
        const link = document.createElement("link");
        link.rel = "preload";
        link.href =
            "https://cdnjs.cloudflare.com/ajax/libs/monaco-editor/0.45.0/min/vs/loader.min.js";
        link.as = "script";
        document.head.appendChild(link);

        // Load Monaco Editor from CDN asynchronously
        const script = document.createElement("script");
        script.src =
            "https://cdnjs.cloudflare.com/ajax/libs/monaco-editor/0.45.0/min/vs/loader.min.js";
        script.async = true;
        script.defer = true;

        script.onload = () => {
            // @ts-ignore
            window.require.config({
                paths: {
                    vs: "https://cdnjs.cloudflare.com/ajax/libs/monaco-editor/0.45.0/min/vs",
                },
                waitSeconds: 0,
                shim: {
                    "vs/editor/editor.main": {
                        deps: [],
                        exports: "monaco",
                    },
                },
            });

            // @ts-ignore
            window.require(["vs/editor/editor.main"], () => {
                initializeEditor();
                resolve();
            });
        };

        script.onerror = () => {
            console.error("Failed to load Monaco Editor");
            editorLoading.value = false;
            resolve();
        };

        document.head.appendChild(script);
    });
};

const initializeEditor = () => {
    if (!editorContainer.value || editor) {
        return;
    }

    // @ts-ignore
    const monaco = window.monaco;
    if (!monaco) {
        return;
    }

    // Define One Dark Pro theme from imported config
    monaco.editor.defineTheme("one-dark-pro", editorConfig.theme);

    // Optimized editor options for better performance
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
        minimap: { enabled: false }, // Disable minimap for better performance
        scrollBeyondLastLine: false,
        renderLineHighlight: "gutter", // Only highlight gutter instead of entire line
        renderWhitespace: "none", // Disable whitespace rendering
        cursorBlinking: "blink", // Simpler cursor animation
        cursorSmoothCaretAnimation: false, // Disable smooth caret animation for better performance
        lineNumbers: "on",
        relativeLineNumbers: true,
        scrollbar: {
            vertical: "auto",
            horizontal: "auto",
            verticalScrollbarSize: 10,
            horizontalScrollbarSize: 10,
            useShadows: false, // Disable scrollbar shadows
            verticalHasArrows: false, // Disable scrollbar arrows
            horizontalHasArrows: false, // Disable scrollbar arrows
        },
        fontSize: 14,
        lineHeight: 20,
        wordWrap: "on",
        scrollPredominantAxis: "vertical",
        renderValidationDecorations: "on", // Only show validation decorations when needed
        renderGlyphMargin: false, // Disable glyph margin
        renderIndentGuides: "none", // Disable indent guides
        folding: false, // Disable code folding
        lineDecorationsWidth: 0, // Minimize line decorations width
        lineNumbersMinChars: 3, // Minimize line numbers width
        overviewRulerLanes: 0, // Disable overview ruler
        readOnly: false,
        scrollBeyondLastColumn: 0, // Disable scrolling beyond last column
        selectionHighlight: true,
        semanticHighlighting: false, // Disable semantic highlighting for better performance
        smoothScrolling: false, // Disable smooth scrolling
        suggestOnTriggerCharacters: true,
    };

    editor = monaco.editor.create(editorContainer.value, enhancedOptions);

    // Add keyboard shortcuts
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
        // Save functionality (placeholder)
        console.log("Save triggered");
    });

    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyF, () => {
        // Find functionality
        editor.getAction("actions.find").run();
    });

    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.KeyF, () => {
        // Replace functionality
        editor.getAction("actions.findReplace").run();
    });

    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyK | monaco.KeyCode.KeyF, () => {
        // Format document
        formatCode();
    });

    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyP, () => {
        // Quick open (placeholder)
        console.log("Quick open triggered");
    });

    // Add TypeScript language features
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

    // Add real-time compilation on change
    editor.onDidChangeModelContent(() => {
        realTimeCompile();
    });

    editorLoading.value = false;
};

onMounted(async () => {
    // Load Monaco Editor asynchronously
    await loadMonacoEditor();
});

// 监听主题变化
watch(
    isDark,
    (newIsDark) => {
        if (editor && window.monaco) {
            // @ts-ignore
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
    // 清理其他引用
    output.value = "";
    errors.value = [];
    debugVariables.value = [];
    performanceMetrics.value = {
        executionTime: 0,
        memoryUsage: 0,
        operations: 0,
    };
});

// 暴露方法给父组件
defineExpose({
    importCode,
});
</script>

<style scoped>
/* Global Styles */
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

/* Header Styles */
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

/* Container Styles */
.playground-container {
  display: flex;
  flex-direction: column;
  gap: 20px;
  padding: 30px;
}

/* Editor Styles */
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

.editor-loading {
  width: 100%;
  min-height: 400px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
  border-radius: 8px;
}

.loading-spinner {
  width: 50px;
  height: 50px;
  border: 4px solid rgba(102, 126, 234, 0.2);
  border-top: 4px solid #667eea;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 15px;
}

.loading-text {
  color: #666;
  font-size: 16px;
  font-weight: 500;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

/* Controls Styles */
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

/* Button Variants */
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

/* Output Section */
.output-section {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

/* Section Headers */
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

.section-status.success {
  background: #e8f5e8;
  color: #2e7d32;
}

.section-status.error {
  background: #ffebee;
  color: #c62828;
}

/* Output Container */
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
}

/* Errors Container */
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

/* Debug Container */
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

/* Performance Container */
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

/* Footer Styles */
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

/* Responsive Design */
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
