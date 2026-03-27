//! 执行性能测试
//!
//! 测试脚本执行性能。

use std::{fs::File, io::Read};
use typescript::create_runtime;

#[test]
fn test_execution_performance() {
    // 读取性能测试脚本
    let mut file = File::open("../../test-local-variable-performance.ts").expect("无法打开测试脚本");
    let mut script = String::new();
    file.read_to_string(&mut script).expect("无法读取测试脚本");

    // 创建 TypeScript 运行时
    let mut runtime = create_runtime();

    // 执行测试脚本
    match runtime.execute_script(&script) {
        Ok(result) => println!("测试脚本执行成功: {:?}", result),
        Err(error) => println!("测试脚本执行失败: {:?}", error),
    }
}

