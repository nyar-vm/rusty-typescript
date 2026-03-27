<template>
  <div class="min-h-screen bg-gradient-to-b from-blue-50 to-white">

    <div class="pt-20 pb-16">
      <div class="max-w-7xl mx-auto px-6 lg:px-8">
        <div class="text-center max-w-4xl mx-auto mb-12">
          <div class="inline-block px-4 py-2 bg-blue-100 text-blue-800 rounded-full text-sm font-semibold mb-6">
            在线编辑器
          </div>
          <h1 class="text-4xl md:text-5xl font-bold mb-6 text-blue-800">
            Rusty TypeScript Playground
          </h1>
          <p class="text-xl text-slate-700 mb-8">
            在线尝试 Rusty TypeScript，体验高性能 TypeScript 编译和运行
          </p>
        </div>

        <div class="flex flex-col lg:flex-row gap-6">
          <div class="lg:w-1/4 order-2 lg:order-1">
            <div class="bg-white rounded-2xl border border-blue-200 shadow-lg overflow-hidden sticky top-6">
              <div class="bg-gradient-to-r from-blue-600 to-blue-700 px-6 py-4">
                <h3 class="font-semibold text-white text-lg flex items-center gap-2">
                  <span class="text-xl">📚</span>
                  示例代码库
                </h3>
              </div>
              <div class="p-4 max-h-[calc(100vh-200px)] overflow-y-auto">
                <TypeScriptExamples @import="handleImportExample" />
              </div>
            </div>
          </div>

          <div class="lg:w-3/4 order-1 lg:order-2">
            <TypeScriptPlayground ref="playgroundRef" />
          </div>
        </div>

        <div class="mt-16 max-w-3xl mx-auto">
          <div class="bg-blue-50 rounded-2xl p-6 border border-blue-100">
            <h3 class="font-semibold text-blue-800 mb-4 flex items-center gap-2">
              <span class="text-xl">💡</span>
              使用提示
            </h3>
            <ul class="text-slate-700 space-y-2">
              <li class="flex items-start gap-2">
                <span class="text-blue-600 mt-1">•</span>
                <span>点击左侧示例卡片中的"导入到 Playground"按钮可快速加载示例代码</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="text-blue-600 mt-1">•</span>
                <span>使用 <kbd class="px-2 py-1 bg-white rounded border border-blue-200 text-sm">Ctrl+Enter</kbd> 快捷键运行代码</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="text-blue-600 mt-1">•</span>
                <span>编辑器支持实时代码检查和语法高亮</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="text-blue-600 mt-1">•</span>
                <span>当 WASM 模块不可用时，系统会自动切换到模拟执行模式</span>
              </li>
            </ul>
          </div>
        </div>

        <div class="mt-8 max-w-3xl mx-auto">
          <div class="bg-amber-50 rounded-2xl p-6 border border-amber-200">
            <h3 class="font-semibold text-amber-800 mb-4 flex items-center gap-2">
              <span class="text-xl">⚠️</span>
              注意事项
            </h3>
            <p class="text-slate-700">
              此 Playground 仅用于演示目的，可能会有一些功能限制。在生产环境中使用时，请确保已正确配置 WASM 模块。
            </p>
          </div>
        </div>
      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import TypeScriptPlayground from "../components/TypeScriptPlayground.vue";
import TypeScriptExamples from "../components/TypeScriptExamples.vue";

const playgroundRef = ref<InstanceType<typeof TypeScriptPlayground> | null>(null);

const handleImportExample = (code: string) => {
    if (playgroundRef.value) {
        playgroundRef.value.importCode(code);
    }
};
</script>
