use typescript::TypeScript;

#[test]
fn test_basic_types() {
    let mut ts = TypeScript::new();

    // 测试基本类型
    let result = ts.execute_script("let x = 10; let y = 'hello'; console.log(x, y); x + y").unwrap();
    println!("Test basic types result: {:?}", result);
}

#[test]
fn test_functions() {
    let mut ts = TypeScript::new();

    // 测试函数
    let result = ts.execute_script("function add(a, b) { return a + b; } add(5, 3)").unwrap();
    println!("Test functions result: {:?}", result);
}

#[test]
fn test_objects() {
    let mut ts = TypeScript::new();

    // 测试对象
    let result = ts.execute_script("let obj = { name: 'test', value: 42 }; obj.name").unwrap();
    println!("Test objects result: {:?}", result);
}

#[test]
fn test_arrays() {
    let mut ts = TypeScript::new();

    // 测试数组
    let result = ts.execute_script("let arr = [1, 2, 3]; arr[1]").unwrap();
    println!("Test arrays result: {:?}", result);
}

#[test]
fn test_control_flow() {
    let mut ts = TypeScript::new();

    // 测试控制流
    let result = ts.execute_script("let x = 10; if (x > 5) { 'greater' } else { 'less' }").unwrap();
    println!("Test control flow result: {:?}", result);
}

#[test]
fn test_loops() {
    let mut ts = TypeScript::new();

    // 测试循环
    let result = ts.execute_script("let sum = 0; for (let i = 0; i < 5; i++) { sum += i; } sum").unwrap();
    println!("Test loops result: {:?}", result);
}

#[test]
fn test_ffi() {
    let mut ts = TypeScript::new();

    // 测试 FFI
    let result = ts.execute_script("std.println('Hello from FFI'); 'FFI test'").unwrap();
    println!("Test FFI result: {:?}", result);
}
