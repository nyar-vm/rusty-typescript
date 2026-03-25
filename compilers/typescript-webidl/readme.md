# TypeScript WebIDL 编译器

TypeScript WebIDL 编译器是一个将 WebIDL 转换为 TypeScript 类型定义的工具，基于 Oak IDL 库构建。

## 功能特性

- **WebIDL 解析**：使用 Oak IDL 库解析 WebIDL 字符串
- **TypeScript 转换**：将解析后的 WebIDL AST 转换为 TypeScript 类型定义
- **错误处理**：提供详细的错误信息
- **文件支持**：从文件读取并解析 WebIDL

## 快速开始

### 安装

在 `Cargo.toml` 文件中添加依赖：

```toml
dependencies = {
    typescript-webidl = { path = "path/to/typescript-webidl" }
}
```

### 使用示例

```rust
use typescript_webidl::{parse, convert_to_typescript};

fn main() {
    let webidl = r#"
        interface TestInterface {
            void test();
        };
    "#;
    
    match parse(webidl) {
        Ok(root) => {
            let typescript = convert_to_typescript(&root);
            println!("{}", typescript);
        }
        Err(error) => {
            println!("Error: {}", error);
        }
    }
}
```

## 项目结构

- `src/lib.rs`：主入口文件，包含解析和转换函数
- `src/converter/`：WebIDL 到 TypeScript 的转换模块
- `src/types/`：类型定义模块
- `src/type_checker/`：类型检查模块
- `tests/`：测试文件

## 技术依赖

- **Oak IDL**：用于 WebIDL 解析
- **Serde**：用于序列化和反序列化
- **Regex**：用于后备 WebIDL 解析

## 注意事项

- 由于 Oak IDL 库的 `parse` 函数可能返回空的 `items` 向量，项目添加了一个基于正则表达式的后备解析器
- 后备解析器支持基本的 WebIDL 语法，包括接口、操作和参数
- 对于复杂的 WebIDL 语法，建议使用标准的 WebIDL 解析器

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

MIT
