# 🏠 @nyar/typescript-homepage

> Rusty TypeScript Official Website & Playground 🦀🎨✨

[![Vue](https://img.shields.io/badge/Vue-3.4+-green?logo=vue.js)](https://vuejs.org)
[![Vite](https://img.shields.io/badge/Vite-5.0+-purple?logo=vite)](https://vitejs.dev)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.3+-blue?logo=typescript)](https://www.typescriptlang.org)
[![Tailwind](https://img.shields.io/badge/Tailwind-3.4+-06B6D4?logo=tailwindcss)](https://tailwindcss.com)

---

## 📋 Introduction

This is the official website and interactive playground for **Rusty TypeScript**, built with Vue 3 + Vite + TypeScript. It provides online code editing, real-time execution, and documentation browsing features.

### ✨ Core Features

| Feature | Description | Status |
|---------|-------------|--------|
| 🎮 **Playground** | Online TypeScript code editing and execution | ✅ Available |
| 📝 **Code Editor** | Monaco Editor integration | ✅ Available |
| 🎨 **Syntax Highlighting** | Shiki code highlighting | ✅ Available |
| 📚 **Documentation** | Markdown documentation rendering | ✅ Available |
| 🌓 **Theme Switching** | Dark/light mode | ✅ Available |
| 📱 **Responsive Design** | Mobile-friendly | ✅ Available |
| 🚀 **PWA Support** | Offline access | 🚧 Planned |

---

## 🚀 Quick Start

### Requirements

- 📦 [pnpm](https://pnpm.io/) 9.0+
- 🔧 [Node.js](https://nodejs.org/) 20+

### Installation

```bash
# Install dependencies
pnpm install

# Install Rusty TypeScript WASM module (for local testing)
cd ../typescript-wasi && pnpm build
cd ../homepage && pnpm link ../typescript-wasi
```

### Development Mode

```bash
# Start development server
pnpm dev

# Visit http://localhost:5173
```

### Production Build

```bash
# Build for production
pnpm build

# Preview production build
pnpm preview
```

---

## 🎮 Playground Features

### Code Editing

- 📝 **Monaco Editor**: Same editor as VS Code
- 🎨 **Syntax Highlighting**: Full TypeScript support
- 💡 **Intelligent Suggestions**: Code completion
- 🔄 **Auto-save**: Local code storage

### Code Execution

- ⚡ **WASM Execution**: Run directly in the browser
- 📊 **Performance Analysis**: Execution time and memory usage
- 🐛 **Error Messages**: Detailed error information
- 📤 **Code Sharing**: URL sharing feature

### Preset Examples

| Example | Description |
|---------|-------------|
| 🚀 **Hello World** | Basic syntax demonstration |
| 📐 **Fibonacci** | Recursive function example |
| 🔄 **Type Gymnastics** | Advanced type operations |
| 📦 **Module System** | Import/export examples |
| 🎯 **Decorators** | Decorator usage |
| 🧮 **Algorithm Implementation** | Common algorithms |

---

## 🎨 Theme System

### Built-in Themes

| Theme | Description |
|-------|-------------|
| 🌙 **Dark** | Dark theme (default) |
| ☀️ **Light** | Light theme |
| 🌅 **Sunset** | Sunset orange theme |
| 🌊 **Ocean** | Ocean blue theme |
| 🌲 **Forest** | Forest green theme |

---

## 🚀 Deployment

### Build

```bash
# Production build
pnpm build

# Output directory: dist/
```

### Static Hosting

```bash
# Vercel
vercel --prod

# Netlify
netlify deploy --prod --dir=dist

# GitHub Pages
pnpm build
gh-pages -d dist
```

---

## 📚 Dependencies

```
@nyar/typescript-homepage
├── vue                     # Framework
├── vue-router              # Routing
├── @monaco-editor/react    # Code editor
├── shiki                   # Syntax highlighting
├── marked                  # Markdown parsing
├── element-plus            # UI component library
├── @element-plus/icons-vue # Icons
├── tailwindcss             # CSS framework
├── @nyar/typescript-wasi   # WASM runtime (workspace)
└── vite                    # Build tool
```

---

## 🤝 Contributing

We welcome Issues and PRs! Please ensure:

1. ✅ Code passes `pnpm lint`
2. ✅ Code passes `pnpm typecheck`
3. ✅ Follows Vue 3 Composition API style
4. ✅ Includes necessary component documentation

---

## 📄 License

MIT License - See [LICENSE](../../license.md)

---

<div align="center">

**🦀 Discover the magic of Rusty TypeScript, starting here 🚀**

[🌐 Live Demo](https://rusty-typescript.dev) | [📖 Documentation](https://rusty-typescript.dev/docs) | [💻 GitHub](https://github.com/nyar-vm/rusty-typescript)

</div>