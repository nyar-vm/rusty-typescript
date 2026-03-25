use typescript::{create_runtime, run_script};

#[test]
fn test_jit_compilation() {
    // 创建 TypeScript 运行时
    let mut runtime = create_runtime();

    // 测试脚本：包含一个热点函数
    let script = r#"
    function hotFunction(x, y) {
        let result = 0;
        for (let i = 0; i < 1000; i++) {
            result += x * y + i;
        }
        return result;
    }
    
    // 多次调用函数，触发 JIT 编译
    console.log('Calling hotFunction...');
    for (let i = 0; i < 150; i++) {
        hotFunction(i, i + 1);
    }
    
    // 最后一次调用，应该使用 JIT 编译
    let result = hotFunction(100, 200);
    console.log('Final result:', result);
    result;
    "#;

    // 执行脚本
    match run_script(&mut runtime, script) {
        Ok(value) => println!("Script executed successfully: {}", value),
        Err(error) => panic!("Script execution failed: {:?}", error),
    }
}
