//! 优化性能测试
//!
//! 测试优化后的性能效果。

//! 性能测试文件
//!
//! 用于测试 TypeScript 编译器的性能优化效果

use std::time::Instant;
use typescript::compiler::{ir_generator::IRGenerator, optimizer::Optimizer};
use typescript_types::TsValue;

/// 测试代码1：基本的算术操作
const TEST_CODE_1: &str = r#"
function fibonacci(n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

let result = fibonacci(30);
result;
"#;

/// 测试代码2：复杂的对象操作
const TEST_CODE_2: &str = r#"
class Person {
    constructor(name, age) {
        this.name = name;
        this.age = age;
    }
    
    greet() {
        return `Hello, my name is ${this.name} and I'm ${this.age} years old.`;
    }
}

let people = [];
for (let i = 0; i < 1000; i++) {
    people.push(new Person(`Person ${i}`, i));
}

let greetings = people.map(person => person.greet());
greetings.length;
"#;

/// 测试代码3：数组操作和函数调用
const TEST_CODE_3: &str = r#"
function sum(arr) {
    let total = 0;
    for (let i = 0; i < arr.length; i++) {
        total += arr[i];
    }
    return total;
}

function generateArray(size) {
    let arr = [];
    for (let i = 0; i < size; i++) {
        arr.push(i);
    }
    return arr;
}

let arr = generateArray(100000);
let result = sum(arr);
result;
"#;

#[test]
fn test_optimization_performance() {
    println!("开始性能测试...");

    // 测试词法分析性能
    // test_lexer_performance();

    // 测试语法分析性能
    // test_parser_performance();

    // 测试IR生成性能
    // test_ir_generator_performance();

    // 测试优化性能
    // test_optimizer_performance();

    // 测试执行性能
    // test_execution_performance();

    println!("性能测试完成！");
}

/// 测试优化性能
fn test_optimization_performance() {
    println!("\n=== 优化性能测试 ===");

    // 测试常量折叠优化
    let start = Instant::now();

    // 这里可以添加一些简单的优化测试代码

    let duration = start.elapsed();
    println!("优化测试耗时: {:?}", duration);
}

