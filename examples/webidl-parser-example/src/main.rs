/// WebIDL 解析器示例
///
/// 这个示例展示了如何使用 WebIDL 解析器来解析 WebIDL 文件并将其转换为 TypeScript 类型定义
use std::fs;
use typescript_webidl::{converter::convert_to_typescript, parser::simple_parse};

fn main() {
    println!("WebIDL 解析器示例");
    println!("==================");

    // 示例 1: 解析简单的 WebIDL 字符串
    println!("\n示例 1: 解析简单的 WebIDL 字符串");
    let simple_webidl = r#"
        interface Point {
            attribute double x;
            attribute double y;
            double distance();
        }
    "#;

    match simple_parse(simple_webidl) {
        Ok(root) => {
            let typescript = convert_to_typescript(&root);
            println!("WebIDL:");
            println!("{}", simple_webidl);
            println!("\n生成的 TypeScript:");
            println!("{}", typescript);
        }
        Err(error) => {
            println!("解析错误: {}", error);
        }
    }

    // 示例 2: 解析包含复杂类型的 WebIDL
    println!("\n示例 2: 解析包含复杂类型的 WebIDL");
    let complex_webidl = r#"
        dictionary User {
            DOMString name;
            unsigned long age;
            (DOMString or long)? nickname;
        };
        
        enum Color {
            "red",
            "green",
            "blue"
        };
        
        interface Calculator {
            static long add(long a, long b);
            long multiply(long a, long b);
        }
    "#;

    match simple_parse(complex_webidl) {
        Ok(root) => {
            let typescript = convert_to_typescript(&root);
            println!("WebIDL:");
            println!("{}", complex_webidl);
            println!("\n生成的 TypeScript:");
            println!("{}", typescript);
        }
        Err(error) => {
            println!("解析错误: {}", error);
        }
    }

    // 示例 3: 从文件读取 WebIDL
    println!("\n示例 3: 从文件读取 WebIDL");
    let webidl_file = "examples/webidl-parser-example/src/sample.idl";

    match fs::read_to_string(webidl_file) {
        Ok(webidl_content) => {
            match simple_parse(&webidl_content) {
                Ok(root) => {
                    let typescript = convert_to_typescript(&root);
                    println!("从文件读取 WebIDL:");
                    println!("{}", webidl_content);
                    println!("\n生成的 TypeScript:");
                    println!("{}", typescript);

                    // 保存生成的 TypeScript 到文件
                    let output_file = "examples/webidl-parser-example/src/sample.d.ts";
                    match fs::write(output_file, typescript) {
                        Ok(_) => println!("\nTypeScript 类型定义已保存到: {}", output_file),
                        Err(e) => println!("\n保存文件失败: {}", e),
                    }
                }
                Err(error) => {
                    println!("解析错误: {}", error);
                }
            }
        }
        Err(e) => {
            println!("读取文件失败: {}", e);
        }
    }

    // 示例 4: 错误处理
    println!("\n示例 4: 错误处理");
    let invalid_webidl = r#"
        interface Invalid {
            attribute string name  // 缺少分号
            void method();
        }
    "#;

    match simple_parse(invalid_webidl) {
        Ok(_) => {
            println!("解析成功 (意外!)");
        }
        Err(error) => {
            println!("解析错误 (预期行为): {}", error);
        }
    }

    println!("\n示例完成!");
}
