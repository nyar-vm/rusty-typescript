<template>
  <div class="min-h-screen bg-gradient-to-b from-blue-50 to-white">

    <!-- 英雄区域 -->
    <div class="pt-40 pb-32">
      <div class="max-w-7xl mx-auto px-6 lg:px-8">
        <div class="text-center max-w-4xl mx-auto mb-32">
          <div class="inline-block px-4 py-2 bg-blue-100 text-blue-800 rounded-full text-sm font-semibold mb-6">
            Rust 实现的 TypeScript 编译器和运行时
          </div>
          <h1 class="text-5xl md:text-7xl lg:text-8xl font-bold leading-tight mb-8">
            <span class="bg-gradient-to-r from-blue-700 via-sky-700 to-indigo-700 bg-clip-text text-transparent">Rusty TypeScript</span>
            <br>
            <span class="bg-gradient-to-r from-blue-600 via-sky-600 to-indigo-600 bg-clip-text text-transparent">高性能 TypeScript 实现</span>
          </h1>
          
          <p class="text-xl md:text-2xl text-slate-700 mb-12 max-w-2xl mx-auto leading-relaxed">
            Rusty TypeScript 是一个用 Rust 实现的 TypeScript 编译器和运行时，提供高性能、类型安全的 TypeScript 代码执行环境，为 TypeScript 生态带来新的可能性。
          </p>
          
          <div class="flex flex-col sm:flex-row gap-6 justify-center items-center mb-16">
            <router-link 
              to="/install"
              class="group relative overflow-hidden px-10 py-4 rounded-2xl text-lg font-bold bg-gradient-to-r from-blue-600 via-sky-600 to-indigo-600 text-white shadow-lg shadow-blue-500/20 hover:shadow-blue-500/30 transition-all duration-300 hover:-translate-y-1 transform hover:scale-105"
            >
              <span class="relative z-10 flex items-center gap-3">
                立即下载
                <svg class="w-5 h-5 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"></path>
                </svg>
              </span>
            </router-link>
            <router-link 
              to="/playground"
              class="group relative overflow-hidden px-10 py-4 rounded-2xl text-lg font-bold bg-white text-blue-700 border border-blue-200 shadow-lg shadow-blue-100/30 hover:shadow-blue-200/50 transition-all duration-300 hover:-translate-y-1 transform hover:scale-105"
            >
              <span class="relative z-10 flex items-center gap-3">
                在线演示
                <svg class="w-5 h-5 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"></path>
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                </svg>
              </span>
            </router-link>
          </div>

          <!-- 特性卡片 -->
          <div class="max-w-5xl mx-auto">
            <div class="bg-white rounded-3xl border border-blue-200 p-8 shadow-lg shadow-blue-100/50">
              <h3 class="text-2xl font-bold text-blue-800 mb-4">什么是 Rusty TypeScript？</h3>
              <p class="text-slate-700 mb-8">Rusty TypeScript 是一个用 Rust 语言实现的 TypeScript 编译器和运行时，旨在提供高性能、类型安全的 TypeScript 代码执行环境。</p>
              <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div class="bg-blue-50 rounded-2xl p-6 border border-blue-100 hover:shadow-md transition-shadow">
                  <div class="w-12 h-12 bg-blue-100 rounded-xl flex items-center justify-center mb-4">
                    <span class="text-blue-800 text-2xl">⚡</span>
                  </div>
                  <h4 class="font-semibold text-blue-800 mb-2">高性能</h4>
                  <p class="text-sm text-slate-600">基于 Rust 实现，编译期优化，提供极致的性能表现</p>
                </div>
                <div class="bg-blue-50 rounded-2xl p-6 border border-blue-100 hover:shadow-md transition-shadow">
                  <div class="w-12 h-12 bg-blue-100 rounded-xl flex items-center justify-center mb-4">
                    <span class="text-blue-800 text-2xl">🛡️</span>
                  </div>
                  <h4 class="font-semibold text-blue-800 mb-2">类型安全</h4>
                  <p class="text-sm text-slate-600">强类型设计，编译期检查，减少运行时错误</p>
                </div>
                <div class="bg-blue-50 rounded-2xl p-6 border border-blue-100 hover:shadow-md transition-shadow">
                  <div class="w-12 h-12 bg-blue-100 rounded-xl flex items-center justify-center mb-4">
                    <span class="text-blue-800 text-2xl">🔄</span>
                  </div>
                  <h4 class="font-semibold text-blue-800 mb-2">易于集成</h4>
                  <p class="text-sm text-slate-600">与现有 TypeScript 生态无缝集成，支持标准 TypeScript 语法</p>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 特性部分 -->
        <div class="space-y-32">
          <!-- 特性 1 -->
          <div class="grid md:grid-cols-2 gap-16 items-center">
            <div class="order-2 md:order-1">
              <div class="text-blue-600 text-sm font-semibold mb-4">📊 高性能编译</div>
              <h3 class="text-3xl md:text-4xl font-bold text-blue-800 mb-6">Rust 驱动的编译器</h3>
              <p class="text-lg text-slate-700 leading-relaxed mb-8">Rusty TypeScript 使用 Rust 语言实现编译器和运行时，编译期优化生成高效代码，提供极致的性能表现。</p>
              <div class="flex items-center gap-4 mb-6">
                <div class="w-12 h-12 bg-blue-100 rounded-xl flex items-center justify-center">
                  <span class="text-blue-800">📝</span>
                </div>
                <div>
                  <h4 class="font-semibold text-blue-800">标准 TypeScript 语法</h4>
                  <p class="text-sm text-slate-600">支持标准 TypeScript 语法，与现有 TypeScript 代码兼容</p>
                </div>
              </div>
              <div class="flex items-center gap-4">
                <div class="w-12 h-12 bg-blue-100 rounded-xl flex items-center justify-center">
                  <span class="text-blue-800">⚙️</span>
                </div>
                <div>
                  <h4 class="font-semibold text-blue-800">编译期优化</h4>
                  <p class="text-sm text-slate-600">生成高效的机器码，减少运行时开销</p>
                </div>
              </div>
            </div>
            <div class="order-1 md:order-2 relative">
              <div class="aspect-video bg-gradient-to-br from-blue-100 to-sky-100 rounded-3xl border border-blue-200 p-8 shadow-lg shadow-blue-100">
                <div class="bg-white rounded-2xl p-4 h-full flex flex-col">
                  <div class="flex items-center gap-2 mb-4">
                    <div class="w-3 h-3 rounded-full bg-red-400"></div>
                    <div class="w-3 h-3 rounded-full bg-yellow-400"></div>
                    <div class="w-3 h-3 rounded-full bg-green-400"></div>
                  </div>
                  <div class="font-mono text-sm text-slate-800 flex-1">
                    <pre class="whitespace-pre-wrap">// 编译 TypeScript 代码
$ rusty-ts compile app.ts

// 运行编译后的代码
$ rusty-ts run app

// 性能对比
Rusty TypeScript: 1.2s
TypeScript + Node: 3.5s</pre>
                  </div>
                </div>
              </div>
              <div class="absolute -bottom-4 -right-4 w-24 h-24 bg-blue-200 rounded-2xl -z-10"></div>
            </div>
          </div>

          <!-- 特性 2 -->
          <div class="grid md:grid-cols-2 gap-16 items-center">
            <div class="relative">
              <div class="aspect-video bg-gradient-to-br from-sky-100 to-blue-100 rounded-3xl border border-blue-200 p-8 shadow-lg shadow-sky-100">
                <div class="bg-white rounded-2xl p-4 h-full flex flex-col">
                  <div class="flex items-center gap-2 mb-4">
                    <div class="w-3 h-3 rounded-full bg-red-400"></div>
                    <div class="w-3 h-3 rounded-full bg-yellow-400"></div>
                    <div class="w-3 h-3 rounded-full bg-green-400"></div>
                  </div>
                  <div class="font-mono text-sm text-slate-800 flex-1">
                    <pre class="whitespace-pre-wrap">// 类型安全示例
interface User {
  name: string;
  age: number;
}

function greet(user: User) {
  return `Hello, ${user.name}!`;
}

// 编译期检查
greet({ name: "Alice" }); // 错误: 缺少 age 属性</pre>
                  </div>
                </div>
              </div>
              <div class="absolute -bottom-4 -left-4 w-24 h-24 bg-sky-200 rounded-2xl -z-10"></div>
            </div>
            <div>
              <div class="text-sky-600 text-sm font-semibold mb-4">⚡ 零开销</div>
              <h3 class="text-3xl md:text-4xl font-bold text-blue-800 mb-6">编译期优化</h3>
              <p class="text-lg text-slate-700 leading-relaxed mb-8">采用 Rust 语言开发，编译期生成最优代码，无运行时反射，确保极致的性能和安全性。</p>
              <div class="flex items-center gap-4 mb-6">
                <div class="w-12 h-12 bg-sky-100 rounded-xl flex items-center justify-center">
                  <span class="text-sky-800">🚀</span>
                </div>
                <div>
                  <h4 class="font-semibold text-blue-800">极致性能</h4>
                  <p class="text-sm text-slate-600">编译期优化，零运行时开销</p>
                </div>
              </div>
              <div class="flex items-center gap-4">
                <div class="w-12 h-12 bg-sky-100 rounded-xl flex items-center justify-center">
                  <span class="text-sky-800">🛡️</span>
                </div>
                <div>
                  <h4 class="font-semibold text-blue-800">类型安全</h4>
                  <p class="text-sm text-slate-600">编译期类型检查，减少运行时错误</p>
                </div>
              </div>
            </div>
          </div>

          <!-- 特性 3 -->
          <div class="grid md:grid-cols-2 gap-16 items-center">
            <div class="order-2 md:order-1">
              <div class="text-blue-600 text-sm font-semibold mb-4">🔧 模块化设计</div>
              <h3 class="text-3xl md:text-4xl font-bold text-blue-800 mb-6">跨文件复用</h3>
              <p class="text-lg text-slate-700 leading-relaxed mb-8">支持跨文件复用，提高代码可维护性和复用性，与现有 TypeScript 生态无缝集成。</p>
              <div class="flex items-center gap-4 mb-6">
                <div class="w-12 h-12 bg-blue-100 rounded-xl flex items-center justify-center">
                  <span class="text-blue-800">🔗</span>
                </div>
                <div>
                  <h4 class="font-semibold text-blue-800">模块化架构</h4>
                  <p class="text-sm text-slate-600">支持跨文件引用和复用</p>
                </div>
              </div>
              <div class="flex items-center gap-4">
                <div class="w-12 h-12 bg-blue-100 rounded-xl flex items-center justify-center">
                  <span class="text-blue-800">🔄</span>
                </div>
                <div>
                  <h4 class="font-semibold text-blue-800">生态集成</h4>
                  <p class="text-sm text-slate-600">与现有 TypeScript 工具链兼容</p>
                </div>
              </div>
            </div>
            <div class="order-1 md:order-2 relative">
              <div class="aspect-video bg-gradient-to-br from-blue-100 to-sky-100 rounded-3xl border border-blue-200 p-8 shadow-lg shadow-blue-100">
                <div class="bg-white rounded-2xl p-4 h-full flex flex-col">
                  <div class="flex items-center gap-2 mb-4">
                    <div class="w-3 h-3 rounded-full bg-red-400"></div>
                    <div class="w-3 h-3 rounded-full bg-yellow-400"></div>
                    <div class="w-3 h-3 rounded-full bg-green-400"></div>
                  </div>
                  <div class="font-mono text-sm text-slate-800 flex-1">
                    <pre class="whitespace-pre-wrap">// utils.ts
export function add(a: number, b: number): number {
  return a + b;
}

// app.ts
import { add } from './utils';

console.log(add(1, 2)); // 3</pre>
                  </div>
                </div>
              </div>
              <div class="absolute -bottom-4 -right-4 w-24 h-24 bg-blue-200 rounded-2xl -z-10"></div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 调用行动 -->
    <div class="py-24 bg-gradient-to-b from-white to-blue-50">
      <div class="max-w-4xl mx-auto px-6 lg:px-8 text-center">
        <h2 class="text-3xl md:text-4xl font-bold text-blue-800 mb-6">准备好体验 Rusty TypeScript 了吗？</h2>
        <p class="text-xl text-slate-700 mb-12">立即下载并开始使用，感受高性能 TypeScript 的魅力</p>
        <div class="flex flex-col sm:flex-row gap-6 justify-center items-center">
          <router-link 
            to="/install"
            class="group relative overflow-hidden px-10 py-4 rounded-2xl text-lg font-bold bg-gradient-to-r from-blue-600 via-sky-600 to-indigo-600 text-white shadow-lg shadow-blue-500/20 hover:shadow-blue-500/30 transition-all duration-300 hover:-translate-y-1 transform hover:scale-105"
          >
            <span class="relative z-10 flex items-center gap-3">
              立即下载
              <svg class="w-5 h-5 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"></path>
              </svg>
            </span>
          </router-link>
          <router-link 
            to="/playground"
            class="group relative overflow-hidden px-10 py-4 rounded-2xl text-lg font-bold bg-white text-blue-700 border border-blue-200 shadow-lg shadow-blue-100/30 hover:shadow-blue-200/50 transition-all duration-300 hover:-translate-y-1 transform hover:scale-105"
          >
            <span class="relative z-10 flex items-center gap-3">
              在线演示
              <svg class="w-5 h-5 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"></path>
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
              </svg>
            </span>
          </router-link>
        </div>
      </div>
    </div>


  </div>
</template>
