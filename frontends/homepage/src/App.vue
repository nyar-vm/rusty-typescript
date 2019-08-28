<template>
  <div class="min-h-screen bg-gradient-to-br from-blue-50 to-sky-50 text-slate-800 relative overflow-hidden flex flex-col">
    <div class="fixed inset-0 z-0">
      <div class="absolute top-[-10%] left-[-10%] w-[500px] h-[500px] bg-blue-400/10 rounded-full blur-[120px]"></div>
      <div class="absolute top-[20%] right-[-5%] w-[600px] h-[600px] bg-sky-400/10 rounded-full blur-[100px]"></div>
      <div class="absolute bottom-[-10%] left-[30%] w-[550px] h-[550px] bg-indigo-400/10 rounded-full blur-[110px]"></div>
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
            <button 
              @click="currentSection = 'home'"
              :class="[
                'px-5 py-2.5 rounded-xl text-sm font-semibold transition-all duration-300',
                currentSection === 'home' 
                  ? 'bg-blue-500/10 text-blue-800 border border-blue-200' 
                  : 'text-slate-600 hover:text-blue-800 hover:bg-blue-50'
              ]"
            >
              首页
            </button>
            <button 
              @click="currentSection = 'download'"
              :class="[
                'px-5 py-2.5 rounded-xl text-sm font-semibold transition-all duration-300',
                currentSection === 'download' 
                  ? 'bg-blue-500/10 text-blue-800 border border-blue-200' 
                  : 'text-slate-600 hover:text-blue-800 hover:bg-blue-50'
              ]"
            >
              下载
            </button>
            <button 
              @click="currentSection = 'docs'"
              :class="[
                'px-5 py-2.5 rounded-xl text-sm font-semibold transition-all duration-300',
                currentSection === 'docs' 
                  ? 'bg-blue-500/10 text-blue-800 border border-blue-200' 
                  : 'text-slate-600 hover:text-blue-800 hover:bg-blue-50'
              ]"
            >
              文档
            </button>
            <button 
              @click="currentSection = 'playground'"
              :class="[
                'px-5 py-2.5 rounded-xl text-sm font-semibold transition-all duration-300',
                currentSection === 'playground' 
                  ? 'bg-blue-500/10 text-blue-800 border border-blue-200' 
                  : 'text-slate-600 hover:text-blue-800 hover:bg-blue-50'
              ]"
            >
              Playground
            </button>
            <button 
              @click="currentSection = 'home'"
              :class="[
                'px-5 py-2.5 rounded-xl text-sm font-semibold transition-all duration-300',
                'text-slate-600 hover:text-blue-800 hover:bg-blue-50'
              ]"
            >
              xRPC
            </button>
          </nav>

          <div class="flex items-center gap-4">
            <button class="hidden md:flex items-center gap-2 px-5 py-2.5 rounded-xl text-sm font-semibold text-slate-700 hover:text-blue-800 transition-all duration-300">
              <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24"><path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/></svg>
              GitHub
            </button>
          </div>
        </div>
      </div>
    </header>

    <main class="relative z-10 flex-1">
      <section v-if="currentSection === 'home'" class="pt-40 pb-32">
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
            
            <div class="max-w-3xl mx-auto mb-16">
              <div class="bg-white rounded-3xl border border-blue-200 p-8 shadow-lg shadow-blue-100/50">
                <h3 class="text-2xl font-bold text-blue-800 mb-4">什么是 Rusty TypeScript？</h3>
                <p class="text-slate-700 mb-6">Rusty TypeScript 是一个用 Rust 语言实现的 TypeScript 编译器和运行时，旨在提供高性能、类型安全的 TypeScript 代码执行环境。</p>
                <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                  <div class="bg-blue-50 rounded-2xl p-6 border border-blue-100">
                    <h4 class="font-semibold text-blue-800 mb-2">高性能</h4>
                    <p class="text-sm text-slate-600">基于 Rust 实现，编译期优化，提供极致的性能表现</p>
                  </div>
                  <div class="bg-blue-50 rounded-2xl p-6 border border-blue-100">
                    <h4 class="font-semibold text-blue-800 mb-2">类型安全</h4>
                    <p class="text-sm text-slate-600">强类型设计，编译期检查，减少运行时错误</p>
                  </div>
                  <div class="bg-blue-50 rounded-2xl p-6 border border-blue-100">
                    <h4 class="font-semibold text-blue-800 mb-2">易于集成</h4>
                    <p class="text-sm text-slate-600">与现有 TypeScript 生态无缝集成，支持标准 TypeScript 语法</p>
                  </div>
                </div>
              </div>
            </div>
            
            <div class="flex flex-col sm:flex-row gap-6 justify-center items-center">
              <button 
                @click="currentSection = 'download'"
                class="group relative overflow-hidden px-10 py-4 rounded-2xl text-lg font-bold bg-gradient-to-r from-blue-600 via-sky-600 to-indigo-600 text-white shadow-lg shadow-blue-500/20 hover:shadow-blue-500/30 transition-all duration-300 hover:-translate-y-1 transform hover:scale-105"
              >
                <span class="relative z-10 flex items-center gap-3">
                  立即下载
                  <svg class="w-5 h-5 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"></path>
                  </svg>
                </span>
              </button>
              <button 
                @click="currentSection = 'docs'"
                class="group px-10 py-4 rounded-2xl text-lg font-bold bg-white border border-blue-300 text-blue-800 hover:bg-blue-50 transition-all duration-300 hover:-translate-y-1 transform hover:scale-105"
              >
                <span class="flex items-center gap-3">
                  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"></path>
                  </svg>
                  查看文档
                </span>
              </button>
            </div>
          </div>

          <div class="space-y-24">
            <div class="grid md:grid-cols-2 gap-16 items-center">
              <div class="order-2 md:order-1">
                <div class="text-blue-600 text-sm font-semibold mb-4">📊 高性能编译</div>
                <h3 class="text-3xl md:text-4xl font-bold text-blue-800 mb-6">Rust 驱动的编译器</h3>
                <p class="text-lg text-slate-700 leading-relaxed mb-8">Rusty TypeScript 使用 Rust 语言实现编译器和运行时，编译期优化生成高效代码，提供极致的性能表现。</p>
                <div class="flex items-center gap-4">
                  <div class="w-12 h-12 bg-blue-100 rounded-xl flex items-center justify-center">
                    <span class="text-blue-800">📝</span>
                  </div>
                  <div>
                    <h4 class="font-semibold text-blue-800">标准 TypeScript 语法</h4>
                    <p class="text-sm text-slate-600">支持标准 TypeScript 语法，与现有 TypeScript 代码兼容</p>
                  </div>
                </div>
              </div>
              <div class="order-1 md:order-2 relative">
                <div class="aspect-video bg-gradient-to-br from-blue-100 to-sky-100 rounded-3xl border border-blue-200 flex items-center justify-center shadow-lg shadow-blue-100">
                  <div class="text-6xl">📊</div>
                </div>
                <div class="absolute -bottom-4 -right-4 w-24 h-24 bg-blue-200 rounded-2xl -z-10"></div>
              </div>
            </div>

            <div class="grid md:grid-cols-2 gap-16 items-center">
              <div class="relative">
                <div class="aspect-video bg-gradient-to-br from-sky-100 to-blue-100 rounded-3xl border border-blue-200 flex items-center justify-center shadow-lg shadow-sky-100">
                  <div class="text-6xl">⚡</div>
                </div>
                <div class="absolute -bottom-4 -left-4 w-24 h-24 bg-sky-200 rounded-2xl -z-10"></div>
              </div>
              <div>
                <div class="text-sky-600 text-sm font-semibold mb-4">⚡ 零开销</div>
                <h3 class="text-3xl md:text-4xl font-bold text-blue-800 mb-6">编译期优化</h3>
                <p class="text-lg text-slate-700 leading-relaxed mb-8">采用 Rust 语言开发，编译期生成最优代码，无运行时反射，确保极致的性能和安全性。</p>
                <div class="flex items-center gap-4">
                  <div class="w-12 h-12 bg-sky-100 rounded-xl flex items-center justify-center">
                    <span class="text-sky-800">🚀</span>
                  </div>
                  <div>
                    <h4 class="font-semibold text-blue-800">极致性能</h4>
                    <p class="text-sm text-slate-600">编译期优化，零运行时开销</p>
                  </div>
                </div>
              </div>
            </div>

            <div class="grid md:grid-cols-2 gap-16 items-center">
              <div class="order-2 md:order-1">
                <div class="text-blue-600 text-sm font-semibold mb-4">🔧 模块化设计</div>
                <h3 class="text-3xl md:text-4xl font-bold text-blue-800 mb-6">跨文件复用</h3>
                <p class="text-lg text-slate-700 leading-relaxed mb-8">支持跨文件复用，提高代码可维护性和复用性，与现有 TypeScript 生态无缝集成。</p>
                <div class="flex items-center gap-4">
                  <div class="w-12 h-12 bg-blue-100 rounded-xl flex items-center justify-center">
                    <span class="text-blue-800">🔗</span>
                  </div>
                  <div>
                    <h4 class="font-semibold text-blue-800">模块化架构</h4>
                    <p class="text-sm text-slate-600">支持跨文件引用和复用</p>
                  </div>
                </div>
              </div>
              <div class="order-1 md:order-2 relative">
                <div class="aspect-video bg-gradient-to-br from-blue-100 to-sky-100 rounded-3xl border border-blue-200 flex items-center justify-center shadow-lg shadow-blue-100">
                  <div class="text-6xl">🔧</div>
                </div>
                <div class="absolute -bottom-4 -right-4 w-24 h-24 bg-blue-200 rounded-2xl -z-10"></div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <section v-if="currentSection === 'download'" class="pt-40 pb-32">
        <div class="max-w-7xl mx-auto px-6 lg:px-8">
          <div class="text-center mb-20">
            <h1 class="text-4xl md:text-5xl font-bold text-blue-800 mb-6">选择适合您的版本</h1>
            <p class="text-xl text-slate-700">Rusty TypeScript 支持所有主流操作系统</p>
          </div>
          
          <div class="space-y-6 max-w-3xl mx-auto">
            <button 
              class="group w-full flex items-center gap-6 p-8 bg-blue-100 border border-blue-200 rounded-3xl hover:bg-blue-200 hover:border-blue-300 transition-all duration-300"
            >
              <div class="w-16 h-16 bg-gradient-to-br from-blue-100 to-sky-100 rounded-2xl flex items-center justify-center border border-blue-300 group-hover:scale-110 transition-transform">
                <span class="text-4xl">🪟</span>
              </div>
              <div class="flex-1 text-left">
                <h3 class="text-2xl font-bold text-blue-800">Windows</h3>
                <p class="text-slate-600">Windows 10/11</p>
              </div>
              <div class="text-right">
                <span class="inline-flex items-center gap-2 px-6 py-3 bg-gradient-to-r from-blue-600 to-sky-600 rounded-xl font-bold text-white group-hover:shadow-lg group-hover:shadow-blue-500/30 transition-all">
                  下载
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"></path>
                  </svg>
                </span>
              </div>
            </button>
            
            <button 
              class="group w-full flex items-center gap-6 p-8 bg-blue-100 border border-blue-200 rounded-3xl hover:bg-blue-200 hover:border-blue-300 transition-all duration-300"
            >
              <div class="w-16 h-16 bg-gradient-to-br from-sky-100 to-blue-100 rounded-2xl flex items-center justify-center border border-blue-300 group-hover:scale-110 transition-transform">
                <span class="text-4xl">🍎</span>
              </div>
              <div class="flex-1 text-left">
                <h3 class="text-2xl font-bold text-blue-800">macOS</h3>
                <p class="text-slate-600">macOS 11+</p>
              </div>
              <div class="text-right">
                <span class="inline-flex items-center gap-2 px-6 py-3 bg-gradient-to-r from-sky-600 to-blue-600 rounded-xl font-bold text-white group-hover:shadow-lg group-hover:shadow-blue-500/30 transition-all">
                  下载
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"></path>
                  </svg>
                </span>
              </div>
            </button>
            
            <button 
              class="group w-full flex items-center gap-6 p-8 bg-blue-100 border border-blue-200 rounded-3xl hover:bg-blue-200 hover:border-blue-300 transition-all duration-300"
            >
              <div class="w-16 h-16 bg-gradient-to-br from-indigo-100 to-blue-100 rounded-2xl flex items-center justify-center border border-blue-300 group-hover:scale-110 transition-transform">
                <span class="text-4xl">🐧</span>
              </div>
              <div class="flex-1 text-left">
                <h3 class="text-2xl font-bold text-blue-800">Linux</h3>
                <p class="text-slate-600">Ubuntu, Fedora, etc.</p>
              </div>
              <div class="text-right">
                <span class="inline-flex items-center gap-2 px-6 py-3 bg-gradient-to-r from-indigo-600 to-blue-600 rounded-xl font-bold text-white group-hover:shadow-lg group-hover:shadow-blue-500/30 transition-all">
                  下载
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"></path>
                  </svg>
                </span>
              </div>
            </button>
            
            <button 
              class="group w-full flex items-center gap-6 p-8 bg-blue-100 border border-blue-200 rounded-3xl hover:bg-blue-200 hover:border-blue-300 transition-all duration-300"
            >
              <div class="w-16 h-16 bg-gradient-to-br from-blue-100 to-indigo-100 rounded-2xl flex items-center justify-center border border-blue-300 group-hover:scale-110 transition-transform">
                <span class="text-4xl">📦</span>
              </div>
              <div class="flex-1 text-left">
                <h3 class="text-2xl font-bold text-blue-800">源码</h3>
                <p class="text-slate-600">GitHub 仓库</p>
              </div>
              <div class="text-right">
                <span class="inline-flex items-center gap-2 px-6 py-3 bg-blue-700 border border-blue-600 rounded-xl font-bold text-white hover:bg-blue-800 transition-all">
                  查看
                  <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24"><path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/></svg>
                </span>
              </div>
            </button>
          </div>
        </div>
      </section>

      <section v-if="currentSection === 'docs'" class="pt-32 pb-20">
        <div class="max-w-7xl mx-auto px-6 lg:px-8">
          <div class="flex flex-col lg:flex-row gap-8">
            <aside class="lg:w-80 flex-shrink-0">
              <div class="sticky top-28 rounded-3xl bg-blue-100 border border-blue-200 p-6">
                <h2 class="text-xl font-bold text-blue-800 mb-6">文档导航</h2>
                <div class="text-slate-600 text-sm">
                  请参考 TypeScript 官方文档
                </div>
              </div>
            </aside>

            <main class="flex-1">
              <div class="rounded-3xl bg-blue-100 border border-blue-200 p-10">
                <div class="text-slate-600 text-center py-16">
                  <p class="text-lg">请参考 TypeScript 官方文档</p>
                </div>
              </div>
            </main>
          </div>
        </div>
      </section>

      <section v-if="currentSection === 'playground'" class="pt-40 pb-32">
        <div class="max-w-7xl mx-auto px-6 lg:px-8">
          <div class="text-center mb-16">
            <h1 class="text-4xl md:text-5xl font-bold text-blue-800 mb-6">TypeScript Playground</h1>
            <p class="text-xl text-slate-700">在线测试和运行 TypeScript 代码</p>
          </div>
          
          <TypeScriptPlayground />
        </div>
      </section>
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
              <li><a href="#" class="text-slate-600 hover:text-blue-800 transition-colors text-sm">文档</a></li>
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
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import TypeScriptPlayground from "@/components/TypeScriptPlayground.vue";

const currentSection = ref<"home" | "download" | "docs" | "playground">("home");

onMounted(() => {
    console.log("App mounted");
});
</script>
