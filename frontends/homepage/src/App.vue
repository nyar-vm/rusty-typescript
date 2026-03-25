<template>
  <div class="min-h-screen bg-gradient-to-br from-blue-50 to-sky-50 text-slate-800 relative overflow-hidden flex flex-col transition-colors duration-300 will-change:background-color" :class="{ 'dark': isDark }">
    <div class="fixed inset-0 z-0 pointer-events-none">
      <div class="absolute top-[-10%] left-[-10%] w-[500px] h-[500px] rounded-full blur-[120px] bg-blue-400/10 transform-gpu"></div>
      <div class="absolute top-[20%] right-[-5%] w-[600px] h-[600px] rounded-full blur-[100px] bg-sky-400/10 transform-gpu"></div>
      <div class="absolute bottom-[-10%] left-[30%] w-[550px] h-[550px] rounded-full blur-[110px] bg-indigo-400/10 transform-gpu"></div>
    </div>

    <header class="fixed top-0 left-0 right-0 z-50 border-b border-blue-200 bg-white/90 backdrop-blur-xl shadow-sm">
      <div class="max-w-7xl mx-auto px-6 lg:px-8">
        <div class="flex items-center justify-between h-20">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 bg-gradient-to-br from-blue-600 via-sky-600 to-indigo-600 rounded-xl flex items-center justify-center shadow-md shadow-blue-500/20">
              <span class="text-white font-bold text-xl">R</span>
            </div>
            <span class="text-2xl font-bold text-blue-800">Rusty TypeScript</span>
          </div>
          
          <nav class="hidden md:flex items-center gap-1">
            <router-link 
              to="/"
              active-class="bg-blue-500/10 text-blue-800 border border-blue-200"
              class="px-5 py-2.5 rounded-xl text-sm font-semibold transition-all duration-300 text-slate-600 hover:text-blue-800 hover:bg-blue-50"
            >
              首页
            </router-link>
            <router-link 
              to="/install"
              active-class="bg-blue-500/10 text-blue-800 border border-blue-200"
              class="px-5 py-2.5 rounded-xl text-sm font-semibold transition-all duration-300 text-slate-600 hover:text-blue-800 hover:bg-blue-50"
            >
              安装
            </router-link>

            <router-link 
              to="/playground"
              active-class="bg-blue-500/10 text-blue-800 border border-blue-200"
              class="px-5 py-2.5 rounded-xl text-sm font-semibold transition-all duration-300 text-slate-600 hover:text-blue-800 hover:bg-blue-50"
            >
              Playground
            </router-link>
            <a 
              :href="docsUrl"
              target="_blank"
              rel="noopener noreferrer"
              class="px-5 py-2.5 rounded-xl text-sm font-semibold transition-all duration-300 text-slate-600 hover:text-blue-800 hover:bg-blue-50"
            >
              文档
            </a>
          </nav>

          <div class="flex items-center gap-2">
            <button 
              @click="showSettings = true"
              class="flex items-center justify-center w-10 h-10 rounded-xl text-sm font-semibold transition-all duration-300 text-slate-700 hover:text-blue-800 hover:bg-blue-50"
              title="设置"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"></path>
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path>
              </svg>
            </button>
            <a href="https://github.com/nyar-vm/rusty-typescript/tree/dev" target="_blank" class="hidden md:flex items-center justify-center w-10 h-10 rounded-xl text-sm font-semibold transition-all duration-300 text-slate-700 hover:text-blue-800 hover:bg-blue-50" title="GitHub">
              <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24"><path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/></svg>
            </a>
          </div>
        </div>
      </div>
    </header>

    <main class="relative z-10 flex-1">
      <router-view />
    </main>

    <footer class="relative z-10 border-t border-blue-200 bg-blue-50">
      <div class="max-w-7xl mx-auto px-6 lg:px-8 py-16">
        <div class="grid grid-cols-2 md:grid-cols-4 gap-12 mb-12">
          <div class="col-span-2 md:col-span-1">
            <div class="flex items-center gap-3 mb-6">
              <div class="w-10 h-10 bg-gradient-to-br from-blue-600 via-sky-600 to-indigo-600 rounded-xl flex items-center justify-center">
                <span class="text-white font-bold text-xl">R</span>
              </div>
              <span class="text-2xl font-bold text-blue-800">Rusty TypeScript</span>
            </div>
            <p class="text-slate-600 text-sm leading-relaxed">
              Rust 实现的 TypeScript 编译器和运行时，提供高性能、类型安全的 TypeScript 代码执行环境，为 TypeScript 生态带来新的可能性。
            </p>
          </div>
          
          <div>
            <h4 class="text-blue-800 font-semibold mb-4">产品</h4>
            <ul class="space-y-3">
              <li><a href="#" class="text-slate-600 hover:text-blue-800 transition-colors text-sm">功能特性</a></li>
              <li><a href="#" class="text-slate-600 hover:text-blue-800 transition-colors text-sm">下载</a></li>
              <li><a href="#" class="text-slate-600 hover:text-blue-800 transition-colors text-sm">更新日志</a></li>
            </ul>
          </div>
          
          <div>
            <h4 class="text-blue-800 font-semibold mb-4">资源</h4>
            <ul class="space-y-3">
              <li><a :href="docsUrl" target="_blank" rel="noopener noreferrer" class="text-slate-600 hover:text-blue-800 transition-colors text-sm">文档</a></li>
              <li><a href="#" class="text-slate-600 hover:text-blue-800 transition-colors text-sm">教程</a></li>
              <li><a href="#" class="text-slate-600 hover:text-blue-800 transition-colors text-sm">API</a></li>
            </ul>
          </div>
          
          <div>
            <h4 class="text-blue-800 font-semibold mb-4">社区</h4>
            <ul class="space-y-3">
              <li><a href="#" class="text-slate-600 hover:text-blue-800 transition-colors text-sm">GitHub</a></li>
              <li><a href="#" class="text-slate-600 hover:text-blue-800 transition-colors text-sm">Discord</a></li>
              <li><a href="#" class="text-slate-600 hover:text-blue-800 transition-colors text-sm">Twitter</a></li>
            </ul>
          </div>
        </div>
        
        <div class="pt-8 border-t border-blue-200 flex flex-col md:flex-row justify-between items-center gap-4">
          <p class="text-slate-600 text-sm">
            © 2026 Rusty TypeScript. All rights reserved.
          </p>
          <div class="flex items-center gap-6">
            <a href="#" class="text-slate-600 hover:text-blue-800 transition-colors text-sm">隐私政策</a>
            <a href="#" class="text-slate-600 hover:text-blue-800 transition-colors text-sm">服务条款</a>
          </div>
        </div>
      </div>
    </footer>

    <!-- 设置模态框 -->
    <div v-if="showSettings" class="fixed inset-0 z-50 flex items-center justify-center p-4">
      <div class="absolute inset-0 bg-black/50 backdrop-blur-sm" @click="showSettings = false"></div>
      <div class="relative z-10">
        <Settings :visible="showSettings" @close="showSettings = false" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { useTheme } from "./composables/useTheme";
import Settings from "./components/Settings.vue";

const { isDark } = useTheme();
const showSettings = ref(false);

const docsUrl = computed(() => {
    const language = navigator.language.startsWith("zh") ? "zh" : "en";
    return language === "zh"
        ? "https://www.typescriptlang.org/zh/docs/"
        : "https://www.typescriptlang.org/docs/";
});
</script>
