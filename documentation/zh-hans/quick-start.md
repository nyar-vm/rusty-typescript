---
title: 快速开始
order: 2
---

# 快速开始

本指南将帮助您快速上手 Rusty TypeScript，包括安装、基本使用和示例。

## 安装

### 系统要求

- Windows 10/11
- macOS 11+
- Linux (Ubuntu 20.04+, Fedora 32+)

### 安装步骤

1. **下载安装包**

   从 [Rusty TypeScript 官网](https://rusty-typescript.org) 下载适合您操作系统的安装包。

2. **安装**

   - **Windows**：运行安装程序，按照提示完成安装
   - **macOS**：打开 `.dmg` 文件，将 Rusty TypeScript 拖入 Applications 文件夹
   - **Linux**：解压 tarball 并运行安装脚本

3. **验证安装**

   打开终端并运行：

   ```bash
   rusty-ts --version
   ```

   如果安装成功，您将看到版本信息。

## 基本使用

### 编译 TypeScript 文件

创建一个简单的 TypeScript 文件 `hello.ts`：

```typescript
function sayHello(name: string): string {
    return `Hello, ${name}!`;
}

console.log(sayHello("Rusty TypeScript"));
```

使用 Rusty TypeScript 编译并运行：

```bash
rusty-ts hello.ts
```

### 使用 REPL

Rusty TypeScript 提供了交互式 REPL 环境：

```bash
rusty-ts
```

在 REPL 中，您可以直接输入 TypeScript 代码并执行：

```typescript
> const x = 10;
> console.log(x);
10
> function add(a: number, b: number): number { return a + b; }
> add(5, 3)
8
```

## 项目配置

对于大型项目，您可以使用 `rusty-ts.config.json` 文件进行配置：

```json
{
  "compilerOptions": {
    "target": "es2020",
    "module": "commonjs",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist"]
}
```

## 示例项目

### 简单的 Web 服务器

```typescript
import { createServer } from 'http';

const server = createServer((req, res) => {
    res.statusCode = 200;
    res.setHeader('Content-Type', 'text/plain');
    res.end('Hello from Rusty TypeScript!\n');
});

server.listen(3000, 'localhost', () => {
    console.log('Server running at http://localhost:3000/');
});
```

### 数据处理

```typescript
function processData(data: number[]): number[] {
    return data
        .filter(x => x > 0)
        .map(x => x * 2)
        .sort((a, b) => a - b);
}

const input = [-1, 5, 3, -2, 8, 0, 4];
const result = processData(input);
console.log('Processed data:', result);
```

## 下一步

- 查看 [功能特性](./features.md) 了解 Rusty TypeScript 的核心功能
- 阅读 [架构设计](./architecture.md) 了解内部工作原理
- 探索 [API 文档](./api.md) 了解可用的 API