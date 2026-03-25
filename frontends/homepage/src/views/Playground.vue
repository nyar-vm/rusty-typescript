<template>
  <div class="min-h-screen bg-gradient-to-b from-blue-50 to-white">

    <!-- 主要内容 -->
    <div class="pt-40 pb-32">
      <div class="max-w-7xl mx-auto px-6 lg:px-8">
        <div class="text-center max-w-4xl mx-auto mb-16">
          <div class="inline-block px-4 py-2 bg-blue-100 text-blue-800 rounded-full text-sm font-semibold mb-6">
            在线编辑器
          </div>
          <h1 class="text-4xl md:text-5xl font-bold mb-6 text-blue-800">
            Rusty TypeScript Playground
          </h1>
          <p class="text-xl text-slate-700 mb-12">
            在线尝试 Rusty TypeScript，体验高性能 TypeScript 编译和运行
          </p>
        </div>

        <div class="bg-white rounded-3xl border border-blue-200 shadow-lg p-6 md:p-8">
          <div class="flex justify-between items-center mb-6">
            <h3 class="font-semibold text-blue-800 text-lg">TypeScript 代码编辑器</h3>
            <div class="flex items-center gap-4">
              <button 
                @click="runCode"
                class="px-5 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 transition-colors shadow-sm"
              >
                运行
              </button>
              <button 
                @click="clearOutput"
                class="px-5 py-2 bg-white text-blue-800 border border-blue-200 rounded-lg text-sm font-medium hover:bg-blue-50 transition-colors"
              >
                清除输出
              </button>
            </div>
          </div>
          
          <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <!-- 编辑器 -->
            <div class="h-[600px] rounded-2xl overflow-hidden border border-blue-200">
              <div ref="editorContainer" style="width: 100%; height: 100%"></div>
            </div>
            
            <!-- 输出结果 -->
            <div class="h-[600px] rounded-2xl overflow-hidden border border-blue-200 bg-slate-900">
              <div class="bg-slate-800 px-4 py-3 border-b border-slate-700">
                <h4 class="text-slate-300 font-medium">输出结果</h4>
              </div>
              <div class="p-4 h-[calc(100%-52px)] overflow-auto">
                <pre class="text-slate-300 font-mono text-sm whitespace-pre-wrap">{{ output }}</pre>
              </div>
            </div>
          </div>
        </div>

        <div class="mt-16 max-w-3xl mx-auto">
          <div class="bg-blue-50 rounded-2xl p-6 border border-blue-100">
            <h3 class="font-semibold text-blue-800 mb-4">关于 Playground</h3>
            <p class="text-slate-700 mb-4">
              Rusty TypeScript Playground 允许您在线编写、编译和运行 TypeScript 代码，体验 Rusty TypeScript 的高性能特性。
            </p>
            <p class="text-slate-700">
              请注意，此 Playground 仅用于演示目的，可能会有一些功能限制。
            </p>
          </div>
        </div>
      </div>
    </div>


  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import * as monaco from "monaco-editor";

const code = ref(`// 在这里编写 TypeScript 代码
function fibonacci(n: number): number {
  if (n <= 1) return n;
  return fibonacci(n - 1) + fibonacci(n - 2);
}

console.log(fibonacci(10));

interface Person {
  name: string;
  age: number;
}

const person: Person = {
  name: "Rusty TypeScript",
  age: 1
};

console.log(person);`);

const output = ref(`55
{ name: "Rusty TypeScript", age: 1 }`);

const editorContainer = ref<HTMLElement | null>(null);
let editor: monaco.editor.IStandaloneCodeEditor | null = null;

const editorOptions = {
    minimap: { enabled: true },
    scrollBeyondLastLine: false,
    automaticLayout: true,
    tabSize: 2,
    fontFamily: '"Fira Code", Consolas, "Courier New", monospace',
    fontSize: 14,
    lineNumbers: "on" as any,
    wordWrap: "on" as const,
    scrollbar: {
        vertical: "auto" as const,
        horizontal: "auto" as const,
    },
};

onMounted(() => {
    if (editorContainer.value) {
        editor = monaco.editor.create(editorContainer.value, {
            value: code.value,
            language: "typescript",
            theme: "vs-dark",
            ...editorOptions,
        });

        // 监听编辑器内容变化
        editor.onDidChangeModelContent(() => {
            if (editor) {
                code.value = editor.getValue();
            }
        });
    }
});

onUnmounted(() => {
    if (editor) {
        editor.dispose();
        editor = null;
    }
});

function runCode() {
    // 模拟运行代码
    try {
        // 这里应该是调用 Rusty TypeScript 编译器和运行时
        // 现在只是模拟输出
        output.value = `运行结果：
55
{ name: "Rusty TypeScript", age: 1 }

编译时间：0.12s
运行时间：0.05s`;
    } catch (error) {
        output.value = `错误：
${error instanceof Error ? error.message : "未知错误"}`;
    }
}

function clearOutput() {
    output.value = "";
}
</script>