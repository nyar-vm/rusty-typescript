<template>
  <div class="pt-40 pb-32">
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
  </div>
</template>
