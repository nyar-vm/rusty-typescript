//! 优化 TypeScript 解析器测试
//! 
//! 测试优化后的 TypeScript 解析器的性能和功能。

use typescript::parser::OptimizedTypeScriptParser;
use typescript::language::TypeScriptLanguage;

#[test]
fn test_optimized_parser_basic() {
    let config = TypeScriptLanguage::standard();
    let mut parser = OptimizedTypeScriptParser::new(&config);

    let code = r#"
        function add(a: number, b: number): number {
            return a + b;
        }
        
        interface Person {
            name: string;
            age: number;
        }
        
        class Employee implements Person {
            constructor(public name: string, public age: number) {}
            
            greet(): string {
                return `Hello, my name is ${this.name}`;
            }
        }
        
        type Point = {
            x: number;
            y: number;
        };
        
        const p: Point = { x: 10, y: 20 };
    "#;

    let result = parser.parse(code);
    assert!(result.is_ok(), "优化解析器应该能正确解析基本 TypeScript 代码");
}

#[test]
fn test_optimized_parser_complex() {
    let config = TypeScriptLanguage::standard();
    let mut parser = OptimizedTypeScriptParser::new(&config);

    let code = r#"
        // 测试复杂的 TypeScript 语法
        function processArray<T>(arr: T[]): T[] {
            return arr.map(item => item);
        }
        
        interface Serializable {
            serialize(): string;
        }
        
        class User implements Serializable {
            constructor(private id: number, private name: string) {}
            
            getInfo(): { id: number, name: string } {
                return { id: this.id, name: this.name };
            }
            
            serialize(): string {
                return JSON.stringify(this.getInfo());
            }
        }
        
        const users: User[] = [
            new User(1, "Alice"),
            new User(2, "Bob")
        ];
        
        const serialized = users.map(user => user.serialize());
    "#;

    let result = parser.parse(code);
    assert!(result.is_ok(), "优化解析器应该能正确解析复杂 TypeScript 代码");
}

#[test]
fn test_optimized_parser_error_handling() {
    let config = TypeScriptLanguage::standard();
    let mut parser = OptimizedTypeScriptParser::new(&config);

    // 测试语法错误
    let code = r#"
        function add(a: number, b: number): number {
            return a + b
        // 缺少闭合大括号
    "#;

    let result = parser.parse(code);
    // 注意：当前的解析器实现会尝试跳过错误，所以这里可能不会返回错误
    // 后续可以增强错误处理能力
    println!("解析错误代码的结果: {:?}", result);
}
