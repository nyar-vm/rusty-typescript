# @nyar/typescript

> Rusty TypeScript WASI Frontend Package 🦀🌐📦

[![npm](https://img.shields.io/badge/npm-package-blue?logo=npm)](https://npmjs.com)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.3+-blue?logo=typescript)](https://www.typescriptlang.org)
[![WASI](https://img.shields.io/badge/WASI-Support-green)](https://wasi.dev/)

---

## 📋 Introduction

This is the WASI frontend package for **Rusty TypeScript**, providing the ability to load, initialize, and use the Rusty TypeScript WASM module in both browser and Node.js environments.

### ✨ Core Features

| Feature | Description | Status |
|---------|-------------|--------|
| 🌐 **WASM Loading** | Automatic loading and initialization of WASM module | ✅ Available |
| 🔧 **TypeScript API** | Complete type definition support | ✅ Available |
| 🖥️ **Multi-Environment** | Supports both browser and Node.js | ✅ Available |
| 🎮 **Playground** | Interactive code execution | 🚧 Planned |
| 📦 **Modular** | ESM / CJS dual format support | ✅ Available |

---

## 🚀 Quick Start

### Installation

```bash
# Using pnpm (recommended)
pnpm add @nyar/typescript

# Using npm
npm install @nyar/typescript

# Using yarn
yarn add @nyar/typescript
```

### Basic Usage

```typescript
import { RustyTypeScript } from '@nyar/typescript';

async function main() {
  // 🚀 Initialize Rusty TypeScript
  const rts = await RustyTypeScript.init();
  
  // 📝 Execute TypeScript code
  const result = await rts.execute(`
    const greeting: string = "Hello from WASM! 🦀";
    console.log(greeting);
    greeting
  `);
  
  console.log('✅ Execution result:', result);
}

main().catch(console.error);
```

### Browser Environment

```html
<!DOCTYPE html>
<html>
<head>
  <title>Rusty TypeScript Playground</title>
</head>
<body>
  <div id="output"></div>
  <script type="module">
    import { RustyTypeScript } from '@nyar/typescript';
    
    async function init() {
      const output = document.getElementById('output');
      
      try {
        // 🌐 Initialize in browser
        const rts = await RustyTypeScript.init({
          wasmUrl: './typescript.wasm'
        });
        
        // Execute code
        const result = await rts.execute(`
          const fib = (n: number): number => {
            if (n <= 1) return n;
            return fib(n - 1) + fib(n - 2);
          };
          fib(10)
        `);
        
        output.innerHTML = `🎉 Result: ${result}`;
      } catch (err) {
        output.innerHTML = `❌ Error: ${err.message}`;
      }
    }
    
    init();
  </script>
</body>
</html>
```

### Node.js Environment

```typescript
import { RustyTypeScript } from '@nyar/typescript';
import { readFileSync } from 'fs';

async function runScript(filePath: string) {
  // 🖥️ Initialize in Node.js
  const rts = await RustyTypeScript.init({
    wasmModule: readFileSync('./typescript.wasm')
  });
  
  // Read and execute file
  const code = readFileSync(filePath, 'utf-8');
  const result = await rts.execute(code);
  
  console.log('✅ Execution completed:', result);
  return result;
}

runScript('./script.ts');
```

---

## 💡 Usage Examples

### Code Editor Integration

```typescript
import { RustyTypeScript } from '@nyar/typescript';
import MonacoEditor from '@monaco-editor/react';

function Playground() {
  const [output, setOutput] = useState('');
  const [rts, setRts] = useState<RustyTypeScript | null>(null);
  
  useEffect(() => {
    // Initialize
    RustyTypeScript.init().then(setRts);
    
    return () => {
      rts?.dispose();
    };
  }, []);
  
  const runCode = async (code: string) => {
    if (!rts) return;
    
    try {
      const result = await rts.execute(code);
      setOutput(JSON.stringify(result.value, null, 2));
    } catch (err) {
      setOutput(`❌ Error: ${err.message}`);
    }
  };
  
  return (
    <div>
      <MonacoEditor
        language="typescript"
        onChange={(value) => runCode(value || '')}
      />
      <pre>{output}</pre>
    </div>
  );
}
```

### Custom Console

```typescript
import { RustyTypeScript } from '@nyar/typescript';

const customConsole = {
  log: (...args: unknown[]) => {
    // Send to logging service
    logService.info(args.join(' '));
  },
  error: (...args: unknown[]) => {
    logService.error(args.join(' '));
  },
  warn: (...args: unknown[]) => {
    logService.warn(args.join(' '));
  }
};

const rts = await RustyTypeScript.init({
  console: customConsole
});
```

### Error Handling

```typescript
import { RustyTypeScript, RustyTypeScriptError } from '@nyar/typescript';

async function safeExecute(code: string) {
  const rts = await RustyTypeScript.init();
  
  try {
    return await rts.execute(code);
  } catch (err) {
    if (err instanceof RustyTypeScriptError) {
      switch (err.type) {
        case 'syntax':
          console.error(`Syntax error (line ${err.location?.line}):`, err.message);
          break;
        case 'type':
          console.error('Type error:', err.message);
          break;
        case 'runtime':
          console.error('Runtime error:', err.message);
          break;
      }
    }
    throw err;
  }
}
```

---

## 🔧 Configuration

### Vite Configuration

```typescript
// vite.config.ts
import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';

export default defineConfig({
  plugins: [wasm()],
  optimizeDeps: {
    exclude: ['@nyar/typescript']
  },
  server: {
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp'
    }
  }
});
```

---

## 🧪 Development

```bash
# Install dependencies
pnpm install

# Development mode
pnpm dev

# Build
pnpm build

# Test
pnpm test

# Type check
pnpm typecheck
```

---

## 📦 Dependencies

| Dependency | Description | Type |
|------------|-------------|------|
| (no runtime dependencies) | - | - |
| typescript | Type checking | dev |
| vite | Build tool | dev |
| vitest | Test framework | dev |
| @types/node | Node.js types | dev |

---

## 🤝 Contributing

We welcome Issues and PRs! Please ensure:

1. ✅ Code passes `pnpm lint`
2. ✅ Code passes `pnpm typecheck`
3. ✅ All tests pass with `pnpm test`
4. ✅ Both browser and Node.js environments are tested

---

## 📄 License

MIT License - See [LICENSE](../../license.md)

---

<div align="center">

**🦀 Run Rust-compiled TypeScript in your browser for ultimate performance 🚀**

</div>