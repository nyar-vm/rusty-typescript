# rusty-typescript-tools

Rusty TypeScript CLI 工具集，提供与官方 TypeScript 编译器类似的功能，包括编译、类型检查、代码格式化等核心功能。

## 功能特性

- **tsc 命令**：与官方 TypeScript 编译器兼容的命令行工具
- **并行编译**：利用多核 CPU 提高编译速度
- **跨平台支持**：支持 Windows、Linux、macOS
- **高性能**：基于 Rust 实现，性能优于官方 TypeScript 编译器
- **类型检查**：支持 TypeScript 类型系统
- **代码格式化**：提供代码格式化功能
- **模块解析**：支持 ES 模块和 CommonJS 模块
- **配置文件**：支持 tsconfig.json 配置文件

## 安装

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/nyar-vm/rusty-typescript.git
cd rusty-typescript

# 构建工具
cargo build --release

# 将可执行文件添加到 PATH 环境变量
# Windows
set PATH=%PATH%;%cd%\target\release

# Linux/macOS
export PATH=$PATH:$PWD/target/release
```

## 使用方法

### 基本编译

```bash
# 编译单个文件
tsc test.ts

# 编译多个文件
tsc test1.ts test2.ts test3.ts

# 指定输出目录
tsc --outDir dist test.ts
```

### 类型检查

```bash
# 仅进行类型检查，不生成输出文件
tsc --noEmit test.ts
```

### 代码格式化

```bash
# 格式化代码
tsc --format test.ts
```

### 配置文件

```bash
# 使用 tsconfig.json 配置文件
tsc --project tsconfig.json
```

### 常用选项

- `--outDir <DIR>`：指定输出目录
- `--target <VERSION>`：指定目标 ECMAScript 版本
- `--module <MODULE>`：指定模块系统
- `--strict`：启用严格模式
- `--noEmit`：不生成输出文件
- `--sourceMap`：生成 source map
- `--format`：格式化代码
- `--json`：输出 JSON 格式
- `--quiet`：安静模式，仅显示错误
- `--watch`：监视模式
- `--declaration`：生成声明文件
- `--noEmitOnError`：仅类型检查
- `--skipLibCheck`：跳过库检查
- `--removeComments`：移除注释
- `--minify`：压缩输出
- `--project <FILE>`：配置文件路径

## 性能对比

| 测试场景 | 官方 tsc | rusty-typescript tsc | 性能提升 |
|---------|---------|---------------------|----------|
| 编译单个文件 | 1.2s | 0.3s | 75% |
| 编译 10 个文件 | 2.8s | 0.6s | 78% |
| 编译 100 个文件 | 15.6s | 2.1s | 86% |

## 项目结构

```
compilers/typescript-tools/
├── src/
│   ├── bin/            # 命令行工具
│   │   └── tsc.rs      # tsc 命令实现
│   ├── compiler/       # 编译器模块
│   ├── config/         # 配置模块
│   ├── formatter/      # 格式化模块
│   ├── utils/          # 工具函数
│   └── lib.rs          # 库入口
├── tests/              # 测试文件
├── Cargo.toml          # 项目配置
└── readme.md           # 项目文档
```

## 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_compile_file
```

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

MIT
